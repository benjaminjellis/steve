pub(crate) mod config;
mod error;
pub(crate) mod logging;
mod prepare;
mod sync;
pub(crate) mod ui;
pub(crate) mod utils;

use crate::error::SteveError;
use clap::{Parser, Subcommand};
use std::process;

const DEFAULT_MAX_EPISODES: usize = 20;
pub(crate) const AUDIO_EXTENSIONS: [&str; 4] = ["mp3", "m4a", "mp4", "aac"];

// TODO: support these four in config
pub(crate) const IPOD_CONTENT_DIR: &str = "iPod Content";
pub(crate) const PODCASTS_CONTENT_DIR: &str = "Podcasts";
pub(crate) const MUSIC_CONTENT_DIR: &str = "Music";
pub(crate) const IPOD_ROOT: &str = "/Volumes/IPOD";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(styles= utils::get_styles())]
#[command(version = VERSION)]
#[command(name = "steve", about = "My iPod content management tool", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Download and prune podcast episodes from configured feeds.
    Prepare {
        #[arg(long)]
        dry_run: bool,
    },
    /// Mirror source into destination (1:1) using rsync.
    Sync {
        #[arg(long)]
        dry_run: bool,
    },
}

fn run(cli: Cli) -> Result<(), SteveError> {
    match cli.command {
        Commands::Prepare { dry_run } => prepare::run_prepare(dry_run),
        Commands::Sync { dry_run } => sync::run_sync(dry_run),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        ui::red_std_err(err.to_string());
        process::exit(1);
    }
}
