use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{self, Command};
use crate::idalib::idb::{IDB, IDBOpenOptions};
use crate::session::config::Config;

/// Top-level command output. Every command ultimately returns one of these,
/// which is serialised as JSON on stdout.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Out {
    Null,
    Bool(bool),
    U64(u64),
    I32(i32),
    String(String),
    Value(Value),
    Db(crate::helpers::json_types::DbView),
    Ok(crate::helpers::json_types::OkView),
    Segment(crate::helpers::json_types::SegmentView),
    Segments(Vec<crate::helpers::json_types::SegmentView>),
    Function(crate::helpers::json_types::FunctionView),
    Functions(Vec<crate::helpers::json_types::FunctionView>),
    FunctionDetail(crate::helpers::json_types::FunctionDetailView),
    Insn(crate::helpers::json_types::InsnView),
    Insns(Vec<crate::helpers::json_types::InsnView>),
    Xrefs(Vec<crate::helpers::json_types::XRefView>),
    Names(Vec<crate::helpers::json_types::NameView>),
    Strings(Vec<crate::helpers::json_types::StringView>),
    Metadata(crate::helpers::json_types::MetadataView),
    Processor(crate::helpers::json_types::ProcessorView),
    Entries(Vec<crate::helpers::json_types::EntryPointView>),
    Comment(crate::helpers::json_types::CmtView),
    Bookmark(crate::helpers::json_types::BookmarkView),
    Bookmarks(crate::helpers::json_types::BookmarksOut),
    Version(crate::helpers::json_types::VersionView),
    VersionInfo(crate::helpers::json_types::VersionInfo),
    License(crate::helpers::json_types::LicenseView),
}

pub fn dispatch(cli: &cli::Cli) -> Result<()> {
    let out = run(cli)?;
    print_out(&out);
    Ok(())
}

pub fn run(cli: &cli::Cli) -> Result<Out> {
    match &cli.command {
        Command::Info(i) => crate::ops::info::run(i),
        Command::Db(s) => crate::ops::db::run(s, cli.db.as_deref()),
        Command::Batch(b) => crate::ops::batch::run(cli, b),
        Command::Parallel(p) => crate::ops::parallel::run(p),
        _ => with_idb(cli, |idb| run_db_op(cli, idb)),
    }
}

/// Dispatch the DB-bound commands (everything that needs an open IDB).
pub(crate) fn run_db_op(cli: &cli::Cli, idb: &mut IDB) -> Result<Out> {
    match &cli.command {
        Command::Segments(_) => Ok(Out::Segments(crate::ops::metadata::segments(idb))),
        Command::SegmentsByRange(r) => match crate::ops::metadata::segment_at(idb, r.address) {
            Some(s) => Ok(Out::Segment(s)),
            None => Ok(Out::Null),
        },
        Command::Functions(f) => Ok(Out::Functions(crate::ops::metadata::functions(idb, f.user))),
        Command::Function(f) => Ok(Out::FunctionDetail(crate::ops::metadata::function(
            idb, f.address,
        )?)),
        Command::Disasm(d) => Ok(Out::Insns(crate::ops::metadata::disasm(
            idb, d.address, d.count,
        )?)),
        Command::Decompile(d) => Ok(Out::Function(crate::ops::metadata::decompile(
            idb,
            d.address,
            d.all_blocks,
        )?)),
        Command::Strings(s) => Ok(Out::Strings(crate::ops::metadata::strings(idb, s.all))),
        Command::Names(_) => Ok(Out::Names(crate::ops::metadata::names(idb))),
        Command::Xrefs(x) => {
            if x.from {
                let Some(ea) = x.address else {
                    anyhow::bail!("xrefs --from requires -a <ea>");
                };
                Ok(Out::Xrefs(crate::ops::search::xrefs_from(idb, ea, x.all)?))
            } else {
                Ok(Out::Xrefs(crate::ops::metadata::xrefs(
                    idb, x.address, x.all,
                )?))
            }
        }
        Command::Entries(_) => Ok(Out::Entries(crate::ops::metadata::entries(idb))),
        Command::Meta(_) => Ok(Out::Metadata(crate::ops::metadata::meta(idb))),
        Command::Processor(_) => Ok(Out::Processor(crate::ops::metadata::processor(idb))),
        Command::Insn(i) => Ok(Out::Insn(crate::ops::metadata::insn(idb, i.address)?)),
        Command::Find(f) => crate::ops::search::find(
            idb,
            f.text.as_deref(),
            f.imm,
            f.pattern.as_deref(),
            f.start,
            256,
        ),
        Command::Bytes(b) => crate::ops::search::bytes(idb, b.address, b.count, b.width.as_deref()),
        Command::Rename(r) => crate::ops::search::rename(idb, r.address, &r.name),
        Command::Comments(c) => Ok(Out::Comment(crate::ops::comments::dispatch(idb, c)?)),
        Command::Bookmarks(b) => Ok(Out::Bookmarks(crate::ops::bookmarks::dispatch(idb, b)?)),
        Command::Signatures(s) => Ok(Out::Ok(crate::ops::signatures::dispatch(idb, s)?)),
        _ => unreachable!("non-DB command reached run_db_op"),
    }
}

/// Resolve `-d/--db` to (binary, idb) honouring the config default.
pub fn resolve_target(cli: &cli::Cli) -> Result<(Option<PathBuf>, PathBuf, bool, bool)> {
    let cfg = Config::load().unwrap_or_default();
    let input = cli
        .db
        .clone()
        .or_else(|| cfg.default_db.clone())
        .ok_or_else(|| anyhow::anyhow!("missing -d/--db <PATH> (an IDB file or a binary)"))?;
    let (binary, idb) = cli::resolve_db(&input);
    Ok((
        binary,
        idb,
        cfg.save.unwrap_or(true),
        cfg.auto_analyse.unwrap_or(true),
    ))
}

/// Open the target database, run the op, close (persisting state). Stateless:
/// nothing is kept in memory or in a registry - the IDB file is the state.
pub(crate) fn with_idb<T, F>(cli: &cli::Cli, f: F) -> Result<T>
where
    F: FnOnce(&mut IDB) -> Result<T>,
{
    let (binary, idb_path, _save, auto_analyse) = resolve_target(cli)?;

    let mut db = match binary {
        Some(bin) => crate::ops::db::open_db(Some(&bin), &idb_path, auto_analyse, true)
            .context("failed to open database")?,
        None => IDB::open(&idb_path).context("failed to open database")?,
    };
    let result = f(&mut db);
    db.save_on_close(true);
    drop(db);
    result
}

/// Same as `with_idb` for closures returning `Result<()>` (used by batch).
pub(crate) fn with_idb_ref<F>(cli: &cli::Cli, f: F) -> Result<()>
where
    F: FnOnce(&mut IDB) -> Result<()>,
{
    let (binary, idb_path, _save, auto_analyse) = resolve_target(cli)?;

    let mut db = match binary {
        Some(bin) => crate::ops::db::open_db(Some(&bin), &idb_path, auto_analyse, true)
            .context("failed to open database")?,
        None => IDB::open(&idb_path).context("failed to open database")?,
    };
    let result = f(&mut db);
    db.save_on_close(true);
    drop(db);
    result
}

pub fn print_out<T: Serialize>(v: &T) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

// re-export for op modules that need IDBOpenOptions
#[allow(unused_imports)]
use IDBOpenOptions as _;
