use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::cli::ParallelCmd;
use crate::ops::top;
use crate::session::session_manager::SessionManager;

#[derive(Serialize, Debug)]
pub struct ParallelResult {
    pub session: u32,
    pub ok: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Run a command against many sessions concurrently. Each session is analysed
/// in its own subprocess, which lets IDA/IDALib instances (and their IDB files)
/// be processed truly in parallel — ideal for multi-agent / multi-process fan-out.
pub fn run(mgr: &SessionManager, p: &ParallelCmd) -> Result<top::Out> {
    let sessions = if p.sessions.is_empty() {
        mgr.list().iter().map(|s| s.id).collect::<Vec<_>>()
    } else {
        p.sessions.clone()
    };
    if sessions.is_empty() {
        bail!("no sessions to parallelise");
    }

    let jobs = if p.jobs == 0 { num_cpus::get() } else { p.jobs }.max(1);
    let bin = match &p.bin {
        Some(b) => b.clone(),
        None => current_exe()?,
    };

    let queue: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(sessions.clone()));
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::new();
    for _ in 0..jobs {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        let bin = bin.clone();
        let op = p.op.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                let next = {
                    let mut q = queue.lock().unwrap();
                    q.pop()
                };
                let Some(id) = next else { break };
                let res = run_one(&bin, &op, id);
                let _ = tx.send(res);
            }
        }));
    }
    drop(tx);
    for h in handles {
        let _ = h.join();
    }

    let mut results: Vec<ParallelResult> = rx.iter().collect();
    results.sort_by_key(|r| r.session);
    Ok(top::Out::Value(serde_json::json!({ "results": results })))
}

fn run_one(bin: &std::path::Path, op: &str, session: u32) -> ParallelResult {
    let argv = crate::ops::batch::shell_words_split(op)
        .map(|mut v| {
            v.insert(0, bin.display().to_string());
            v.push("--session".to_string());
            v.push(session.to_string());
            v
        })
        .unwrap_or_else(|_| vec![bin.display().to_string(), "--help".to_string()]);

    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    let out = cmd.output();
    match out {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            let json = serde_json::from_str(&stdout).unwrap_or(serde_json::Value::String(stdout));
            ParallelResult {
                session,
                ok: true,
                output: Some(json),
                error: None,
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            ParallelResult {
                session,
                ok: false,
                output: None,
                error: Some(stderr),
            }
        }
        Err(e) => ParallelResult {
            session,
            ok: false,
            output: None,
            error: Some(format!("{e}")),
        },
    }
}

fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().context("cannot determine own executable path")
}
