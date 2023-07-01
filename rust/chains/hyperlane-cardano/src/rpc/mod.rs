use cardano_rpc::apis::configuration::Configuration;
use cardano_rpc::apis::default_api::{
    last_finalized_block, merkle_trees_by_block_number, messages_by_block_range,
    LastFinalizedBlockError, MerkleTreesByBlockNumberError, MessagesByBlockRangeError,
};
use cardano_rpc::apis::Error;
use cardano_rpc::models::{
    LastFinalizedBlock200Response, MerkleTreesByBlockNumber200Response,
    MessagesByBlockRange200Response,
};
use serde::{Deserialize, Serialize};

const RPC_URL: &str = "http://localhost:3000";

pub async fn get_finalized_block_number() -> Result<i32, Error<LastFinalizedBlockError>> {
    let configuration = configuration();
    last_finalized_block(&configuration).await.map(|r| {
        r.last_finalized_block
            .expect("API always returns it or fails")
    })
}

pub async fn get_messages_by_block_range(
    from_block: i32,
    to_block: i32,
) -> Result<MessagesByBlockRange200Response, Error<MessagesByBlockRangeError>> {
    let configuration = configuration();
    messages_by_block_range(&configuration, from_block, to_block).await
}

pub async fn get_merkle_trees_at_block_number(
    block_number: i32,
) -> Result<MerkleTreesByBlockNumber200Response, Error<MerkleTreesByBlockNumberError>> {
    let configuration = configuration();
    merkle_trees_by_block_number(&configuration, block_number).await
}

fn configuration() -> Configuration {
    let client = reqwest::Client::builder().build().unwrap();
    Configuration {
        base_path: String::from(RPC_URL),
        client,
        ..Configuration::new().clone()
    }
}
