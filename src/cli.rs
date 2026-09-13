use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "idalib-cli",
    version = VERSION,
    about = "Agent-native CLI for the IDA Pro IDALib (v9.1) via idalib-rs",
    long_about = "A single-binary, agent-first command line interface to the IDA Pro IDALib.\n\n\
        Sessions: each session is an independent, stateful IDA/IDALib analysis process with\n\
        its own IDB state.  Sessions can be opened, inspected and closed independently, and\n\
        many sessions can be analysed in parallel across multiple processes/agents.\n\n\
        Output: every command emits structured JSON on stdout by default, which makes the\n\
        tool directly usable by code agents (Codex, Claude Code, OpenCode, ...).  Use\n\
        --json/--raw for machine consumption, or plain human-readable text otherwise."
)]
#[command(after_help = "EXAMPLES:\n\
    # create + open + analyse a binary in one go\n\
    idalib-cli session open -b ./target.bin\n\n\
    # list sessions\n\
    idalib-cli session list\n\n\
    # decompile a function inside session 1\n\
    idalib-cli decompile -s 1 -a 0x401000\n\n\
    # dump all functions as JSON (machine readable)\n\
    idalib-cli funcs -s 1 --json\n")]
pub struct Cli {
    /// Emit machine-readable JSON instead of human text
    #[arg(short = 'j', long, global = true)]
    pub json: bool,

    /// Session to operate on (for commands that need a database). If omitted,
    /// the first ready session is used.
    #[arg(short = 's', long, global = true, value_name = "SESSION")]
    pub session: Option<SessionId>,

    /// Run several commands in a single invocation (session is optional for all)
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Query and initialise the IDA/IDALib runtime
    Info(InfoCmd),
    /// Manage analysis sessions (each is an independent stateful IDA process)
    Session(SessionCmd),
    /// Dump low-level database details
    Segments(SegmentsCmd),
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
    /// Find the segment containing an address
    SegmentsByRange(SegmentsByRangeCmd),
    /// Show one instruction at an address
    Insn(InsnCmd),
    /// Read/write comments in the database
    Comments(CommentsCmd),
    /// Manage bookmarks
    Bookmarks(BookmarksCmd),
    /// Generate FLIRT signature files from the database
    Signatures(SignaturesCmd),
    /// Run several sub-commands in sequence against one session
    Batch(BatchCmd),
    /// Run a sub-command across many sessions in parallel (one process each)
    Parallel(ParallelCmd),
}

/// Run a command against a set of sessions concurrently. Each session is
/// handled by its own subprocess (`idalib-cli <op> -s <id>`), so independent
/// IDB files are analysed in parallel across multiple processes/agents.
#[derive(Args, Debug)]
pub struct ParallelCmd {
    /// The command to run against each session (e.g. "decompile -a 0x401000").
    /// Wrap the whole op in quotes; flags inside it are passed through verbatim.
    #[arg(required = true, value_name = "OP", allow_hyphen_values = true)]
    pub op: String,

    /// Sessions to target (repeatable; default: all sessions)
    #[arg(short = 'S', long = "sessions", value_delimiter = ',')]
    pub sessions: Vec<SessionId>,

    /// Maximum number of concurrent workers (default: number of CPUs)
    #[arg(long = "jobs", default_value_t = 0)]
    pub jobs: usize,

    /// Path to this executable (defaults to the running binary)
    #[arg(long, value_name = "PATH")]
    pub bin: Option<std::path::PathBuf>,
}

/// Run a sequence of commands against a single session in one invocation.
#[derive(Args, Debug)]
pub struct BatchCmd {
    /// The commands to run, each as a full command-line (e.g. "functions -u",
    /// "decompile -a 0x401000"). Commands are executed in order; the last one's
    /// output is emitted on stdout (all outputs are available in `--json` mode).
    #[arg(required = true, value_name = "OP", trailing_var_arg = true)]
    pub ops: Vec<String>,

    /// Session to operate on (defaults to first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
}

// ---------------------------------------------------------------------------
// info
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct InfoCmd {
    /// Show version information
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
// session
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct SessionCmd {
    #[command(subcommand)]
    pub command: SessionAction,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    /// Create a new session (IDB is opened immediately)
    Open(SessionOpenCmd),
    /// List all sessions
    List(SessionListCmd),
    /// Show details for one session
    Show(SessionShowCmd),
    /// Close (and optionally save) a session, releasing its IDB
    Close(SessionCloseCmd),
    /// Destroy a session (remove it from the registry)
    Remove(SessionRemoveCmd),
    /// Save the IDB of a session
    Save(SessionSaveCmd),
    /// Run an analysis (auto-wait) on a session
    Analyze(SessionAnalyzeCmd),
}

#[derive(Args, Debug)]
pub struct SessionOpenCmd {
    /// Path to the input binary
    #[arg(short = 'b', long, required = true)]
    pub binary: PathBuf,
    /// Path where the IDB will be stored (defaults to <binary>.i64)
    #[arg(short = 'o', long)]
    pub idb: Option<PathBuf>,
    /// Optional name for the session
    #[arg(short = 'n', long)]
    pub name: Option<String>,
    /// Run full auto-analysis (default: on)
    #[arg(short = 'a', long, default_value_t = true, action = clap::ArgAction::Set)]
    pub auto_analyse: bool,
    /// Save IDB on close (default: on)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub save: bool,
    /// Enable IDA console messages (may be noisy)
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct SessionListCmd {}

#[derive(Args, Debug)]
pub struct SessionShowCmd {
    /// Session id (falls back to the global --session or the first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
}

#[derive(Args, Debug)]
pub struct SessionCloseCmd {
    /// Session id (falls back to the global --session or the first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
    /// Save the IDB before closing (default: session's configured value)
    #[arg(long)]
    pub save: Option<bool>,
}

#[derive(Args, Debug)]
pub struct SessionRemoveCmd {
    /// Session id (falls back to the global --session or the first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
}

#[derive(Args, Debug)]
pub struct SessionSaveCmd {
    /// Session id (falls back to the global --session or the first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
}

#[derive(Args, Debug)]
pub struct SessionAnalyzeCmd {
    /// Session id (falls back to the global --session or the first ready session)
    #[arg(short = 's', long)]
    pub session: Option<SessionId>,
    /// Wait until auto-analysis completes (default: on)
    #[arg(long, default_value_t = true)]
    pub wait: bool,
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
    /// Show string addresses too (in text mode)
    #[arg(long)]
    pub addresses: bool,
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
    /// Path to store generated signatures (optional; must be a directory)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub type SessionId = u32;

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
