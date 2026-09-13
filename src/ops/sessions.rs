use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::idalib::idb::{IDB, IDBOpenOptions};

use crate::helpers::json_types as jt;
use crate::ops::top::Out;
use crate::session::session::{IdbSpec, Session, SessionId, SessionState};
use crate::session::session_manager::SessionManager;

pub fn default_idb_path(binary: &std::path::Path) -> PathBuf {
    let file_name = binary
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "binary".to_string());
    let name = format!("{file_name}.i64");

    // Explicit `idb_dir` in the config wins; otherwise the IDB lives next to
    // the analysed binary.
    if let Ok(cfg) = crate::session::config::Config::load() {
        if let Some(dir) = cfg.idb_dir {
            return dir.join(name);
        }
    }
    match binary.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(name),
        _ => PathBuf::from(name),
    }
}

/// Open a session's database. If the IDB file already exists, it is reopened
/// directly (preserving all state); otherwise it is created fresh from the
/// input binary. This is what gives each session its own persistent IDB state.
pub fn open_db(
    binary: &PathBuf,
    idb_path: &PathBuf,
    auto_analyse: bool,
    save: bool,
) -> Result<IDB> {
    if idb_path.exists() {
        Ok(IDB::open(idb_path)?)
    } else {
        if let Some(parent) = idb_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create dir {}", parent.display()))?;
        }
        let db = IDBOpenOptions::new()
            .idb(idb_path)
            .save(save)
            .auto_analyse(auto_analyse)
            .open(binary)?;
        Ok(db)
    }
}

/// Create a session record and (re)build its IDB from the input binary. The
/// IDB is a file; once created, subsequent commands reopen it on demand.
pub fn create_session(
    mgr: &mut SessionManager,
    binary: PathBuf,
    idb: Option<PathBuf>,
    name: Option<String>,
    auto_analyse: bool,
    save: bool,
) -> Result<SessionId> {
    if !binary.exists() {
        bail!("binary not found: {}", binary.display());
    }
    let idb_path = idb.unwrap_or_else(|| default_idb_path(&binary));
    let id = mgr.create(binary, idb_path, auto_analyse, save, name);

    // Build the IDB from the binary now so the session is immediately usable.
    let (bin, idb_path, auto_analyse, save) = {
        let s = mgr.get(id).unwrap();
        (
            s.config.binary.clone(),
            s.config.idb.clone(),
            s.config.auto_analyse,
            s.config.save,
        )
    };
    let result = open_db(&bin, &idb_path, auto_analyse, save);
    match result {
        Ok(mut db) => {
            db.save_on_close(true);
            drop(db);
            let s = mgr.get_mut(id).unwrap();
            s.state = SessionState::Ready;
            s.error = None;
        }
        Err(e) => {
            let s = mgr.get_mut(id).unwrap();
            s.state = SessionState::Failed;
            s.error = Some(format!("{e:#}"));
            s.last_error = Some(format!("{e:#}"));
            return Err(anyhow::anyhow!("failed to open IDB: {e:#}"));
        }
    }
    Ok(id)
}

pub fn open_session(
    mgr: &mut SessionManager,
    binary: PathBuf,
    idb: Option<PathBuf>,
    name: Option<String>,
    auto_analyse: bool,
    save: bool,
) -> Result<jt::SessionView> {
    let id = create_session(mgr, binary, idb, name, auto_analyse, save)?;
    let s = mgr.get(id).unwrap();
    Ok(session_view(s))
}

pub fn list_sessions(mgr: &SessionManager) -> Vec<jt::SessionView> {
    mgr.list().iter().map(|s| session_view(s)).collect()
}

/// Resolve a session id: explicit request > global `--session` > first ready
/// session > first session.
pub fn resolve_id(
    mgr: &SessionManager,
    requested: Option<SessionId>,
    global: Option<SessionId>,
) -> Result<SessionId> {
    if let Some(id) = requested.or(global) {
        return Ok(id);
    }
    mgr.list()
        .iter()
        .find(|s| s.state == SessionState::Ready)
        .map(|s| s.id)
        .or_else(|| mgr.list().first().map(|s| s.id))
        .ok_or_else(|| anyhow::anyhow!("no sessions; create one with `session open -b <file>`"))
}

pub fn show_session(mgr: &SessionManager, id: SessionId) -> Result<jt::SessionView> {
    let s = mgr
        .get(id)
        .ok_or_else(|| anyhow::anyhow!("session {id} not found"))?;
    Ok(session_view(s))
}

pub fn close_session(
    mgr: &mut SessionManager,
    id: SessionId,
    save: Option<bool>,
) -> Result<jt::SessionView> {
    let s = mgr
        .get_mut(id)
        .ok_or_else(|| anyhow::anyhow!("session {id} not found"))?;
    if let Some(save) = save {
        s.config.save = save;
    }
    s.state = SessionState::Closed;
    Ok(session_view(s))
}

pub fn remove_session(mgr: &mut SessionManager, id: SessionId) -> Result<jt::OkView> {
    mgr.remove(id)
        .ok_or_else(|| anyhow::anyhow!("session {id} not found"))?;
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("session {id} removed")),
    })
}

pub fn save_session(mgr: &mut SessionManager, id: SessionId) -> Result<jt::OkView> {
    let s = mgr
        .get_mut(id)
        .ok_or_else(|| anyhow::anyhow!("session {id} not found"))?;
    s.config.save = true;
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("session {id} marked for save on close")),
    })
}

pub fn analyze_session(mgr: &mut SessionManager, id: SessionId, _wait: bool) -> Result<jt::OkView> {
    let s = mgr
        .get(id)
        .ok_or_else(|| anyhow::anyhow!("session {id} not found"))?;
    let binary = s.config.binary.clone();
    let idb_path = s.config.idb.clone();
    let auto_analyse = s.config.auto_analyse;
    let save = s.config.save;

    let mut db = open_db(&binary, &idb_path, auto_analyse, save)?;
    db.save_on_close(save);
    drop(db);

    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("session {id} analysed")),
    })
}

pub fn session_view(s: &Session) -> jt::SessionView {
    jt::SessionView {
        id: s.id.to_string(),
        name: s.name.clone(),
        state: s.state.as_str().to_string(),
        created_at: s.created_at.clone(),
        idb: Some(s.config.idb.display().to_string()),
        binary: Some(s.config.binary.display().to_string()),
        auto_analyse: Some(s.config.auto_analyse),
        save: Some(s.config.save),
        ready: s.state == SessionState::Ready,
        error: s.last_error.clone(),
    }
}

pub fn spec_from_cli(
    binary: PathBuf,
    idb: Option<PathBuf>,
    auto_analyse: bool,
    save: bool,
) -> Result<IdbSpec> {
    if !binary.exists() {
        bail!("binary not found: {}", binary.display());
    }
    Ok(IdbSpec {
        idb: idb.unwrap_or_else(|| default_idb_path(&binary)),
        binary,
        auto_analyse,
        save,
    })
}

pub fn run(
    mgr: &mut SessionManager,
    s: &crate::cli::SessionCmd,
    global: Option<SessionId>,
) -> Result<Out> {
    match &s.command {
        crate::cli::SessionAction::Open(o) => Ok(Out::Session(open_session(
            mgr,
            o.binary.clone(),
            o.idb.clone(),
            o.name.clone(),
            o.auto_analyse,
            o.save,
        )?)),
        crate::cli::SessionAction::List(_) => Ok(Out::Sessions(list_sessions(mgr))),
        crate::cli::SessionAction::Show(sh) => {
            let id = resolve_id(mgr, sh.session, global)?;
            Ok(Out::Session(show_session(mgr, id)?))
        }
        crate::cli::SessionAction::Close(c) => {
            let id = resolve_id(mgr, c.session, global)?;
            Ok(Out::Session(close_session(mgr, id, c.save)?))
        }
        crate::cli::SessionAction::Remove(r) => {
            let id = resolve_id(mgr, r.session, global)?;
            Ok(Out::Ok(remove_session(mgr, id)?))
        }
        crate::cli::SessionAction::Save(sv) => {
            let id = resolve_id(mgr, sv.session, global)?;
            Ok(Out::Ok(save_session(mgr, id)?))
        }
        crate::cli::SessionAction::Analyze(a) => {
            let id = resolve_id(mgr, a.session, global)?;
            Ok(Out::Ok(analyze_session(mgr, id, a.wait)?))
        }
    }
}
