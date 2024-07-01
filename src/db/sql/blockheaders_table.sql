
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
);

