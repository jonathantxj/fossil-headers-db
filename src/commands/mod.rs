use anyhow::{Context, Result};
use futures_util::future::join_all;
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

use crate::{db, endpoints, types::type_utils};

const MAX_RETRIES: u32 = 5;

pub async fn fill_gaps(
    start: Option<i64>,
    end: Option<i64>,
    should_terminate: Arc<AtomicBool>,
) -> Result<()> {
    db::create_tables()
        .await
        .context("Failed to create tables")?;

    let range_start_pointer = start.unwrap_or(0).max(0);
    let range_end = get_range_end(end).await?;

    if range_end < 0 || range_start_pointer == range_end {
        info!("Empty database");
        return Ok(());
    }

    fill_missing_blocks(range_start_pointer, range_end, &should_terminate).await
}

async fn fill_missing_blocks(
    mut range_start_pointer: i64,
    range_end: i64,
    should_terminate: &AtomicBool,
) -> Result<()> {
    while !should_terminate.load(Ordering::Relaxed) {
        match db::find_first_gap(range_start_pointer, range_end).await? {
            Some(block_number) => {
                info!("[fill_gaps] Found missing block number: {}", block_number);
                if process_missing_block(block_number, &mut range_start_pointer).await? {
                    continue;
                }
                return Ok(());
            }
            None => {
                info!(
                    "[fill_gaps] No missing values found from {} to {}",
                    range_start_pointer, range_end
                );
                return Ok(());
            }
        }
    }
    Ok(())
}

async fn process_missing_block(block_number: i64, range_start_pointer: &mut i64) -> Result<bool> {
    for i in 0..MAX_RETRIES {
        match endpoints::get_full_block_by_number(block_number).await {
            Ok(block) => {
                db::write_blockheader(block).await?;
                *range_start_pointer = block_number + 1;
                info!(
                    "[fill_gaps] Successfully wrote block {} after {} retries",
                    block_number, i
                );
                return Ok(true);
            }
            Err(e) => warn!("[fill_gaps] Error retrieving block {}: {}", block_number, e),
        }
    }
    error!("[fill_gaps] Error with block number {}", block_number);
    Ok(false)
}

async fn get_range_end(end: Option<i64>) -> Result<i64> {
    Ok(match end {
        Some(s) => s,
        None => db::get_last_stored_blocknumber()
            .await
            .context("[fill_gaps] Error retrieving last_recorded_block")?,
    })
}

pub async fn update_from(
    start: Option<i64>,
    end: Option<i64>,
    size: u32,
    should_terminate: Arc<AtomicBool>,
) -> Result<()> {
    db::create_tables()
        .await
        .context("Failed to create tables")?;

    let first_missing_block = get_first_missing_block(start).await?;
    info!("First missing block: {}", first_missing_block);

    let last_block = get_last_block(end).await?;
    info!("Range end: {}", last_block);

    update_blocks(first_missing_block, last_block, size, &should_terminate).await
}

async fn update_blocks(
    first_missing_block: i64,
    last_block: i64,
    size: u32,
    should_terminate: &AtomicBool,
) -> Result<()> {
    let time_started = Instant::now();

    for n in (first_missing_block..=last_block - size as i64).step_by(size as usize) {
        if should_terminate.load(Ordering::Relaxed) {
            info!("Termination requested. Stopping update process.");
            break;
        }

        let range_end = (last_block).min(n + size as i64);

        let tasks: Vec<_> = (n..range_end)
            .map(|block_number| task::spawn(process_block(block_number)))
            .collect();

        join_all(tasks).await;
        info!("Written blocks {} - {}", n, range_end);
    }

    info!("Time elapsed: {:?}", time_started.elapsed());
    Ok(())
}

async fn process_block(block_number: i64) -> Result<()> {
    for i in 0..MAX_RETRIES {
        match endpoints::get_full_block_by_number(block_number).await {
            Ok(block) => match db::write_blockheader(block).await {
                Ok(_) => {
                    if i > 0 {
                        info!(
                            "[update_from] Successfully wrote block {} after {} retries",
                            block_number, i
                        );
                    }
                    return Ok(());
                }
                Err(e) => warn!("[update_from] Error writing block {}: {}", block_number, e),
            },
            Err(e) => warn!(
                "[update_from] Error retrieving block {}: {}",
                block_number, e
            ),
        }
    }
    error!("[update_from] Error with block number {}", block_number);
    Err(anyhow::anyhow!("Failed to process block {}", block_number))
}

async fn get_first_missing_block(start: Option<i64>) -> Result<i64> {
    Ok(match start {
        Some(s) => s,
        None => {
            db::get_last_stored_blocknumber()
                .await
                .context("[update_from] Error retrieving first_recorded_block")?
                + 1
        }
    })
}

async fn get_last_block(end: Option<i64>) -> Result<i64> {
    Ok(match end {
        Some(s) => s,
        None => {
            let latest_block_hex = endpoints::get_latest_blocknumber()
                .await
                .context("Failed to get latest block number")?;
            type_utils::convert_hex_string_to_i64(&latest_block_hex)
        }
    })
}
