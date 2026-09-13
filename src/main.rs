use std::process::ExitCode;

use clap::Parser;
use idalib_cli::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    // Keep IDA's console output ("Thank you for using IDA...", loader messages)
    // off stdout so the JSON output contract stays clean.
    idalib_cli::idalib::enable_console_messages(false);

    // Apply runtime config (defaults for IDA dir, IDB dir, default db).
    let cfg = idalib_cli::session::config::Config::load()?;
    if let Some(dir) = cfg.idadir {
        if std::env::var_os("IDADIR").is_none() {
            // Safe: single-threaded startup, before any env access by IDA.
            unsafe { std::env::set_var("IDADIR", &dir) };
        }
    }

    idalib_cli::ops::top::dispatch(cli)
}
