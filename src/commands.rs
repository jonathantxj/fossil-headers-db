use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::{db, endpoints, type_utils};

const MAX_RETRIES: u64 = 5;
const PARALLEL_REQUESTS: usize = 100;

pub async fn fill_gaps(start: Option<i64>, end: Option<i64>, should_terminate: Arc<Mutex<bool>>) {
    let _ = db::create_tables().await.unwrap();

    let mut range_start_pointer: i64 = if start.is_some_and(|x| x > 0) {
        start.unwrap()
    } else {
        0
    };

    let range_end: i64 = match end {
        Some(s) => s,
        _ => {
            db::get_last_stored_blocknumber()
                .await
                .expect("[main: fill_gaps] error retrieving last_recorded_block")
                .0
        }
    };

    if range_end < 0 || range_start_pointer == range_end {
        println!("empty database");
        return;
    }

    loop {
        let terminate = should_terminate.lock().unwrap();
        if *terminate {
            return;
        }
        drop(terminate);
        let missing_block: Result<Option<(i64,)>, sqlx::Error> =
            db::find_first_gap(range_start_pointer, range_end).await;
        match missing_block {
            Ok(res) => match res {
                Some(s) => {
                    let block_number = s.0;
                    println!(
                        "[main: fill_gaps] found missing block number: {}",
                        block_number
                    );
                    let mut res = false;
                    for i in 0..MAX_RETRIES {
                        if res {
                            break;
                        }
                        let block = endpoints::get_full_block_by_number(block_number).await;
                        if block.is_err() {
                            println!("[main: fill_gaps] error retrieving block {}", block_number);
                            println!("{:#?}", block.unwrap_err());
                            continue;
                        }
                        let write_res = db::write_blockheader(1, block.unwrap().result).await;
                        if write_res.is_err() {
                            println!("[main: fill_gaps] error writing block {}", block_number);
                            println!("{}", write_res.unwrap_err());
                            continue;
                        }
                        range_start_pointer = block_number + 1;
                        res = true;
                        println!(
                            "[main: fill_gaps] successfully writing block {} after {} retries",
                            block_number, i
                        );
                    }
                    if !res {
                        println!("[main: fill_gaps] Error with block number {}", block_number);
                        return;
                    }
                }
                _ => {
                    println!(
                        "[main: fill_gaps] no missing values found from {} to {}",
                        range_start_pointer, range_end
                    );
                    return;
                }
            },
            Err(ref e) => {
                print!("{}", e.to_string())
            }
        }
    }
}

pub async fn update_from(start: Option<i64>, end: Option<i64>, should_terminate: Arc<Mutex<bool>>) {
    let _ = db::create_tables().await.unwrap();
    let first_missing_block: i64 = match start {
        Some(s) => s,
        _ => {
            db::get_last_stored_blocknumber()
                .await
                .expect("[main: update_from] error retrieving first_recorded_block")
                .0
                + 1
        }
    };
    println!("first_missing_block: {}", first_missing_block);

    let last_block: i64 = match end {
        Some(s) => s,
        _ => {
            let latest_block_hex: String =
                endpoints::get_latest_blocknumber().await.unwrap().result;
            type_utils::convert_hex_string_to_i64(&latest_block_hex)
        }
    };
    println!("range end: {}", last_block);

    let time_started: Instant = Instant::now();
    for n in
        (first_missing_block..=last_block - PARALLEL_REQUESTS as i64).step_by(PARALLEL_REQUESTS)
    {
        let terminate = should_terminate.lock().unwrap();
        if *terminate {
            drop(terminate);
            break;
        }
        drop(terminate);

        let range_end: i64 = if last_block < n + PARALLEL_REQUESTS as i64 {
            last_block
        } else {
            n + PARALLEL_REQUESTS as i64
        };

        let mut tasks = vec![];
        for block_number in n..range_end {
            tasks.push(tokio::task::spawn(async move {
                let mut res = false;
                for i in 0..MAX_RETRIES {
                    if res {
                        break;
                    }
                    let block = endpoints::get_full_block_by_number(block_number).await;
                    if block.is_err() {
                        println!(
                            "[main: update_from] error retrieving block {}",
                            block_number
                        );
                        println!("{}", block.unwrap_err());
                        continue;
                    }
                    let write_res =
                        db::write_blockheader(PARALLEL_REQUESTS as u32, block.unwrap().result)
                            .await;
                    if write_res.is_err() {
                        println!("[main: update_from] error writing block {}", block_number);
                        println!("{}", write_res.unwrap_err());
                        continue;
                    }
                    res = true;
                    if i > 0 {
                        println!(
                            "[main: update_from] successfully writing block {} after {} retries",
                            block_number, i
                        );
                    }
                }
                if !res {
                    println!(
                        "[main: update_from] Error with block number {}",
                        block_number
                    );
                    return;
                }
            }));
        }
        let _ = futures_util::future::join_all(tasks).await;
        println!("written blocks {} - {}", n, range_end);
    }
    println!("time elapsed: {:?}", time_started.elapsed());
}
