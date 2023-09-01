use std::fmt::Debug;

use async_trait::async_trait;
use derive_more::{AsRef, Deref};
use derive_new::new;

use eyre::{Context, Result};
use hyperlane_base::MultisigCheckpointSyncer;
use hyperlane_core::{HyperlaneMessage, KnownHyperlaneDomain, H256};
use tracing::warn;

use crate::msg::metadata::BaseMetadataBuilder;

use super::base::{MetadataToken, MultisigIsmMetadataBuilder, MultisigMetadata};

#[derive(Debug, Clone, Deref, new, AsRef)]
pub struct MessageIdMultisigMetadataBuilder(BaseMetadataBuilder);

#[async_trait]
impl MultisigIsmMetadataBuilder for MessageIdMultisigMetadataBuilder {
    fn token_layout(&self) -> Vec<MetadataToken> {
        vec![
            MetadataToken::CheckpointMailbox,
            MetadataToken::CheckpointRoot,
            MetadataToken::Signatures,
        ]
    }

    async fn fetch_metadata(
        &self,
        validators: &[H256],
        threshold: u8,
        message: &HyperlaneMessage,
        checkpoint_syncer: &MultisigCheckpointSyncer,
    ) -> Result<Option<MultisigMetadata>> {
        const CTX: &str = "When fetching MessageIdMultisig metadata";
        match KnownHyperlaneDomain::try_from(message.destination) {
            Ok(KnownHyperlaneDomain::CardanoTest1) => {
                // TODO[cardano]: Deduplicate code between keccak checkpoint & blake2b checkpoint types
                let Some(quorum_checkpoint) = checkpoint_syncer
                .fetch_checkpoint_blake2b(validators, threshold as usize, message.nonce)
                .await
                .context(CTX)?
            else {
                return Ok(None);
            };

                if quorum_checkpoint.checkpoint.message_id != message.id() {
                    warn!(
                        "Quorum checkpoint message id {} does not match message id {}",
                        quorum_checkpoint.checkpoint.message_id,
                        message.id()
                    );
                    return Ok(None);
                }

                Ok(Some(MultisigMetadata::new(
                    quorum_checkpoint.checkpoint.checkpoint.checkpoint,
                    quorum_checkpoint.signatures,
                    None,
                    None,
                )))
            }
            _ => {
                let Some(quorum_checkpoint) = checkpoint_syncer
            .fetch_checkpoint(validators, threshold as usize, message.nonce)
            .await
            .context(CTX)?
        else {
            return Ok(None);
        };

                if quorum_checkpoint.checkpoint.message_id != message.id() {
                    warn!(
                        "Quorum checkpoint message id {} does not match message id {}",
                        quorum_checkpoint.checkpoint.message_id,
                        message.id()
                    );
                    return Ok(None);
                }

                Ok(Some(MultisigMetadata::new(
                    quorum_checkpoint.checkpoint.checkpoint,
                    quorum_checkpoint.signatures,
                    None,
                    None,
                )))
            }
        }
    }
}
