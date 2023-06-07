use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::{
    Announcement, ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneProvider, SignedType, TxOutcome, ValidatorAnnounce, H256, U256,
};

#[derive(Debug)]
pub struct CardanoValidatorAnnounce {}

impl CardanoValidatorAnnounce {
    pub fn new(conf: &ConnectionConf, locator: ContractLocator) -> Self {
        Self {}
    }
}

impl HyperlaneContract for CardanoValidatorAnnounce {
    fn address(&self) -> H256 {
        todo!() // TODO[cardano]
    }
}

impl HyperlaneChain for CardanoValidatorAnnounce {
    fn domain(&self) -> &HyperlaneDomain {
        todo!() // TODO[cardano]
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        todo!() // TODO[cardano]
    }
}

#[async_trait]
impl ValidatorAnnounce for CardanoValidatorAnnounce {
    async fn get_announced_storage_locations(
        &self,
        validators: &[H256],
    ) -> ChainResult<Vec<Vec<String>>> {
        todo!() // TODO[cardano]
    }

    async fn announce(
        &self,
        announcement: SignedType<Announcement>,
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        todo!() // TODO[cardano]
    }

    async fn announce_tokens_needed(
        &self,
        announcement: SignedType<Announcement>,
    ) -> ChainResult<U256> {
        todo!() // TODO[cardano]
    }
}
