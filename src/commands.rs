use std::env::Args;
use std::path::PathBuf;

use cargo_set_lib::{CargoManifestService, RealFileSystem};
use clap::{Parser, Subcommand, ValueEnum};

pub fn cli_execute(args: Args) -> anyhow::Result<()> {
    let cli = Cli::parse_from(args);
    let cargo_manifest_service = CargoManifestService::new(RealFileSystem);

    match &cli.log_level {
        Some(level) => {
            tracing_subscriber::fmt()
                .with_max_level(level)
                .pretty()
                .init();
        }
        None => {}
    }

    match &cli.command {
        Some(Commands::Set {
            workspace,
            _crate,
            path,
            set_version,
            bump,
        }) => {
            tracing::trace!(
                workspace = workspace,
                crate = _crate,
                path = path.as_ref().unwrap().display().to_string(),
                set_version = set_version.as_ref(),
                "command - set"
            );

            let mut manifest = cargo_manifest_service.load_manifest(path.as_ref().unwrap())?;

            if let Some(set_version) = set_version {
                cargo_manifest_service.update_version(&mut manifest, _crate, set_version)?;
            } else if let Some(_bump_level) = bump {
                todo!("haven't implemented bump yet")
            }
        }
        None => {}
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Into<tracing::metadata::LevelFilter> for &LogLevel {
    fn into(self) -> tracing::metadata::LevelFilter {
        match self {
            LogLevel::Trace => tracing::metadata::LevelFilter::TRACE,
            LogLevel::Debug => tracing::metadata::LevelFilter::DEBUG,
            LogLevel::Info => tracing::metadata::LevelFilter::INFO,
            LogLevel::Warn => tracing::metadata::LevelFilter::WARN,
            LogLevel::Error => tracing::metadata::LevelFilter::ERROR,
            LogLevel::Off => tracing::metadata::LevelFilter::OFF,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(subcommand_required = true)]
pub struct Cli {
    name: Option<String>,

    #[arg(global = true, help_heading = "Globals", long, default_value = "info")]
    log_level: Option<LogLevel>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Set {
        #[arg(long, default_missing_value = "true", default_value = "false")]
        workspace: bool,

        #[arg(long, name = "crate")]
        _crate: String,

        #[arg(long, default_value = "Cargo.toml")]
        path: Option<PathBuf>,

        #[arg(long, conflicts_with = "bump", required_unless_present = "bump")]
        set_version: Option<String>,

        #[arg(long, required_unless_present = "set_version")]
        bump: Option<BumpLevel>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BumpLevel {
    Patch,
    Minor,
    Major,
}
