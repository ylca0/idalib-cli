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
        crate::cli::DbAction::Info(i) => Ok(Out::Db(db_info(&take(&i.db)?)?)),
    }
}
