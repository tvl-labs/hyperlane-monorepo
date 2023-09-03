use crate::provider::CardanoProvider;
use crate::rpc::CardanoRpc;
use crate::ConnectionConf;
use async_trait::async_trait;
use hex::ToHex;
use hyperlane_core::{
    Announcement, ChainCommunicationError, ChainResult, ContractLocator, HyperlaneChain,
    HyperlaneContract, HyperlaneDomain, HyperlaneProvider, SignedType, TxOutcome,
    ValidatorAnnounce, H256, U256,
};

#[derive(Debug)]
pub struct CardanoValidatorAnnounce {
    cardano_rpc: CardanoRpc,
    domain: HyperlaneDomain,
}

impl CardanoValidatorAnnounce {
    pub fn new(conf: &ConnectionConf, locator: ContractLocator) -> Self {
        let cardano_rpc = CardanoRpc::new(&conf.url);
        Self {
            cardano_rpc,
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
        self.cardano_rpc
            .get_validator_storage_locations(validators)
            .await
            .map_err(ChainCommunicationError::from_other)
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
