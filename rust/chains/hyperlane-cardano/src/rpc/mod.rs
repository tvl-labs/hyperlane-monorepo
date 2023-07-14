use hex::ToHex;
use hyperlane_cardano_rpc_rust_client::apis::configuration::Configuration;
use hyperlane_cardano_rpc_rust_client::apis::default_api::{
    get_validator_storage_locations, last_finalized_block, merkle_tree, messages_by_block_range,
    GetValidatorStorageLocationsError, LastFinalizedBlockError, MerkleTreeError,
    MessagesByBlockRangeError,
};
use hyperlane_cardano_rpc_rust_client::apis::Error;
use hyperlane_cardano_rpc_rust_client::models::{
    GetValidatorStorageLocations200Response, GetValidatorStorageLocationsRequest,
    MerkleTree200Response, MessagesByBlockRange200Response,
};
use url::Url;

use hyperlane_core::H256;

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

    pub async fn get_validator_storage_locations(
        &self,
        validator_addresses: &[H256],
    ) -> Result<Vec<Vec<String>>, Error<GetValidatorStorageLocationsError>> {
        let validator_addresses: Vec<String> = validator_addresses
            .iter()
            .map(|v| format!("0x{}", v.encode_hex::<String>()))
            .collect();
        let validator_storage_locations = get_validator_storage_locations(
            &self.0,
            GetValidatorStorageLocationsRequest {
                validator_addresses,
            },
        )
        .await?;
        let result = validator_storage_locations
            .validator_storage_locations
            .iter()
            .map(|vs| vec![String::from(&vs.storage_location)])
            .collect();
        Ok(result)
    }
}
