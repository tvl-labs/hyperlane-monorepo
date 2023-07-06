use crate::rpc::OutboxRpc;
use crate::{CardanoMailbox, ConnectionConf};
use async_trait::async_trait;
use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneMessage, IndexRange, Indexer, LogMeta, Mailbox,
    MessageIndexer, H256,
};

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
        todo!() // TODO[cardano]
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
