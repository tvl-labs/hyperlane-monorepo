use crate::cardano::signer::Keypair;
use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::accumulator::incremental::IncrementalMerkle;
use hyperlane_core::{
    ChainResult, Checkpoint, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, Mailbox, TxCostEstimate, TxOutcome, H256, U256,
};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU64;

pub struct CardanoMailbox {}

impl CardanoMailbox {
    pub fn new(
        conf: &ConnectionConf,
        locator: ContractLocator,
        payer: Option<Keypair>,
    ) -> ChainResult<Self> {
        return Ok(CardanoMailbox {});
    }
}

impl HyperlaneContract for CardanoMailbox {
    fn address(&self) -> H256 {
        todo!()
    }
}

impl HyperlaneChain for CardanoMailbox {
    fn domain(&self) -> &HyperlaneDomain {
        todo!()
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        todo!()
    }
}

impl Debug for CardanoMailbox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[async_trait]
impl Mailbox for CardanoMailbox {
    async fn tree(&self, lag: Option<NonZeroU64>) -> ChainResult<IncrementalMerkle> {
        todo!()
    }

    async fn count(&self, lag: Option<NonZeroU64>) -> ChainResult<u32> {
        todo!()
    }

    async fn delivered(&self, id: H256) -> ChainResult<bool> {
        todo!()
    }

    async fn latest_checkpoint(&self, lag: Option<NonZeroU64>) -> ChainResult<Checkpoint> {
        todo!()
    }

    async fn default_ism(&self) -> ChainResult<H256> {
        todo!()
    }

    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        todo!()
    }

    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        todo!()
    }

    async fn process_estimate_costs(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<TxCostEstimate> {
        todo!()
    }

    fn process_calldata(&self, message: &HyperlaneMessage, metadata: &[u8]) -> Vec<u8> {
        todo!()
    }
}
