use hex::ToHex;
use hyperlane_cardano_rpc_rust_client::apis::configuration::Configuration;
use hyperlane_cardano_rpc_rust_client::apis::default_api::{
    estimate_inbound_message_fee, get_validator_storage_locations, inbox_ism_parameters,
    is_inbox_message_delivered, last_finalized_block, merkle_tree, messages_by_block_range,
    submit_inbound_message, EstimateInboundMessageFeeError, GetValidatorStorageLocationsError,
    InboxIsmParametersError, IsInboxMessageDeliveredError, LastFinalizedBlockError,
    MerkleTreeError, MessagesByBlockRangeError, SubmitInboundMessageError,
};
use hyperlane_cardano_rpc_rust_client::apis::Error;
use hyperlane_cardano_rpc_rust_client::models::{
    EstimateInboundMessageFee200Response, EstimateInboundMessageFeeRequest,
    EstimateInboundMessageFeeRequestMessage, GetValidatorStorageLocationsRequest,
    InboxIsmParameters200Response, IsInboxMessageDelivered200Response, MerkleTree200Response,
    MessagesByBlockRange200Response, SubmitInboundMessage200Response, SubmitInboundMessageRequest,
};
use url::Url;

use hyperlane_core::{HyperlaneMessage, H256};

pub mod conversion;

#[derive(Debug)]
pub struct CardanoRpc(Configuration);

impl CardanoRpc {
    pub fn new(url: &Url) -> CardanoRpc {
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

    pub async fn get_ism_parameters(
        &self,
    ) -> Result<InboxIsmParameters200Response, Error<InboxIsmParametersError>> {
        inbox_ism_parameters(&self.0).await
    }

    pub async fn is_inbox_message_delivered(
        &self,
        message_id: H256,
    ) -> Result<IsInboxMessageDelivered200Response, Error<IsInboxMessageDeliveredError>> {
        is_inbox_message_delivered(&self.0, message_id.encode_hex::<String>().as_str()).await
    }

    // NOTE: We must mock so much we don't even need a tight/deterministic
    // implementation in the RPC...
    pub async fn estimate_inbox_message_fee(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> Result<EstimateInboundMessageFee200Response, Error<EstimateInboundMessageFeeError>> {
        let origin_mailbox = format!("0x{}", hex::encode(&metadata[0..32]));
        let checkpoint_root = hex::encode(&metadata[32..64]);
        let checkpoint_index = u32::from_be_bytes(metadata[64..68].try_into().unwrap());
        let signatures = metadata[68..metadata.len() - 1]
            .chunks(64)
            .map(|s| hex::encode(s))
            .collect();
        estimate_inbound_message_fee(
            &self.0,
            EstimateInboundMessageFeeRequest {
                relayer_cardano_address:
                    "addr_test1vqvjvk3qezccu5a3gce65mqvg4tpfy47plv68wmh68paswqv3jaqe".to_string(), // TODO: Read from config
                origin: message.origin as f32,
                origin_mailbox,
                checkpoint_root,
                checkpoint_index: checkpoint_index as f32,
                message: Box::new(EstimateInboundMessageFeeRequestMessage {
                    version: message.version as f32,
                    nonce: message.nonce as f32,
                    origin_domain: message.origin as f32,
                    sender: format!("0x{}", message.sender.encode_hex::<String>()),
                    destination_domain: message.destination as f32,
                    recipient: format!("0x{}", message.recipient.encode_hex::<String>()),
                    message: message.body.encode_hex(),
                }),
                //
                signatures,
            },
        )
        .await
    }

    // TODO: Share metadata decoding code with estimating fee
    pub async fn submit_inbox_message(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> Result<SubmitInboundMessage200Response, Error<SubmitInboundMessageError>> {
        let origin_mailbox = format!("0x{}", hex::encode(&metadata[0..32]));
        let checkpoint_root = hex::encode(&metadata[32..64]);
        let checkpoint_index = u32::from_be_bytes(metadata[64..68].try_into().unwrap());
        let signatures = metadata[68..metadata.len() - 1]
            .chunks(64)
            .map(|s| hex::encode(s))
            .collect();
        submit_inbound_message(
            &self.0,
            SubmitInboundMessageRequest {
                // TODO: Read from config
                relayer_cardano_address:
                    "addr_test1vqvjvk3qezccu5a3gce65mqvg4tpfy47plv68wmh68paswqv3jaqe".to_string(),
                // TODO: Read from config
                private_key: "e8e34f6c74e22577d609803dfe9c8773f10e478e7dadf6d065a78ae42a21f912"
                    .to_string(),
                origin: message.origin as f32,
                origin_mailbox,
                checkpoint_root,
                checkpoint_index: checkpoint_index as f32,
                message: Box::new(EstimateInboundMessageFeeRequestMessage {
                    version: message.version as f32,
                    nonce: message.nonce as f32,
                    origin_domain: message.origin as f32,
                    sender: format!("0x{}", message.sender.encode_hex::<String>()),
                    destination_domain: message.destination as f32,
                    recipient: format!("0x{}", message.recipient.encode_hex::<String>()),
                    message: message.body.encode_hex(),
                }),
                //
                signatures,
            },
        )
        .await
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
