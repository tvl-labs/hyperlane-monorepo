use crate::cardano::Keypair;
use crate::provider::CardanoProvider;
use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::accumulator::incremental::IncrementalMerkle;
use hyperlane_core::{
    ChainResult, Checkpoint, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, IndexRange, Indexer, LogMeta, Mailbox, MessageIndexer,
    TxCostEstimate, TxOutcome, H256, U256,
};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU64;

pub struct CardanoMailbox {
    inbox: H256,
    outbox: H256,
    domain: HyperlaneDomain,
}

impl CardanoMailbox {
    pub fn new(
        conf: &ConnectionConf,
        locator: ContractLocator,
        payer: Option<Keypair>,
    ) -> ChainResult<Self> {
        Ok(CardanoMailbox {
            domain: locator.domain.clone(),
            inbox: locator.address,
            outbox: locator.address,
        })
    }
}

impl HyperlaneContract for CardanoMailbox {
    fn address(&self) -> H256 {
        self.outbox
    }
}

impl HyperlaneChain for CardanoMailbox {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(CardanoProvider::new(self.domain.clone()))
    }
}

impl Debug for CardanoMailbox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self as &dyn HyperlaneContract)
    }
}

#[async_trait]
impl Mailbox for CardanoMailbox {
    async fn tree(&self, lag: Option<NonZeroU64>) -> ChainResult<IncrementalMerkle> {
        // TODO[cardano]
        Ok(IncrementalMerkle::default())
    }

    async fn count(&self, lag: Option<NonZeroU64>) -> ChainResult<u32> {
        // TODO[cardano]
        Ok(0)
    }

    async fn latest_checkpoint(&self, lag: Option<NonZeroU64>) -> ChainResult<Checkpoint> {
        // TODO[cardano]
        Ok(Checkpoint {
            mailbox_domain: self.domain.id(),
            mailbox_address: self.outbox,
            index: 0,
            root: H256::zero(),
        })
    }

    async fn delivered(&self, id: H256) -> ChainResult<bool> {
        todo!("Relayer") // TODO[cardano]
    }

    async fn default_ism(&self) -> ChainResult<H256> {
        todo!("Relayer") // TODO[cardano]
    }

    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        todo!("Relayer") // TODO[cardano]
    }

    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        todo!("Relayer") // TODO[cardano]
    }

    async fn process_estimate_costs(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<TxCostEstimate> {
        todo!("Relayer") // TODO[cardano]
    }

    fn process_calldata(&self, message: &HyperlaneMessage, metadata: &[u8]) -> Vec<u8> {
        todo!("Relayer") // TODO[cardano]
    }
}
