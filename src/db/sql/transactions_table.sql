CREATE TABLE IF NOT EXISTS transactions (
    block_number BIGINT REFERENCES blockheaders(number),
    transaction_hash CHAR(66) PRIMARY KEY,
    from_addr CHAR(42),
    to_addr CHAR(42),
    value VARCHAR(78) NOT NULL,
    gas_price VARCHAR(78) NOT NULL,
    max_priority_fee_per_gas VARCHAR(78),
    max_fee_per_gas VARCHAR(78),
    transaction_index INTEGER NOT NULL,
    gas VARCHAR(78) NOT NULL,
    chain_id VARCHAR(78)
    );
