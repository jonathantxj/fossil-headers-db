use crate::types::BlockHeaderWithFullTransaction;
use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use log::{info, warn};
use sqlx::QueryBuilder;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<Arc<Pool<Postgres>>> = OnceCell::const_new();

pub async fn get_db_pool() -> Result<Arc<Pool<Postgres>>> {
    match DB_POOL.get() {
        Some(pool) => Ok(pool.clone()),
        None => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(
                    &std::env::var("DB_CONNECTION_STRING")
                        .expect("DB_CONNECTION_STRING must be set"),
                )
                .await
                .context("Failed to create database pool")?;
            let arc_pool = Arc::new(pool);
            match DB_POOL.set(arc_pool.clone()) {
                Ok(_) => Ok(arc_pool),
                Err(_) => Ok(DB_POOL.get().expect("Pool was just set").clone()),
            }
        }
    }
}

pub async fn create_tables() -> Result<()> {
    let pool = get_db_pool().await.context("Failed to get database pool")?;
    sqlx::query(include_str!("./sql/blockheaders_table.sql"))
        .execute(&*pool)
        .await
        .context("Failed to create blockheaders table")?;
    sqlx::query(include_str!("./sql/transactions_table.sql"))
        .execute(&*pool)
        .await
        .context("Failed to create transactions table")?;
    Ok(())
}

pub async fn get_last_stored_blocknumber() -> Result<i64> {
    let pool = get_db_pool().await.context("Failed to get database pool")?;
    let result: (i64,) = sqlx::query_as("SELECT COALESCE(MAX(number), -1) FROM blockheaders")
        .fetch_one(&*pool)
        .await
        .context("Failed to get last stored block number")?;

    Ok(result.0)
}

pub async fn find_first_gap(start: i64, end: i64) -> Result<Option<i64>> {
    let pool = get_db_pool().await.context("Failed to get database pool")?;
    let result: Option<(i64,)> = sqlx::query_as(
        r#"
        WITH RECURSIVE number_series(n) AS (
            SELECT $1
            UNION ALL
            SELECT n + 1 FROM number_series WHERE n < $2
        )
        SELECT n FROM number_series
        WHERE n NOT IN (SELECT number FROM blockheaders WHERE number BETWEEN $1 AND $2)
        LIMIT 1
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_optional(&*pool)
    .await
    .context("Failed to find first gap")?;

    Ok(result.map(|r| r.0))
}

pub async fn write_blockheader(block_header: BlockHeaderWithFullTransaction) -> Result<()> {
    let pool = get_db_pool().await.context("Failed to get database pool")?;
    let mut tx = pool.begin().await.context("Failed to start transaction")?;

    // Insert block header
    let result = sqlx::query(
        r#"
        INSERT INTO blockheaders (
            author, block_hash, number, parent_hash, beneficiary, gas_limit, gas_used, 
            timestamp, extra_data, difficulty, mix_hash, nonce, uncles_hash, 
            transaction_root, receipts_root, state_root, base_fee_per_gas, 
            withdrawals_root, parent_beacon_block_root, blob_gas_used, 
            excess_blob_gas, total_difficulty, step, signature
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24)
        ON CONFLICT (block_hash) DO NOTHING
        "#,
    )
    .bind(&block_header.author)
    .bind(&block_header.hash)
    .bind(convert_hex_string_to_i64(&block_header.number))
    .bind(&block_header.parent_hash)
    .bind(&block_header.author)
    .bind(convert_hex_string_to_i64(&block_header.gas_limit))
    .bind(convert_hex_string_to_i64(&block_header.gas_used))
    .bind(
        Utc.timestamp_opt(convert_hex_string_to_i64(&block_header.timestamp), 0)
            .single()
            .context("Invalid timestamp")?,
    )
    .bind(convert_hex_string_to_bytes(&block_header.extra_data))
    .bind(convert_hex_string_to_i64(&block_header.difficulty))
    .bind(&block_header.mix_hash)
    .bind(&block_header.nonce)
    .bind(&block_header.sha3_uncles)
    .bind(&block_header.transactions_root)
    .bind(&block_header.receipts_root)
    .bind(&block_header.state_root)
    .bind(option_fn_handler(
        convert_hex_string_to_i64,
        block_header.base_fee_per_gas.as_deref(),
    ))
    .bind(&block_header.withdrawals_root)
    .bind(&block_header.parent_beacon_block_root)
    .bind(&block_header.blob_gas_used)
    .bind(&block_header.excess_blob_gas)
    .bind(&block_header.total_difficulty)
    .bind(option_fn_handler(
        convert_hex_string_to_i64,
        block_header.step.as_deref(),
    ))
    .bind(option_fn_handler(
        convert_hex_string_to_bytes,
        block_header.signature.as_deref(),
    ))
    .execute(&mut *tx) // Changed this line
    .await
    .context("Failed to insert block header")?;

    if result.rows_affected() == 0 {
        warn!("Block header already exists: {}", block_header.hash);
    } else {
        info!("Inserted block header: {}", block_header.hash);
    }

    // Insert transactions
    if !block_header.transactions.is_empty() {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO transactions (
                block_number, block_hash, transaction_hash, mint, source_hash, nonce, 
                transaction_index, from_addr, to_addr, value, gas_price, max_priority_fee_per_gas, 
                max_fee_per_gas, gas, input, chain_id, type, v
            ) ",
        );

        query_builder.push_values(block_header.transactions.iter(), |mut b, tx| {
            b.push_bind(convert_hex_string_to_i64(&tx.block_number))
                .push_bind(&tx.block_hash)
                .push_bind(&tx.hash)
                .push_bind(&tx.mint)
                .push_bind(&tx.source_hash)
                .push_bind(&tx.nonce)
                .push_bind(convert_hex_string_to_i64(&tx.transaction_index))
                .push_bind(&tx.from)
                .push_bind(&tx.to)
                .push_bind(&tx.value)
                .push_bind(&tx.gas_price)
                .push_bind(&tx.max_priority_fee_per_gas)
                .push_bind(&tx.max_fee_per_gas)
                .push_bind(&tx.gas)
                .push_bind(convert_hex_string_to_bytes(&tx.input))
                .push_bind(&tx.chain_id)
                .push_bind(convert_hex_string_to_i64(&tx.r#type))
                .push_bind(&tx.v);
        });

        query_builder.push(" ON CONFLICT (transaction_hash) DO NOTHING");

        let query = query_builder.build();
        let result = query
            .execute(&mut *tx)
            .await
            .context("Failed to insert transactions")?; // Changed this line

        info!(
            "Inserted {} transactions for block {}",
            result.rows_affected(),
            block_header.hash
        );
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

// Helper functions
fn convert_hex_string_to_i64(hex_str: &str) -> i64 {
    i64::from_str_radix(hex_str.trim_start_matches("0x"), 16).expect("Invalid hex string")
}

fn convert_hex_string_to_bytes(hex_str: &str) -> Vec<u8> {
    hex::decode(hex_str.trim_start_matches("0x")).expect("Invalid hex string")
}

fn option_fn_handler<F, T>(f: F, option: Option<&str>) -> Option<T>
where
    F: Fn(&str) -> T,
{
    option.map(f)
}
