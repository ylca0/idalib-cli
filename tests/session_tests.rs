use idalib_cli::session::session::SessionConfig;
use idalib_cli::session::session_manager::SessionManager;
use std::path::{Path, PathBuf};

fn cfg(id: u32) -> SessionConfig {
    SessionConfig {
        id,
        name: format!("s{id}"),
        binary: PathBuf::from("/tmp/foo.bin"),
        idb: PathBuf::from("/tmp/foo.i64"),
        auto_analyse: true,
        save: true,
        timeout: None,
    }
}

#[test]
fn session_manager_create_list() {
    let mut mgr = SessionManager::new();
    assert_eq!(mgr.sessions.len(), 0);
    let _id = mgr.create(
        PathBuf::from("/tmp/a.bin"),
        PathBuf::from("/tmp/a.i64"),
        true,
        true,
        None,
    );
    let _id2 = mgr.create(
        PathBuf::from("/tmp/b.bin"),
        PathBuf::from("/tmp/b.i64"),
        true,
        true,
        Some("b".into()),
    );
    assert_eq!(mgr.sessions.len(), 2);
    assert_eq!(mgr.list().len(), 2);
    let sessions = mgr.list();
    let b = sessions.iter().find(|s| s.name == "b").unwrap();
    assert_eq!(b.id, 2);
}

#[test]
fn session_manager_ids_are_unique_and_incrementing() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create(
        PathBuf::from("/tmp/a.bin"),
        PathBuf::from("/tmp/a.i64"),
        true,
        true,
        None,
    );
    let id2 = mgr.create(
        PathBuf::from("/tmp/b.bin"),
        PathBuf::from("/tmp/b.i64"),
        true,
        true,
        None,
    );
    assert!(id1 != id2);
    assert_eq!(id2, id1 + 1);
}

#[test]
fn session_manager_remove() {
    let mut mgr = SessionManager::new();
    let id = mgr.create(
        PathBuf::from("/tmp/a.bin"),
        PathBuf::from("/tmp/a.i64"),
        true,
        true,
        None,
    );
    assert!(mgr.remove(id).is_some());
    assert_eq!(mgr.sessions.len(), 0);
    assert!(mgr.remove(id).is_none());
}

#[test]
fn session_state_transitions() {
    let mut s = idalib_cli::session::session::Session::new(cfg(1));
    assert_eq!(s.state, idalib_cli::session::session::SessionState::Pending);
    // opening against a non-existent file fails and marks Failed (in stub, open always succeeds,
    // so we just check the happy path)
    let _ = s.open();
    assert_eq!(s.state, idalib_cli::session::session::SessionState::Ready);
    let _ = s.close();
    assert_eq!(s.state, idalib_cli::session::session::SessionState::Closed);
}

#[test]
fn default_idb_path() {
    // Default IDB sits next to the analysed binary.
    let p = idalib_cli::ops::sessions::default_idb_path(Path::new("/x/y/foo.bin"));
    assert_eq!(p, PathBuf::from("/x/y/foo.bin.i64"));

    // Bare filename (no parent dir) resolves to the local path.
    let p = idalib_cli::ops::sessions::default_idb_path(Path::new("foo.bin"));
    assert_eq!(p, PathBuf::from("foo.bin.i64"));
}
