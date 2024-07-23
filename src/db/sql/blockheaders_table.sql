
CREATE TABLE IF NOT EXISTS blockheaders (
    block_hash CHAR(66) PRIMARY KEY,
    number BIGINT UNIQUE,
    gas_limit BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    nonce VARCHAR(78) NOT NULL,
    transaction_root CHAR(66),
    receipts_root CHAR(66),
    state_root CHAR(66)
    );
