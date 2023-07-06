use crate::rpc::conversion::FromRpc;
use crate::rpc::OutboxRpc;
use crate::{CardanoMailbox, ConnectionConf};
use async_trait::async_trait;
use hex::FromHex;
use hyperlane_core::{
    ChainCommunicationError, ChainResult, ContractLocator, HyperlaneMessage, IndexRange, Indexer,
    LogMeta, Mailbox, MessageIndexer, H256, U256,
};
use std::str::FromStr;

#[derive(Debug)]
pub struct CardanoMailboxIndexer {
    outbox_rpc: OutboxRpc,
    mailbox: CardanoMailbox,
}

impl CardanoMailboxIndexer {
    pub fn new(conf: &ConnectionConf, locator: ContractLocator) -> ChainResult<Self> {
        let outbox_rpc = OutboxRpc::new(&conf.url);
        let mailbox = CardanoMailbox::new(conf, locator, None)?;
        Ok(Self {
            outbox_rpc,
            mailbox,
        })
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.mailbox.finalized_block_number().await
    }
}

#[async_trait]
impl Indexer<HyperlaneMessage> for CardanoMailboxIndexer {
    async fn fetch_logs(&self, range: IndexRange) -> ChainResult<Vec<(HyperlaneMessage, LogMeta)>> {
        let (from, to) = match range {
            IndexRange::Blocks(from, to) => (from, to),
            IndexRange::Sequences(from, to) => {
                return Err(ChainCommunicationError::from_other_str(
                    "Cardano does not support message-nonce based indexing",
                ))
            }
        };

        tracing::info!(
            "Fetching Cardano HyperlaneMessage logs from {} to {}",
            from,
            to
        );

        let response = self
            .outbox_rpc
            .get_messages_by_block_range(from, to)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        let vec = response.messages;
        Ok(vec
            .into_iter()
            .map(|m| {
                (
                    HyperlaneMessage::from_rpc(m.message.as_ref()),
                    LogMeta {
                        address: self.mailbox.outbox,
                        block_number: m.block as u64,
                        // TODO[cardano]: do we need real values?
                        block_hash: H256::zero(),
                        transaction_hash: H256::zero(),
                        transaction_index: 0,
                        log_index: U256::zero(),
                    },
                )
            })
            .collect())
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.get_finalized_block_number().await
    }
}

#[async_trait]
impl MessageIndexer for CardanoMailboxIndexer {
    async fn fetch_count_at_tip(&self) -> ChainResult<(u32, u32)> {
        self.mailbox
            .tree_and_tip(None)
            .await
            .map(|(tree, tip)| (tree.count() as u32, tip))
    }
}

// TODO[cardano]: only used by 'scraper' agent
#[async_trait]
impl Indexer<H256> for CardanoMailboxIndexer {
    async fn fetch_logs(&self, _range: IndexRange) -> ChainResult<Vec<(H256, LogMeta)>> {
        todo!() // TODO[cardano]
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.get_finalized_block_number().await
    }
}
