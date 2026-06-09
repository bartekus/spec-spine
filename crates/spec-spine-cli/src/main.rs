//! `spec-spine` — the multi-call CLI. A thin wrapper over `spec-spine-core`:
//! it parses args, loads config, calls the engine, prints results, and maps the
//! typed `Error` to a stable process exit code. All `process::exit`, stdout, and
//! `git`/clock side effects live here, never in the library.

mod cmd_compile;
mod cmd_registry;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use spec_spine_types::{Config, Error};

#[derive(Parser)]
#[command(
    name = "spec-spine",
    version,
    about = "A typed, hash-verifiable authority ledger over a markdown spec corpus."
)]
struct Cli {
    /// Repository root (defaults to the current directory).
    #[arg(long, global = true, value_name = "DIR")]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile specs/*/spec.md into a deterministic registry.
    Compile,
    /// Read-only queries over the compiled registry.
    Registry {
        #[command(subcommand)]
        query: cmd_registry::RegistryQuery,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repo = match cli.repo {
        Some(p) => p,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let result = match &cli.command {
        Command::Compile => cmd_compile::run(&repo),
        Command::Registry { query } => cmd_registry::run(&repo, query),
    };

    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("spec-spine: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}

/// Load `<repo>/spec-spine.toml`, or the working default if it is absent.
pub(crate) fn load_repo_config(repo: &Path) -> Result<Config, Error> {
    let path = repo.join("spec-spine.toml");
    match std::fs::read_to_string(&path) {
        Ok(src) => spec_spine_types::load_config(&src),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(Error::Io(format!("read {}: {e}", path.display()))),
    }
}
