use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Args, Parser, Subcommand};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "idalib-cli",
    version = VERSION,
    about = "Agent-native CLI for the IDA Pro IDALib (v9.1) via idalib-rs",
    long_about = "A single-binary, agent-first command line interface to the IDA Pro IDALib.\n\n\
        Stateless: every command takes `-d/--db <PATH>` pointing at either an IDB\n\
        file (.i64) or a binary (an IDB is created next to it on first use). All\n\
        state lives in the IDB file itself - comments, bookmarks, names and\n\
        analysis results persist across invocations.\n\n\
        Output: every command emits a JSON document on stdout; errors go to\n\
        stderr with a non-zero exit code."
)]
#[command(after_help = "EXAMPLES:\n\
    # analyse a binary (IDB created next to it on first use)\n\
    idalib-cli -d ./target.bin functions\n\n\
    # reopen an existing IDB\n\
    idalib-cli -d ./target.i64 decompile -a 0x401000\n\n\
    # several commands in one process (IDB opened once)\n\
    idalib-cli -d ./target.bin batch -- \"meta\" \"segments\" \"functions -u\"\n\n\
    # same command across many IDBs concurrently (one subprocess each)\n\
    idalib-cli parallel -d \"./a.i64,./b.i64\" -- \"functions -u\"\n")]
pub struct Cli {
    /// Database to operate on: an IDB file (.i64) or a binary (IDB created
    /// next to it if absent). Required by every command.
    #[arg(short = 'd', long = "db", global = true, value_name = "PATH")]
    pub db: Option<PathBuf>,

    /// Emit machine-readable JSON (JSON is already the default)
    #[arg(short = 'j', long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Runtime, version and license information
    Info(InfoCmd),
    /// Database management: open (create IDB), info, close, remove
    Db(DbCmd),
    /// List all segments in the database
    Segments(SegmentsCmd),
    /// Find the segment containing an address
    SegmentsByRange(SegmentsByRangeCmd),
    /// List all functions
    Functions(FunctionsCmd),
    /// Show detailed info about one function (CFG, blocks, xrefs)
    Function(FunctionCmd),
    /// Disassemble a run of instructions
    Disasm(DisasmCmd),
    /// Decompile a function to pseudo-code
    Decompile(DecompileCmd),
    /// List strings in the database
    Strings(StringsCmd),
    /// List named locations
    Names(NamesCmd),
    /// List cross-references (to an address, or all)
    Xrefs(XrefsCmd),
    /// List entry points
    Entries(EntriesCmd),
    /// Show database metadata (file type, compiler, bitness, ...)
    Meta(MetaCmd),
    /// Show processor information
    Processor(ProcessorCmd),
    /// Show one instruction at an address
    Insn(InsnCmd),
    /// Read/write comments in the database
    Comments(CommentsCmd),
    /// Manage bookmarks
    Bookmarks(BookmarksCmd),
    /// Generate FLIRT signature files from the database
    Signatures(SignaturesCmd),
    /// Run several sub-commands in sequence against one database
    Batch(BatchCmd),
    /// Run a sub-command across many databases in parallel (one process each)
    Parallel(ParallelCmd),
}

// ---------------------------------------------------------------------------
// info
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct InfoCmd {
    /// Show IDA version
    #[arg(long)]
    pub version: bool,
    /// Show IDA installation/licence information
    #[arg(long)]
    pub ida: bool,
    /// Show all runtime information at once
    #[arg(long)]
    pub all: bool,
}

// ---------------------------------------------------------------------------
// db
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct DbCmd {
    #[command(subcommand)]
    pub command: DbAction,
}

#[derive(Subcommand, Debug)]
pub enum DbAction {
    /// Open (or create) the IDB for a binary; runs auto-analysis
    Open(DbOpenCmd),
    /// Show details about a database (paths, IDB state)
    Info(DbInfoCmd),
    /// Close the database (flush pending state; state already saved per-op)
    Close(DbCloseCmd),
    /// Remove the IDB file (and .id0/.id1/... siblings if any)
    Remove(DbRemoveCmd),
}

#[derive(Args, Debug)]
pub struct DbOpenCmd {
    /// Path to the IDB file or the input binary
    #[arg(short = 'd', long = "db", value_name = "PATH")]
    pub db: Option<PathBuf>,
    /// Save the IDB on close (default: on)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub save: bool,
    /// Run full auto-analysis (default: on)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub auto_analyse: bool,
}

#[derive(Args, Debug)]
pub struct DbInfoCmd {
    #[arg(short = 'd', long = "db", value_name = "PATH")]
    pub db: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DbCloseCmd {
    #[arg(short = 'd', long = "db", value_name = "PATH")]
    pub db: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DbRemoveCmd {
    /// Database to delete (the .i64 file, or a binary with its sibling IDB)
    #[arg(short = 'd', long = "db", value_name = "PATH")]
    pub db: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// shared arg structs
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct SegmentsCmd {}

#[derive(Args, Debug)]
pub struct SegmentsByRangeCmd {
    #[arg(short = 'a', long, value_parser = parse_hex)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct FunctionsCmd {
    /// Only include non-library, non-thunk functions
    #[arg(short = 'u', long)]
    pub user: bool,
}

#[derive(Args, Debug)]
pub struct FunctionCmd {
    /// Function address
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct DisasmCmd {
    /// Start address
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
    /// Number of instructions to disassemble (default: 8)
    #[arg(short = 'n', long, default_value_t = 8)]
    pub count: usize,
}

#[derive(Args, Debug)]
pub struct DecompileCmd {
    /// Function start address
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
    /// Include all blocks in the pseudo-code (not just reachable)
    #[arg(short = 'A', long)]
    pub all_blocks: bool,
}

#[derive(Args, Debug)]
pub struct StringsCmd {
    /// Also show hidden strings
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct NamesCmd {}

#[derive(Args, Debug)]
pub struct XrefsCmd {
    /// Address to query xrefs to
    #[arg(short = 'a', long, value_parser = parse_hex)]
    pub address: Option<u64>,
    /// Also include data xrefs (default: code only)
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct EntriesCmd {}

#[derive(Args, Debug)]
pub struct MetaCmd {}

#[derive(Args, Debug)]
pub struct ProcessorCmd {}

#[derive(Args, Debug)]
pub struct InsnCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

// ---------------------------------------------------------------------------
// edit commands
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct CommentsCmd {
    #[command(subcommand)]
    pub command: CommentsAction,
}

#[derive(Subcommand, Debug)]
pub enum CommentsAction {
    /// Read comment at an address
    Get(CommentsGetCmd),
    /// Set a (non-repeatable) comment at an address
    Set(CommentsSetCmd),
    /// Append a comment at an address
    Append(CommentsAppendCmd),
    /// Remove the comment at an address
    Remove(CommentsRemoveCmd),
}

#[derive(Args, Debug)]
pub struct CommentsGetCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct CommentsSetCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
    #[arg(short = 'c', long, required = true)]
    pub comment: String,
}

#[derive(Args, Debug)]
pub struct CommentsAppendCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
    #[arg(short = 'c', long, required = true)]
    pub comment: String,
}

#[derive(Args, Debug)]
pub struct CommentsRemoveCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct BookmarksCmd {
    #[command(subcommand)]
    pub command: BookmarksAction,
}

#[derive(Subcommand, Debug)]
pub enum BookmarksAction {
    /// Mark a bookmark at an address
    Add(BookmarksAddCmd),
    /// List all bookmarks
    List(BookmarksListCmd),
    /// Show description for a bookmark at an address
    Get(BookmarksGetCmd),
    /// Erase the bookmark at an address
    Remove(BookmarksRemoveCmd),
}

#[derive(Args, Debug)]
pub struct BookmarksAddCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
    #[arg(short = 'd', long, default_value = "")]
    pub description: String,
}

#[derive(Args, Debug)]
pub struct BookmarksListCmd {}

#[derive(Args, Debug)]
pub struct BookmarksGetCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct BookmarksRemoveCmd {
    #[arg(short = 'a', long, value_parser = parse_hex, required = true)]
    pub address: u64,
}

#[derive(Args, Debug)]
pub struct SignaturesCmd {
    /// Build .sig/.pat signature files from the IDB (IDA's make_signatures)
    #[arg(long)]
    pub make: bool,
    /// Only analyse patterns (no full analysis)
    #[arg(long)]
    pub only_pat: bool,
}

// ---------------------------------------------------------------------------
// batch / parallel
// ---------------------------------------------------------------------------

/// Run a sequence of commands against one database in a single invocation.
#[derive(Args, Debug)]
pub struct BatchCmd {
    /// The commands to run, each as a full command-line without the global
    /// -d flag (e.g. "functions -u", "decompile -a 0x401000")
    #[arg(required = true, value_name = "OP", trailing_var_arg = true)]
    pub ops: Vec<String>,
}

/// Run a command against a set of databases concurrently. Each database is
/// handled by its own subprocess (`idalib-cli -d <db> <op>`).
#[derive(Args, Debug)]
pub struct ParallelCmd {
    /// The command to run against each database (e.g. "decompile -a 0x401000")
    #[arg(required = true, value_name = "OP", allow_hyphen_values = true)]
    pub op: String,

    /// Databases to target: an IDB or binary path; also accepts a comma-
    /// separated list or a glob (default: none => required)
    #[arg(short = 'd', long = "db", required = true, value_name = "SPEC")]
    pub dbs: String,

    /// Maximum number of concurrent workers (default: number of CPUs)
    #[arg(long = "jobs", default_value_t = 0)]
    pub jobs: usize,

    /// Path to this executable (defaults to the running binary)
    #[arg(long, value_name = "PATH")]
    pub bin: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub fn parse_hex(s: &str) -> Result<u64, String> {
    let t = s.trim();
    let (neg, digits) = if let Some(rest) = t.strip_prefix('-') {
        (true, rest)
    } else {
        (false, t)
    };
    let v = u64::from_str_radix(digits.strip_prefix("0x").unwrap_or(digits), 16)
        .map_err(|e| format!("invalid hex address '{s}': {e}"))?;
    if neg { Ok(v.wrapping_neg()) } else { Ok(v) }
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
    s.parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|e| format!("invalid duration '{s}': {e}"))
}

/// Resolve a `-d/--db` value to (binary, idb) paths. If the path is an IDB
/// (.i64), the binary path is unknown (None) - reopening uses the IDB itself.
/// If it is a binary, the IDB path is `<binary>.i64` next to it (honouring
/// the configured `idb_dir`).
pub fn resolve_db(db: &Path) -> (Option<PathBuf>, PathBuf) {
    let ext = db.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "i64" || ext == "idb" {
        (None, db.to_path_buf())
    } else {
        (Some(db.to_path_buf()), default_idb_path(db))
    }
}

/// Default IDB path for a binary: next to it, `<binary>.i64`. An explicit
/// `idb_dir` in the config overrides the directory.
pub fn default_idb_path(binary: &Path) -> PathBuf {
    let file_name = binary
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "binary".to_string());
    let name = format!("{file_name}.i64");

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
