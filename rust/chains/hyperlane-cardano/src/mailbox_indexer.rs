use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneMessage, IndexRange, Indexer, LogMeta, MessageIndexer,
    H256,
};

#[derive(Debug)]
pub struct CardanoMailboxIndexer {}

impl CardanoMailboxIndexer {
    pub fn new(conf: &ConnectionConf, locator: ContractLocator) -> ChainResult<Self> {
        Ok(Self {})
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        todo!() // TODO[cardano]
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
        // TODO[cardano]
        Ok((0, 0))
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
