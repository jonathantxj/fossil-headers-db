use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub nonce: String,
    #[serde(rename(deserialize = "blockHash"))]
    pub block_hash: String,
    #[serde(rename(deserialize = "blockNumber"))]
    pub block_number: String,
    #[serde(rename(deserialize = "transactionIndex"))]
    pub transaction_index: String,
    pub value: String, // beneficiary
    #[serde(rename(deserialize = "gasPrice"))]
    pub gas_price: String,
    pub gas: String,
    pub input: String,
    pub r#type: String,
    pub v: String,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(rename(deserialize = "maxPriorityFeePerGas"))]
    pub max_priority_fee_per_gas: Option<String>,
    #[serde(rename(deserialize = "maxFeePerGas"))]
    pub max_fee_per_gas: Option<String>,
    #[serde(rename(deserialize = "chainId"))]
    pub chain_id: Option<String>,
    pub mint: Option<String>,
    #[serde(rename(deserialize = "sourceHash"))]
    pub source_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockHeaderWithFullTransaction {
    pub author: String,
    pub difficulty: String,
    #[serde(rename(deserialize = "extraData"))]
    pub extra_data: String,
    #[serde(rename(deserialize = "gasLimit"))]
    pub gas_limit: String,
    #[serde(rename(deserialize = "gasUsed"))]
    pub gas_used: String,
    pub hash: String,
    pub miner: String, // beneficiary
    #[serde(rename(deserialize = "mixHash"))]
    pub mix_hash: Option<String>,
    pub nonce: Option<String>,
    pub number: String,
    #[serde(rename(deserialize = "parentHash"))]
    pub parent_hash: String,
    #[serde(rename(deserialize = "receiptsRoot"))]
    pub receipts_root: String,
    #[serde(rename(deserialize = "sha3Uncles"))]
    pub sha3_uncles: String,
    #[serde(rename(deserialize = "stateRoot"))]
    pub state_root: String,
    #[serde(rename(deserialize = "totalDifficulty"))]
    pub total_difficulty: String,
    pub timestamp: String,
    #[serde(rename(deserialize = "transactionsRoot"))]
    pub transactions_root: String,
    pub transactions: Vec<Transaction>,
    #[serde(rename(deserialize = "baseFeePerGas"))]
    pub base_fee_per_gas: Option<String>,
    #[serde(rename(deserialize = "withdrawalsRoot"))]
    pub withdrawals_root: Option<String>,
    #[serde(rename(deserialize = "blobGasUsed"))]
    pub blob_gas_used: Option<String>,
    #[serde(rename(deserialize = "excessBlobGas"))]
    pub excess_blob_gas: Option<String>,
    #[serde(rename(deserialize = "parentBeaconBlockRoot"))]
    pub parent_beacon_block_root: Option<String>,
    pub step: Option<String>,
    pub signature: Option<String>,
}
