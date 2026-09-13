use std::process::ExitCode;

use clap::Parser;
use idalib_cli::cli::Cli;
use idalib_cli::session::storage;

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

    // Apply runtime config (defaults for IDA dir, IDB dir, save/auto-analyse).
    let cfg = idalib_cli::session::config::Config::load()?;
    if let Some(dir) = cfg.idadir {
        if std::env::var_os("IDADIR").is_none() {
            // Safe: single-threaded startup, before any env access by IDA.
            unsafe { std::env::set_var("IDADIR", &dir) };
        }
    }

    // Restore any persisted sessions (registry lives in ~/.idapro/idalib-cli/sessions.json).
    let reg = storage::load()?;
    let mut mgr = storage::restore_mgr(&reg);

    idalib_cli::ops::top::dispatch(cli, &mut mgr)?;

    // Persist session state so subsequent invocations (and parallel agents)
    // see the same sessions.
    let new_reg = storage::snapshot(&mgr);
    storage::save(&new_reg)?;
    Ok(())
}
