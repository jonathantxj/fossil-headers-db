use serde::Deserialize;
use serde_json::json;

use crate::types::BlockHeaderWithFullTransaction;

const NODE_CONNECTION_STRING: &str = "NODE_CONNECTION_STRING";

#[derive(Deserialize)]
pub struct GetBlockNumberRpcResponse {
    pub result: String,
}

pub async fn get_latest_blocknumber() -> Result<GetBlockNumberRpcResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let params = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "eth_blockNumber",
        "params": []
    });
    client
        .post(NODE_CONNECTION_STRING)
        .json(&params)
        .send()
        .await
        .unwrap()
        .json::<GetBlockNumberRpcResponse>()
        .await
}

#[derive(Debug, Deserialize)]
pub struct GetBlockDataRpcResponse {
    pub result: BlockHeaderWithFullTransaction,
}

pub async fn get_full_block_by_number(
    number: i64,
) -> Result<GetBlockDataRpcResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let params = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "eth_getBlockByNumber",
        "params": [number, true]
    });

    client
        .post(NODE_CONNECTION_STRING)
        .json(&params)
        .send()
        .await
        .unwrap()
        .json::<GetBlockDataRpcResponse>()
        .await
}
