mod commands;
mod db;
mod endpoints;
mod fossil_mmr;
mod router;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use core::cmp::min;
use futures::future::join;
use log::{info, warn};
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

    /// Number of threads (Max = 1000)
    #[arg(short, long, default_value_t = db::DB_MAX_CONNECTIONS)]
    loopsize: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    Fix,
    Update,
    MMR,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();
    let should_terminate = Arc::new(AtomicBool::new(false));
    let terminate_clone = should_terminate.clone();

    setup_ctrlc_handler(Arc::clone(&should_terminate))?;

    let router = async {
        let res = router::initialize_router(should_terminate.clone()).await;
        match res {
            Ok(()) => info!("Router task completed"),
            Err(e) => warn!("Router task failed: {:?}", e),
        };
    };

    let updater = async {
        let res = match cli.mode {
            Mode::Fix => {
                commands::fill_gaps(cli.start, cli.end, Arc::clone(&terminate_clone)).await
            }
            Mode::Update => {
                commands::update_from(
                    cli.start,
                    cli.end,
                    min(cli.loopsize, db::DB_MAX_CONNECTIONS),
                    Arc::clone(&terminate_clone),
                )
                .await
            }
            Mode::MMR => commands::mmr(Arc::clone(&terminate_clone)).await,
        };

        match res {
            Ok(()) => info!("Updater task completed"),
            Err(e) => warn!("Updater task failed: {:?}", e),
        };
    };

    let _ = join(router, updater).await;

    Ok(())
}

fn setup_ctrlc_handler(should_terminate: Arc<AtomicBool>) -> Result<()> {
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C");
        info!("Waiting for current processes to finish...");
        should_terminate.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")
}
