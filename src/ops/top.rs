use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::cli::{self, Command};
use crate::session::session::{SessionId, SessionState};
use crate::session::session_manager::SessionManager;

/// Top-level command output. Every command ultimately returns one of these,
/// which is serialised as JSON (or rendered as text) to stdout.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Out {
    Null,
    Bool(bool),
    U64(u64),
    I32(i32),
    String(String),
    Value(Value),
    Session(crate::helpers::json_types::SessionView),
    Sessions(Vec<crate::helpers::json_types::SessionView>),
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
    BookmarksList(Vec<crate::helpers::json_types::BookmarkView>),
    Version(crate::helpers::json_types::VersionView),
    VersionInfo(crate::helpers::json_types::VersionInfo),
    License(crate::helpers::json_types::LicenseView),
}

pub fn dispatch(cli: &cli::Cli, mgr: &mut SessionManager) -> Result<()> {
    let out = run(cli, mgr)?;
    print_out(&out);
    Ok(())
}

pub fn run(cli: &cli::Cli, mgr: &mut SessionManager) -> Result<Out> {
    match &cli.command {
        Command::Info(i) => crate::ops::info::run(i),
        Command::Session(s) => crate::ops::sessions::run(mgr, s, cli.session),
        Command::Segments(_) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Segments(crate::ops::metadata::segments(idb)))
        }),
        Command::SegmentsByRange(r) => {
            with_idb(
                mgr,
                cli.session,
                |idb| match crate::ops::metadata::segment_at(idb, r.address) {
                    Some(s) => Ok(Out::Segment(s)),
                    None => Ok(Out::Null),
                },
            )
        }
        Command::Functions(f) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Functions(crate::ops::metadata::functions(idb, f.user)))
        }),
        Command::Function(f) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::FunctionDetail(crate::ops::metadata::function(
                idb, f.address,
            )?))
        }),
        Command::Disasm(d) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Insns(crate::ops::metadata::disasm(
                idb, d.address, d.count,
            )?))
        }),
        Command::Decompile(d) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Function(crate::ops::metadata::decompile(
                idb,
                d.address,
                d.all_blocks,
            )?))
        }),
        Command::Strings(s) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Strings(crate::ops::metadata::strings(idb, s.all)))
        }),
        Command::Names(_) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Names(crate::ops::metadata::names(idb)))
        }),
        Command::Xrefs(x) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Xrefs(crate::ops::metadata::xrefs(
                idb, x.address, x.all,
            )?))
        }),
        Command::Entries(_) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Entries(crate::ops::metadata::entries(idb)))
        }),
        Command::Meta(_) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Metadata(crate::ops::metadata::meta(idb)))
        }),
        Command::Processor(_) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Processor(crate::ops::metadata::processor(idb)))
        }),
        Command::Insn(i) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Insn(crate::ops::metadata::insn(idb, i.address)?))
        }),
        Command::Comments(c) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Comment(crate::ops::comments::dispatch(idb, c)?))
        }),
        Command::Bookmarks(b) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Bookmarks(crate::ops::bookmarks::dispatch(idb, b)?))
        }),
        Command::Signatures(s) => with_idb(mgr, cli.session, |idb| {
            Ok(Out::Ok(crate::ops::signatures::dispatch(idb, s)?))
        }),
        Command::Batch(b) => crate::ops::batch::run(mgr, b),
        Command::Parallel(p) => crate::ops::parallel::run(mgr, p),
    }
}

/// Select the session to operate on. Uses `-s/--session` if supplied; otherwise
/// the first ready session; otherwise the first session.
fn select_session(
    mgr: &SessionManager,
    requested: Option<SessionId>,
) -> Result<&crate::session::session::Session> {
    if let Some(id) = requested {
        return mgr
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("session {id} not found"));
    }
    mgr.sessions
        .iter()
        .find(|(_, s)| s.state == SessionState::Ready)
        .map(|(_, s)| s)
        .or_else(|| mgr.sessions.iter().next().map(|(_, s)| s))
        .ok_or_else(|| {
            anyhow::anyhow!("no sessions; create one with `idalib-cli session open -b <file>`")
        })
}

/// Open the target session's IDB from disk (or create it fresh), run the op,
/// and close it (saving per the session's `save` setting). This is what makes
/// sessions stateful across CLI invocations: the IDB file is the persisted state.
fn with_idb<T, F>(mgr: &mut SessionManager, requested: Option<SessionId>, f: F) -> Result<T>
where
    T: Serialize,
    F: FnOnce(&mut crate::idalib::idb::IDB) -> Result<T>,
{
    let (id, binary, idb_path, auto_analyse, save) = {
        let s = select_session(mgr, requested)?;
        (
            s.id,
            s.config.binary.clone(),
            s.config.idb.clone(),
            s.config.auto_analyse,
            s.config.save,
        )
    };

    let mut db = crate::ops::sessions::open_db(&binary, &idb_path, auto_analyse, save)
        .context("failed to open session IDB")?;
    let result = f(&mut db);
    // Save the database if the session is configured to persist changes.
    db.save_on_close(save);
    drop(db);

    let s = mgr.get_mut(id).unwrap();
    match &result {
        Ok(_) => {
            s.state = SessionState::Ready;
            s.error = None;
        }
        Err(e) => {
            s.last_error = Some(format!("{e:#}"));
        }
    }
    result
}

pub fn print_out<T: Serialize>(v: &T) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}
