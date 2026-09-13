use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::cli::ParallelCmd;
use crate::ops::top;

#[derive(Serialize, Debug)]
pub struct ParallelResult {
    pub db: String,
    pub ok: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Expand the `-d/--db` spec into a list of database paths. Accepts a single
/// path, a comma-separated list, or a glob pattern.
fn expand_dbs(spec: &str) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if p.contains('*') || p.contains('?') {
            let matched = glob_paths(p)?;
            if matched.is_empty() {
                bail!("glob matched nothing: {p}");
            }
            out.extend(matched);
        } else {
            out.push(PathBuf::from(p));
        }
    }
    if out.is_empty() {
        bail!("no databases given");
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn glob_paths(pattern: &str) -> Result<Vec<PathBuf>> {
    // Minimal glob: shell-expand via the pattern's directory + manual match
    // (avoids adding a glob dependency).
    let (dir, pat) = match pattern.rfind('/') {
        Some(i) => (&pattern[..=i], &pattern[i + 1..]),
        None => ("", pattern),
    };
    let entries = std::fs::read_dir(if dir.is_empty() { "." } else { dir })
        .with_context(|| format!("read dir {dir}"))?;
    let mut out = Vec::new();
    for e in entries {
        let e = e?;
        let name = e.file_name().to_string_lossy().to_string();
        if glob_match(pat, &name) {
            out.push(e.path());
        }
    }
    Ok(out)
}

fn glob_match(pat: &str, name: &str) -> bool {
    // '*' matches any run of chars, '?' matches exactly one char.
    let pc: Vec<char> = pat.chars().collect();
    let nc: Vec<char> = name.chars().collect();
    fn m(p: &[char], n: &[char]) -> bool {
        if p.is_empty() {
            return n.is_empty();
        }
        match p[0] {
            '*' => (0..=n.len()).any(|i| m(&p[1..], &n[i..])),
            '?' => !n.is_empty() && m(&p[1..], &n[1..]),
            c => !n.is_empty() && n[0] == c && m(&p[1..], &n[1..]),
        }
    }
    m(&pc, &nc)
}

/// Run a command against many databases concurrently. Each database is
/// analysed in its own subprocess (`idalib-cli -d <db> <op>`), which lets
/// IDA/IDALib instances (and their IDB files) be processed truly in parallel.
pub fn run(p: &ParallelCmd) -> Result<top::Out> {
    let dbs = expand_dbs(&p.dbs)?;

    let jobs = if p.jobs == 0 { num_cpus::get() } else { p.jobs }.max(1);
    let bin = match &p.bin {
        Some(b) => b.clone(),
        None => current_exe()?,
    };

    let queue: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(dbs.clone()));
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
                let Some(db) = next else { break };
                let res = run_one(&bin, &op, &db);
                let _ = tx.send(res);
            }
        }));
    }
    drop(tx);
    for h in handles {
        let _ = h.join();
    }

    let mut results: Vec<ParallelResult> = rx.iter().collect();
    results.sort_by(|a, b| a.db.cmp(&b.db));
    Ok(top::Out::Value(serde_json::json!({ "results": results })))
}

fn run_one(bin: &std::path::Path, op: &str, db: &std::path::Path) -> ParallelResult {
    let display = db.display().to_string();
    let argv = crate::ops::batch::shell_words_split(op)
        .map(|mut v| {
            v.insert(0, bin.display().to_string());
            v.push("-d".to_string());
            v.push(display.clone());
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
                db: display,
                ok: true,
                output: Some(json),
                error: None,
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            ParallelResult {
                db: display,
                ok: false,
                output: None,
                error: Some(stderr),
            }
        }
        Err(e) => ParallelResult {
            db: display,
            ok: false,
            output: None,
            error: Some(format!("{e}")),
        },
    }
}

fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().context("cannot determine own executable path")
}
