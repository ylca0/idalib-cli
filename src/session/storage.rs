use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::session::session::{SessionConfig, SessionId, SessionState};
use crate::session::session_manager::SessionManager;

/// On-disk registry of sessions so they survive across CLI invocations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionRecord {
    pub id: SessionId,
    pub name: String,
    pub binary: PathBuf,
    pub idb: PathBuf,
    pub auto_analyse: bool,
    pub save: bool,
    pub created_at: String,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionRegistry {
    pub sessions: BTreeMap<SessionId, SessionRecord>,
    pub next_id: SessionId,
}

pub fn registry_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("sessions.json"))
}

/// Runtime base directory for idalib-cli state (registry, config, temp IDBs).
/// Defaults to `~/.idapro/idalib-cli`; override with `IDALIB_CLI_HOME`.
pub fn base_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("IDALIB_CLI_HOME") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Ok(PathBuf::from(home).join(".idapro").join("idalib-cli"))
}

pub fn load() -> Result<SessionRegistry> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(SessionRegistry::default());
    }
    let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let reg: SessionRegistry =
        serde_json::from_str(&data).with_context(|| format!("parse {}", path.display()))?;
    Ok(reg)
}

pub fn save(reg: &SessionRegistry) -> Result<()> {
    let path = registry_path()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("create dir {}", dir.display()))?;
    }
    let data = serde_json::to_string_pretty(reg)?;
    fs::write(&path, data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Rebuild an in-memory `SessionManager` from the on-disk registry.
pub fn restore_mgr(reg: &SessionRegistry) -> SessionManager {
    let max_id = reg.sessions.keys().max().copied().unwrap_or(0);
    let next_id = reg.next_id.max(max_id + 1).max(1);
    let mut mgr = SessionManager::new_with_next(next_id);
    for rec in reg.sessions.values() {
        let cfg = SessionConfig {
            id: rec.id,
            name: rec.name.clone(),
            binary: rec.binary.clone(),
            idb: rec.idb.clone(),
            auto_analyse: rec.auto_analyse,
            save: rec.save,
            timeout: None,
        };
        let mut s = crate::session::session::Session::new(cfg);
        s.state = match rec.state.as_str() {
            "ready" => SessionState::Ready,
            "closed" => SessionState::Closed,
            "failed" => SessionState::Failed,
            _ => SessionState::Pending,
        };
        s.error = rec.error.clone();
        s.last_error = rec.error.clone();
        s.created_at = rec.created_at.clone();
        mgr.sessions.insert(rec.id, s);
    }
    mgr
}

/// Serialize the current in-memory manager back to a registry for saving.
pub fn snapshot(mgr: &SessionManager) -> SessionRegistry {
    let mut reg = SessionRegistry {
        next_id: mgr.next_id(),
        sessions: BTreeMap::new(),
    };
    for s in mgr.list() {
        reg.sessions.insert(
            s.id,
            SessionRecord {
                id: s.id,
                name: s.name.clone(),
                binary: s.config.binary.clone(),
                idb: s.config.idb.clone(),
                auto_analyse: s.config.auto_analyse,
                save: s.config.save,
                created_at: s.created_at.clone(),
                state: s.state.as_str().to_string(),
                error: s.last_error.clone(),
            },
        );
    }
    reg
}
