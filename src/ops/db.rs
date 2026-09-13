use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::idalib::idb::{IDB, IDBOpenOptions};

use crate::helpers::json_types as jt;
use crate::ops::top::Out;

/// Open a database. If `idb_path` exists it is reopened directly (preserving
/// all state); otherwise it is created fresh from the binary. This is what
/// makes the CLI stateless yet persistent: the IDB file *is* the state.
pub fn open_db(
    binary: Option<&Path>,
    idb_path: &Path,
    auto_analyse: bool,
    save: bool,
) -> Result<IDB> {
    if idb_path.exists() {
        Ok(IDB::open(idb_path)?)
    } else {
        let Some(binary) = binary else {
            bail!(
                "IDB not found: {} (pass a binary to create it)",
                idb_path.display()
            );
        };
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

/// View of a database on disk.
pub fn db_info(db: &std::path::Path) -> Result<jt::DbView> {
    let (binary, idb) = crate::cli::resolve_db(db);
    let idb = idb.as_path();
    if !idb.exists() && binary.is_none() {
        bail!("database not found: {}", idb.display());
    }
    Ok(jt::DbView {
        input: db.display().to_string(),
        binary: binary.as_ref().map(|p| p.display().to_string()),
        idb: idb.display().to_string(),
        idb_exists: idb.exists(),
        size_bytes: std::fs::metadata(idb).map(|m| m.len()).unwrap_or(0),
    })
}

/// Create/open the IDB for a binary (or reopen an existing IDB) explicitly.
pub fn db_open(db: &std::path::Path, save: bool, auto_analyse: bool) -> Result<jt::DbView> {
    let (binary, idb) = crate::cli::resolve_db(db);
    let mut handle = open_db(binary.as_deref(), &idb, auto_analyse, save)?;
    handle.save_on_close(save);
    drop(handle);
    db_info(db)
}

/// Flush a database: open + close (no-op if the IDB does not exist yet).
pub fn db_close(db: &std::path::Path) -> Result<jt::OkView> {
    let (binary, idb) = crate::cli::resolve_db(db);
    if !idb.exists() {
        bail!("no IDB to close: {}", idb.display());
    }
    let mut handle = open_db(binary.as_deref(), &idb, true, true)?;
    handle.save_on_close(true);
    drop(handle);
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("database closed: {}", idb.display())),
    })
}

/// Delete the IDB file (and IDA's sibling .id0/.id1/.id2/.nam/.til if any).
pub fn db_remove(db: &std::path::Path) -> Result<jt::OkView> {
    let (binary, idb) = crate::cli::resolve_db(db);
    if !idb.exists() {
        bail!("no IDB to remove: {}", idb.display());
    }
    let removed: &mut Vec<String> = &mut Vec::new();
    let base = idb.with_extension("");
    for suffix in ["i64", "idb", "id0", "id1", "id2", "nam", "til"] {
        let p = base.with_extension(suffix);
        if p.exists() {
            std::fs::remove_file(&p).with_context(|| format!("remove {}", p.display()))?;
            removed.push(p.display().to_string());
        }
    }
    // The input itself was a binary: never delete it, just report.
    let _ = binary;
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("removed {}", removed.join(", "))),
    })
}

/// Run auto-analysis to completion (open + auto_wait + save).
pub fn db_analyze(db: &std::path::Path) -> Result<jt::OkView> {
    let (binary, idb) = crate::cli::resolve_db(db);
    let mut handle = open_db(binary.as_deref(), &idb, true, true)?;
    let _ = handle.auto_wait();
    handle.save_on_close(true);
    drop(handle);
    Ok(jt::OkView {
        ok: true,
        detail: Some(format!("analysis complete: {}", idb.display())),
    })
}

pub fn run(s: &crate::cli::DbCmd, global: Option<&std::path::Path>) -> Result<Out> {
    // Per-subcommand -d wins; fall back to the global -d / configured default.
    let take = |local: &Option<PathBuf>| -> Result<PathBuf> {
        local
            .clone()
            .or_else(|| global.map(|p| p.to_path_buf()))
            .or_else(|| {
                crate::session::config::Config::load()
                    .ok()
                    .and_then(|c| c.default_db)
            })
            .ok_or_else(|| anyhow::anyhow!("missing -d/--db <PATH>"))
    };
    match &s.command {
        crate::cli::DbAction::Open(o) => {
            Ok(Out::Db(db_open(&take(&o.db)?, o.save, o.auto_analyse)?))
        }
        crate::cli::DbAction::Info(i) => Ok(Out::Db(db_info(&take(&i.db)?)?)),
        crate::cli::DbAction::Close(c) => Ok(Out::Ok(db_close(&take(&c.db)?)?)),
        crate::cli::DbAction::Remove(r) => Ok(Out::Ok(db_remove(&take(&r.db)?)?)),
    }
}
