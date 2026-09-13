use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use super::session::{IdbSpec, Session, SessionConfig, SessionId, SessionState};

#[derive(Default)]
pub struct SessionManager {
    pub sessions: HashMap<SessionId, Session>,
    next_id: SessionId,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn new_with_next(next_id: SessionId) -> Self {
        SessionManager {
            sessions: HashMap::new(),
            next_id,
        }
    }

    pub fn next_id(&self) -> SessionId {
        self.next_id
    }

    pub fn create(
        &mut self,
        binary: PathBuf,
        idb: PathBuf,
        auto_analyse: bool,
        save: bool,
        name: Option<String>,
    ) -> SessionId {
        let id = self.next_id;
        self.next_id += 1;
        let cfg = SessionConfig {
            id,
            name: name.unwrap_or_else(|| format!("session-{id}")),
            binary,
            idb,
            auto_analyse,
            save,
            timeout: None,
        };
        let session = Session::new(cfg);
        self.sessions.insert(id, session);
        id
    }

    pub fn create_with_spec(&mut self, spec: IdbSpec, name: Option<String>) -> SessionId {
        self.create(spec.binary, spec.idb, spec.auto_analyse, spec.save, name)
    }

    pub fn get(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }

    pub fn get_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }

    pub fn remove(&mut self, id: SessionId) -> Option<Session> {
        self.sessions.remove(&id)
    }

    pub fn open(&mut self, id: SessionId) -> Result<()> {
        let s = self
            .sessions
            .get_mut(&id)
            .ok_or_else(|| anyhow!("session {id} not found"))?;
        s.open()
    }

    pub fn close(&mut self, id: SessionId) -> Result<()> {
        let s = self
            .sessions
            .get_mut(&id)
            .ok_or_else(|| anyhow!("session {id} not found"))?;
        s.close()
    }

    pub fn open_all(&mut self) -> Result<()> {
        let ids: Vec<SessionId> = self.sessions.keys().copied().collect();
        for id in ids {
            self.open(id)?;
        }
        Ok(())
    }

    pub fn close_all(&mut self) -> Result<()> {
        let ids: Vec<SessionId> = self.sessions.keys().copied().collect();
        for id in ids {
            let _ = self.close(id);
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<&Session> {
        let mut v: Vec<&Session> = self.sessions.values().collect();
        v.sort_by_key(|s| s.id);
        v
    }

    pub fn ready(&self, id: SessionId) -> bool {
        self.sessions
            .get(&id)
            .map(|s| s.state == SessionState::Ready)
            .unwrap_or(false)
    }

    pub fn state(&self, id: SessionId) -> Option<SessionState> {
        self.sessions.get(&id).map(|s| s.state.clone())
    }
}
