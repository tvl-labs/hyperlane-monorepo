use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::{
    ChainResult, ContractLocator, IndexRange, Indexer, InterchainGasPayment, LogMeta,
};
use tracing::instrument;

#[derive(Debug)]
pub struct CardanoInterchainGasPaymasterIndexer {}

impl CardanoInterchainGasPaymasterIndexer {
    pub fn new(_conf: &ConnectionConf, _locator: ContractLocator) -> Self {
        Self {}
    }
}

#[async_trait]
impl Indexer<InterchainGasPayment> for CardanoInterchainGasPaymasterIndexer {
    #[instrument(err, skip(self))]
    async fn fetch_logs(
        &self,
        _range: IndexRange,
    ) -> ChainResult<Vec<(InterchainGasPayment, LogMeta)>> {
        // TODO[cardano]: gas payments?
        Ok(vec![])
    }

    #[instrument(level = "debug", err, ret, skip(self))]
    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        // TODO[cardano]: gas payments?
        Ok(0)
    }
}
