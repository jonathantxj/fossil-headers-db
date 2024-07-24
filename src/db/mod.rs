use crate::types::type_utils::convert_hex_string_to_i64;
use crate::types::type_utils::convert_i64_to_hex_string;
use crate::types::BlockDetails;
use crate::types::BlockHeaderWithFullTransaction;
use crate::types::Transaction;
use crate::types::TxHash;
use anyhow::{Context, Result};
use log::{info, warn};
use sqlx::postgres::PgConnectOptions;
use sqlx::ConnectOptions;
use sqlx::QueryBuilder;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<Arc<Pool<Postgres>>> = OnceCell::const_new();
pub const DB_MAX_CONNECTIONS: u32 = 1000;

pub async fn get_db_pool() -> Result<Arc<Pool<Postgres>>> {
    match DB_POOL.get() {
        Some(pool) => Ok(pool.clone()),
        None => {
            let mut conn_options: PgConnectOptions = dotenvy::var("DB_CONNECTION_STRING")
                .expect("DB_CONNECTION_STRING must be set")
                .parse()?;
            conn_options =
                conn_options.log_slow_statements(log::LevelFilter::Debug, Duration::new(120, 0));

            let pool = PgPoolOptions::new()
                .max_connections(DB_MAX_CONNECTIONS)
                .connect_with(conn_options)
                .await?;
            let arc_pool = Arc::new(pool);
            match DB_POOL.set(arc_pool.clone()) {
                Ok(_) => Ok(arc_pool),
                Err(_) => Ok(DB_POOL.get().expect("Pool was just set").clone()),
            }
        }
    }
}

pub async fn create_tables() -> Result<()> {
    let pool = get_db_pool().await?;
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

pub async fn get_hashes(start: i64, end: i64) -> Result<Vec<TxHash>> {
    let pool = get_db_pool().await?;
    let result: Vec<TxHash> = sqlx::query_as(
        r#"
        SELECT block_number, transaction_hash FROM transactions
            WHERE block_number BETWEEN $1 AND $2
            AND gas = gas_price
            LIMIT 100
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(&*pool)
    .await
    .context("Failed to get hashes")?;

    Ok(result)
}

pub async fn migrate_transactions(should_terminate: &Arc<AtomicBool>) -> Result<()> {
    let pool = get_db_pool().await?;
    // sqlx::query(include_str!("./sql/migration/copy_deleted_record.sql"))
    //     .execute(&*pool)
    //     .await?;
    
    // sqlx::query(include_str!("./sql/migration/delete_trigger.sql"))
    //     .execute(&*pool)
    //     .await?;

    for i in 0..4095 {
        if should_terminate.load(Ordering::Relaxed) {
            info!("Termination requested. Stopping update process at {i}");
            break;
        }
        
        let prefix: String = {
            let hex = convert_i64_to_hex_string(i);
            if hex.len() < 5 {
                hex + "0%"
            } else {
                hex + "%"
            }
        };

        loop {
            if should_terminate.load(Ordering::Relaxed) {
                break;
            }
            let res = sqlx::query(
                r#"DELETE FROM transactions
                WHERE transaction_hash LIKE $1
                ;"#
            )
            .bind(prefix.clone())
            .execute(&*pool)
            .await?;
            let rows = res.rows_affected();
            if rows == 0 {
                info!("Done migrating prefix: {prefix}");
            } else {
                info!("Migrated {rows} rows with prefix: {prefix}")
            }
        }
    }

    Ok(())
}

pub async fn fix_gas(transaction: Transaction) -> Result<()> {
    let pool = get_db_pool().await?;
    let res = sqlx::query(
        r#"
        UPDATE transactions
        SET gas = $1, transaction_index = $2
        WHERE transaction_hash = $3 AND block_number = $4
        "#,
    )
    .bind(transaction.gas)
    .bind(transaction.transaction_index)
    .bind(&transaction.hash)
    .bind(&transaction.block_number)
    .execute(&*pool)
    .await?;
    if res.rows_affected() > 0 {
        info!(
            "Fixed gas for record {} - {}",
            &transaction.block_number, &transaction.hash
        );
    }

    Ok(())
}

pub async fn write_blockheader(block_header: BlockHeaderWithFullTransaction) -> Result<()> {
    let pool = get_db_pool().await?;
    let mut tx = pool.begin().await?;

    // Insert block header
    let result = sqlx::query(
        r#"
        INSERT INTO blockheaders2 (
            block_hash, number, gas_limit, gas_used, nonce,
            transaction_root, receipts_root, state_root
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (number) DO NOTHING
        "#,
    )
    .bind(&block_header.hash)
    .bind(convert_hex_string_to_i64(&block_header.number))
    .bind(convert_hex_string_to_i64(&block_header.gas_limit))
    .bind(convert_hex_string_to_i64(&block_header.gas_used))
    .bind(&block_header.nonce)
    .bind(&block_header.transactions_root)
    .bind(&block_header.receipts_root)
    .bind(&block_header.state_root)
    .execute(&mut *tx) // Changed this line
    .await?;

    if result.rows_affected() == 0 {
        // warn!(
        //     "Block already exists: -- block number: {}, block hash: {}",
        //     block_header.number, block_header.hash
        // );
        // return Ok(());
    } else {
        info!(
            "Inserted block number: {}, block hash: {}",
            block_header.number, block_header.hash
        );
    }

    // Insert transactions
    if !block_header.transactions.is_empty() {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO transactions2 (
                block_number, transaction_hash,
                transaction_index, from_addr, to_addr, value, gas_price, max_priority_fee_per_gas, 
                max_fee_per_gas, gas, chain_id
            ) ",
        );

        query_builder.push_values(block_header.transactions.iter(), |mut b, tx| {
            b.push_bind(convert_hex_string_to_i64(&tx.block_number))
                .push_bind(&tx.hash)
                .push_bind(convert_hex_string_to_i64(&tx.transaction_index))
                .push_bind(&tx.from)
                .push_bind(&tx.to)
                .push_bind(&tx.value)
                .push_bind(&tx.gas_price)
                .push_bind(&tx.max_priority_fee_per_gas)
                .push_bind(&tx.max_fee_per_gas)
                .push_bind(&tx.gas)
                .push_bind(&tx.chain_id);
        });

        query_builder.push(
        "ON CONFLICT (transaction_hash) DO UPDATE SET 
            transaction_index = EXCLUDED.transaction_index,
            gas = EXCLUDED.gas"
        );

        let query = query_builder.build();
        let result = query
            .execute(&mut *tx)
            .await?;

        info!(
            "Inserted {} transactions for block {}",
            result.rows_affected(),
            block_header.hash
        );
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

pub async fn get_blockheaders(start_blocknumber: i64) -> Result<Vec<BlockDetails>> {
    let pool = get_db_pool().await?;
    let result: Vec<BlockDetails> = sqlx::query_as(
        r#"
        SELECT block_hash, number FROM blockheaders
            WHERE number >= $1
            ORDER BY number ASC
            LIMIT 1
        "#,
    )
    .bind(start_blocknumber)
    .fetch_all(&*pool)
    .await
    .context("Failed to get blockheaders")?;

    Ok(result)
}
