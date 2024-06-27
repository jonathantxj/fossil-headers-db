use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::BlockHeaderWithFullTransaction;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);
static NODE_CONNECTION_STRING: Lazy<String> = Lazy::new(|| {
    std::env::var("NODE_CONNECTION_STRING").expect("NODE_CONNECTION_STRING must be set")
});

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub result: T,
    // pub id: String,
    // pub jsonrpc: String,
}

#[derive(Serialize)]
struct RpcRequest<'a, T> {
    jsonrpc: &'a str,
    id: &'a str,
    method: &'a str,
    params: T,
}

pub async fn get_latest_blocknumber() -> Result<String> {
    let params = RpcRequest {
        jsonrpc: "2.0",
        id: "0",
        method: "eth_blockNumber",
        params: Vec::<String>::new(),
    };

    make_rpc_call::<_, String>(&params)
        .await
        .context("Failed to get latest block number")
}

pub async fn get_full_block_by_number(number: i64) -> Result<BlockHeaderWithFullTransaction> {
    let params = RpcRequest {
        jsonrpc: "2.0",
        id: "0",
        method: "eth_getBlockByNumber",
        params: vec![format!("0x{:x}", number), true.to_string()],
    };

    make_rpc_call::<_, BlockHeaderWithFullTransaction>(&params)
        .await
        .context("Failed to get full block by number")
}

async fn make_rpc_call<T: Serialize, R: for<'de> Deserialize<'de>>(params: &T) -> Result<R> {
    let response = CLIENT
        .post(NODE_CONNECTION_STRING.as_str())
        .json(params)
        .send()
        .await
        .context("Failed to send request")?
        .json::<RpcResponse<R>>()
        .await
        .context("Failed to parse response")?;

    Ok(response.result)
}
