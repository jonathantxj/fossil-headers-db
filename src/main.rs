mod commands;
mod db;
mod endpoints;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use core::cmp::min;
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[arg(value_enum)]
    mode: Mode,

    /// Start block number
    #[arg(short, long)]
    start: Option<i64>,

    /// End block number
    #[arg(short, long)]
    end: Option<i64>,

    /// Number of threads (Max = 4000)
    #[arg(short, long, default_value_t = db::DB_MAX_CONNECTIONS)]
    loopsize: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    Fix,
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();
    let should_terminate = Arc::new(AtomicBool::new(false));

    setup_ctrlc_handler(Arc::clone(&should_terminate))?;

    match cli.mode {
        Mode::Fix => {
            commands::fill_gaps(cli.start, cli.end, Arc::clone(&should_terminate))
                .await
                .context("Failed to fill gaps")?;
        }
        Mode::Update => {
            commands::update_from(
                cli.start,
                cli.end,
                min(cli.loopsize, db::DB_MAX_CONNECTIONS),
                Arc::clone(&should_terminate),
            )
            .await
            .context("Failed to update")?;
        }
    }

    Ok(())
}

fn setup_ctrlc_handler(should_terminate: Arc<AtomicBool>) -> Result<()> {
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C");
        info!("Waiting for current blocks to finish...");
        should_terminate.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")
}
