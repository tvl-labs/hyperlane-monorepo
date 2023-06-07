use async_trait::async_trait;
use hyperlane_core::{
    BlockInfo, ChainResult, HyperlaneChain, HyperlaneDomain, HyperlaneProvider, TxnInfo, H256,
};

#[derive(Debug)]
pub struct CardanoProvider {
    domain: HyperlaneDomain,
}

impl CardanoProvider {
    pub fn new(domain: HyperlaneDomain) -> Self {
        CardanoProvider { domain }
    }
}

impl HyperlaneChain for CardanoProvider {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(CardanoProvider {
            domain: self.domain.clone(),
        })
    }
}

#[async_trait]
impl HyperlaneProvider for CardanoProvider {
    async fn get_block_by_hash(&self, _hash: &H256) -> ChainResult<BlockInfo> {
        todo!() // TODO[cardano]: only required by 'scraper' agent
    }

    async fn get_txn_by_hash(&self, _hash: &H256) -> ChainResult<TxnInfo> {
        todo!() // TODO[cardano]: only required by 'scraper' agent
    }

    async fn is_contract(&self, _address: &H256) -> ChainResult<bool> {
        Ok(true)
    }
}
