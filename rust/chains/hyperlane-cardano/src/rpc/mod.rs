use hyperlane_cardano_rpc_rust_client::apis::configuration::Configuration;
use hyperlane_cardano_rpc_rust_client::apis::default_api::{
    last_finalized_block, merkle_tree, messages_by_block_range, LastFinalizedBlockError,
    MerkleTreeError, MessagesByBlockRangeError,
};
use hyperlane_cardano_rpc_rust_client::apis::Error;
use hyperlane_cardano_rpc_rust_client::models::{
    MerkleTree200Response, MessagesByBlockRange200Response,
};
use url::Url;

pub mod conversion;

#[derive(Debug)]
pub struct OutboxRpc(Configuration);

impl OutboxRpc {
    pub fn new(url: &Url) -> OutboxRpc {
        let client = reqwest::Client::builder().build().unwrap();
        Self(Configuration {
            base_path: url.to_string().trim_end_matches("/").to_string(),
            client,
            ..Configuration::new().clone()
        })
    }

    pub async fn get_finalized_block_number(&self) -> Result<u32, Error<LastFinalizedBlockError>> {
        last_finalized_block(&self.0).await.map(|r| {
            r.last_finalized_block
                // TODO[cardano]: make the 'last_finalized_block' non-zero in OpenAPI.
                .expect("API always returns it or fails") as u32
        })
    }

    pub async fn get_messages_by_block_range(
        &self,
        from_block: u32,
        to_block: u32,
    ) -> Result<MessagesByBlockRange200Response, Error<MessagesByBlockRangeError>> {
        messages_by_block_range(&self.0, from_block as i32, to_block as i32).await
    }

    pub async fn get_latest_merkle_tree(
        &self,
    ) -> Result<MerkleTree200Response, Error<MerkleTreeError>> {
        merkle_tree(&self.0).await
    }
}
