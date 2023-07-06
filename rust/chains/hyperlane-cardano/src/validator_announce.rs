use crate::provider::CardanoProvider;
use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::{
    Announcement, ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneProvider, SignedType, TxOutcome, ValidatorAnnounce, H256, U256,
};

#[derive(Debug)]
pub struct CardanoValidatorAnnounce {
    domain: HyperlaneDomain,
}

impl CardanoValidatorAnnounce {
    pub fn new(conf: &ConnectionConf, locator: ContractLocator) -> Self {
        Self {
            domain: locator.domain.clone(),
        }
    }
}

impl HyperlaneContract for CardanoValidatorAnnounce {
    fn address(&self) -> H256 {
        H256::zero() // TODO[cardano]
    }
}

impl HyperlaneChain for CardanoValidatorAnnounce {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(CardanoProvider::new(self.domain.clone()))
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
        // TODO[cardano]: auto-announcing of validator is probably not needed?
        Ok(TxOutcome {
            txid: H256::zero(),
            executed: false,
            gas_used: U256::zero(),
            gas_price: U256::zero(),
        })
    }

    async fn announce_tokens_needed(&self, announcement: SignedType<Announcement>) -> Option<U256> {
        // TODO[cardano]: auto-announcing of validator is probably not needed?
        Some(U256::zero())
    }
}
