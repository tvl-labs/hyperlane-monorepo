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
        assert!(validators.len() == 1);
        // TODO[cardano]
        Ok(vec![vec![
            "file:///tmp/test_cardano_checkpoints_0x70997970c51812dc3a010c7d01b50e0d17dc79c8"
                .to_string(),
        ]])
    }

    async fn announce(
        &self,
        announcement: SignedType<Announcement>,
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        todo!() // TODO[cardano]
    }

    async fn announce_tokens_needed(&self, announcement: SignedType<Announcement>) -> Option<U256> {
        todo!() // TODO[cardano]
    }
}
