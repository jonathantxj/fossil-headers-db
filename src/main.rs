mod commands;
pub mod db;
mod endpoints;
mod type_utils;
pub mod types;

use clap::{Parser, ValueEnum};
use ctrlc;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// What mode to run the program in
    #[arg(value_enum)]
    mode: Mode,
    #[arg(short, long)]
    start: Option<i64>,
    #[arg(short, long)]
    end: Option<i64>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Fix,
    Update,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let recvd_terminate = Arc::new(Mutex::new(false));
    let fn_should_terminate = Arc::clone(&recvd_terminate);

    ctrlc::set_handler(move || {
        println!("Received Ctrl+C");
        println!("Waiting for current blocks to finish...");
        let mut terminate = recvd_terminate.lock().unwrap();
        *terminate = true;
    })
    .expect("Error setting Ctrl-C handler");

    match cli.mode {
        Mode::Fix => {
            commands::fill_gaps(cli.start, cli.end, fn_should_terminate).await;
        }
        Mode::Update => {
            commands::update_from(cli.start, cli.end, fn_should_terminate).await;
        }
    }
}
