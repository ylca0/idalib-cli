use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use serde::Serialize;

use crate::idalib::idb::{IDB, IDBOpenOptions};

pub type SessionId = u32;
pub type Address = u64;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(0);

pub static IDALIB_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Pending,
    Ready,
    Closed,
    Failed,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Pending => "pending",
            SessionState::Ready => "ready",
            SessionState::Closed => "closed",
            SessionState::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionConfig {
    pub id: SessionId,
    pub name: String,
    pub binary: PathBuf,
    pub idb: PathBuf,
    pub auto_analyse: bool,
    pub save: bool,
    pub timeout: Option<Duration>,
}

pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub config: SessionConfig,
    pub state: SessionState,
    pub idb: Option<IDB>,
    pub error: Option<String>,
    pub created_at: String,
    pub last_error: Option<String>,
}

impl Session {
    pub fn new(config: SessionConfig) -> Self {
        let created_at = crate::util::now_utc();
        Session {
            id: config.id,
            name: config.name.clone(),
            state: SessionState::Pending,
            config,
            idb: None,
            error: None,
            created_at,
            last_error: None,
        }
    }

    pub fn open(&mut self) -> Result<(), anyhow::Error> {
        let _guard = IDALIB_MUTEX.lock().unwrap();
        let binary = self.config.binary.clone();
        let idb_path = self.config.idb.clone();

        let open_result = IDBOpenOptions::new()
            .idb(&idb_path)
            .save(self.config.save)
            .auto_analyse(self.config.auto_analyse)
            .open(&binary);

        match open_result {
            Ok(idb) => {
                self.state = SessionState::Ready;
                self.error = None;
                self.last_error = None;
                self.idb = Some(idb);
                Ok(())
            }
            Err(e) => {
                self.state = SessionState::Failed;
                self.error = Some(format!("{e:#}"));
                self.last_error = Some(format!("{e:#}"));
                Err(anyhow!("failed to open IDB: {e:#}"))
            }
        }
    }

    pub fn close(&mut self) -> Result<(), anyhow::Error> {
        let mut idb = self.idb.take();
        if let Some(db) = idb.as_mut() {
            db.save_on_close(self.config.save);
        }
        self.idb = None;
        self.state = SessionState::Closed;
        let _ = &mut idb;
        Ok(())
    }

    pub fn with_idb<T>(&mut self, f: impl FnOnce(&mut IDB) -> Result<T>) -> Result<T> {
        let db = self.idb.as_mut().context("session not open")?;
        f(db)
    }
}

#[derive(Debug, Clone)]
pub struct IdbSpec {
    pub binary: PathBuf,
    pub idb: PathBuf,
    pub auto_analyse: bool,
    pub save: bool,
}
