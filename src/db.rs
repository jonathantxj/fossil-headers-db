use chrono::{TimeZone, Utc};
use sqlx::{postgres::PgPoolOptions, Acquire, Pool, Postgres, QueryBuilder};

use crate::{
    type_utils::{convert_hex_string_to_bytes, convert_hex_string_to_i64, option_fn_handler},
    types::BlockHeaderWithFullTransaction,
};

const DB_CONNECTION_STRING: &str = "DB_CONNECTION_STRING";

pub async fn get_new_postgres_pool(max_connections: u32) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(DB_CONNECTION_STRING)
        .await
}

pub async fn create_tables() -> Result<(), sqlx::Error> {
    const CREATE_BLOCKHEADERS_TABLE: &str = "
        CREATE TABLE IF NOT EXISTS blockheaders (
        author CHAR(42),
        block_hash CHAR(66) PRIMARY KEY,
        number BIGINT,
        parent_hash CHAR(66),
        beneficiary CHAR(42),
        gas_limit BIGINT NOT NULL,
        gas_used BIGINT NOT NULL,
        timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL,
        extra_data BYTEA NOT NULL,
        difficulty VARCHAR(78) NOT NULL,
        mix_hash CHAR(66),
        nonce VARCHAR(78) NOT NULL,
        uncles_hash CHAR(66),
        transaction_root CHAR(66),
        receipts_root CHAR(66),
        state_root CHAR(66),
        base_fee_per_gas VARCHAR(78),
        withdrawals_root CHAR(66),
        parent_beacon_block_root CHAR(66),
        blob_gas_used VARCHAR(78),
        excess_blob_gas VARCHAR(78),
        total_difficulty VARCHAR(78),
        step BIGINT,
        signature BYTEA
        );";

    const CREATE_TRANSACTIONS_TABLE: &str = "
        CREATE TABLE IF NOT EXISTS transactions (
        block_number BIGINT,
        block_hash CHAR(66) REFERENCES blockheaders(block_hash),
        transaction_hash CHAR(66) PRIMARY KEY,
        mint VARCHAR(78),
        source_hash CHAR(66),
        nonce VARCHAR(78) NOT NULL,
        transaction_index INTEGER NOT NULL,
        from_addr CHAR(42),
        to_addr CHAR(42),
        value VARCHAR(78) NOT NULL,
        gas_price VARCHAR(78) NOT NULL,
        max_priority_fee_per_gas VARCHAR(78),
        max_fee_per_gas VARCHAR(78),
        gas VARCHAR(78) NOT NULL,
        input BYTEA,
        chain_id VARCHAR(78),
        type SMALLINT NOT NULL,
        v VARCHAR(78)
        );";
    let pool = get_new_postgres_pool(1).await?;
    let _ = sqlx::query(CREATE_BLOCKHEADERS_TABLE)
        .execute(&pool)
        .await?;
    let _ = sqlx::query(CREATE_TRANSACTIONS_TABLE)
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn get_last_stored_blocknumber() -> Result<(i64,), sqlx::Error> {
    const QUERY_STRING: &str = "SELECT CASE WHEN EXISTS (SELECT 1 FROM blockheaders) THEN (SELECT max(number) from blockheaders) ELSE -1 END;";

    let pool = get_new_postgres_pool(1).await?;
    sqlx::query_as(QUERY_STRING).fetch_one(&pool).await
}

pub async fn find_first_gap(start: i64, end: i64) -> Result<Option<(i64,)>, sqlx::Error> {
    const QUERY_STRING: &str = "SELECT s.i
        FROM generate_series($1, $2) s(i)
        WHERE NOT EXISTS (SELECT 1 FROM blockheaders WHERE number = s.i);";

    let pool = get_new_postgres_pool(1).await?;
    sqlx::query_as(QUERY_STRING)
        .bind(start)
        .bind(end)
        .fetch_optional(&pool)
        .await
}

pub async fn write_blockheader(
    max_connections: u32,
    bh: BlockHeaderWithFullTransaction,
) -> Result<(), sqlx::Error> {
    const INSERT_HEADER_STRING: &str = "INSERT INTO blockheaders (author, block_hash, number,
         parent_hash, beneficiary, gas_limit, gas_used, timestamp, extra_data, difficulty,
         mix_hash, nonce, uncles_hash, transaction_root, receipts_root, state_root,
         base_fee_per_gas, withdrawals_root, parent_beacon_block_root, blob_gas_used,
         excess_blob_gas, total_difficulty, step, signature)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
         $19, $20, $21, $22, $23, $24)
    ON CONFLICT DO NOTHING ";
    let pool = get_new_postgres_pool(max_connections).await?;
    let mut conn = pool.acquire().await?;
    let mut tx = conn.begin().await?;

    let _ = sqlx::query(INSERT_HEADER_STRING)
        .bind(bh.author.clone())
        .bind(bh.hash.clone())
        .bind(convert_hex_string_to_i64(&bh.number))
        .bind(bh.parent_hash)
        .bind(bh.author.clone())
        .bind(convert_hex_string_to_i64(&bh.gas_limit))
        .bind(convert_hex_string_to_i64(&bh.gas_used))
        .bind(
            Utc.timestamp_opt(convert_hex_string_to_i64(&bh.timestamp) as i64, 0)
                .unwrap(),
        )
        .bind(convert_hex_string_to_bytes(&bh.extra_data))
        .bind(convert_hex_string_to_i64(&bh.difficulty))
        .bind(bh.mix_hash)
        .bind(bh.nonce)
        .bind(bh.sha3_uncles)
        .bind(bh.transactions_root)
        .bind(bh.receipts_root)
        .bind(bh.state_root)
        .bind(option_fn_handler(
            convert_hex_string_to_i64,
            bh.base_fee_per_gas,
        ))
        .bind(bh.withdrawals_root)
        .bind(bh.parent_beacon_block_root)
        .bind(bh.blob_gas_used)
        .bind(bh.excess_blob_gas)
        .bind(bh.total_difficulty)
        .bind(option_fn_handler(convert_hex_string_to_i64, bh.step))
        .bind(option_fn_handler(convert_hex_string_to_bytes, bh.signature))
        .execute(&mut *tx)
        .await;

    if bh.transactions.len() == 0 {
        return tx.commit().await;
    }

    let mut transactions_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO transactions (block_number, block_hash,
            transaction_hash, mint, source_hash, nonce, transaction_index, from_addr, to_addr, value,
            gas_price, max_priority_fee_per_gas, max_fee_per_gas, gas, input, chain_id, type, v) "
    );

    transactions_query_builder.push_values(bh.transactions, |mut b, tn| {
        // If you wanted to bind these by-reference instead of by-value,
        // you'd need an iterator that yields references that live as long as `query_builder`,
        // e.g. collect it to a `Vec` first.
        b.push_bind(convert_hex_string_to_i64(&tn.block_number))
            .push_bind(tn.block_hash)
            .push_bind(tn.hash)
            .push_bind(tn.mint)
            .push_bind(tn.source_hash)
            .push_bind(tn.nonce)
            .push_bind(convert_hex_string_to_i64(&tn.transaction_index))
            .push_bind(tn.from)
            .push_bind(tn.to)
            .push_bind(tn.value)
            .push_bind(tn.gas_price)
            .push_bind(tn.max_priority_fee_per_gas)
            .push_bind(tn.max_fee_per_gas)
            .push_bind(tn.gas)
            .push_bind(convert_hex_string_to_bytes(&tn.input))
            .push_bind(tn.chain_id)
            .push_bind(convert_hex_string_to_i64(&tn.r#type))
            .push_bind(tn.v);
    });
    let _ = transactions_query_builder.build().execute(&mut *tx).await?;
    tx.commit().await
}
