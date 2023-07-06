use crate::cardano::Keypair;
use crate::provider::CardanoProvider;
use crate::rpc::OutboxRpc;
use crate::ConnectionConf;
use async_trait::async_trait;
use hyperlane_core::accumulator::incremental::IncrementalMerkle;
use hyperlane_core::accumulator::TREE_DEPTH;
use hyperlane_core::{
    ChainCommunicationError, ChainResult, Checkpoint, ContractLocator, HyperlaneChain,
    HyperlaneContract, HyperlaneDomain, HyperlaneMessage, HyperlaneProvider, Mailbox,
    TxCostEstimate, TxOutcome, H256, U256,
};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU64;
use std::str::FromStr;

pub struct CardanoMailbox {
    inbox: H256,
    outbox: H256,
    domain: HyperlaneDomain,
    outbox_rpc: OutboxRpc,
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
            outbox_rpc: OutboxRpc::new(&conf.url),
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
        assert!(lag.is_none(), "Cardano always returns the finalized result");
        let finalized_block_number = self
            .outbox_rpc
            .get_finalized_block_number()
            .await
            .map_err(ChainCommunicationError::from_other)?;
        let merkle_trees_response = self
            .outbox_rpc
            .get_merkle_trees_at_block_number(finalized_block_number)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        let merkle_trees = merkle_trees_response.merkle_trees;
        if merkle_trees.is_empty() {
            return Ok(IncrementalMerkle::default());
        }
        let last_merkle_tree = merkle_trees.last().unwrap();
        let branch: [H256; TREE_DEPTH] = last_merkle_tree
            .branches
            .iter()
            .map(
                |b| H256::from_str(b).unwrap(), /* TODO: better error handling for RPC output */
            )
            .collect::<Vec<H256>>()
            .try_into()
            .unwrap();
        let count = last_merkle_tree.count as usize;
        Ok(IncrementalMerkle::new(branch, count))
    }

    async fn count(&self, lag: Option<NonZeroU64>) -> ChainResult<u32> {
        self.tree(lag).await.map(|t| t.count() as u32)
    }

    async fn latest_checkpoint(&self, lag: Option<NonZeroU64>) -> ChainResult<Checkpoint> {
        let tree = self.tree(lag).await?;
        let count: u32 = tree
            .count()
            .try_into()
            .map_err(ChainCommunicationError::from_other)?;
        let root = tree.root();
        let index = count.checked_sub(1).ok_or_else(|| {
            ChainCommunicationError::from_contract_error_str(
                "Outbox is empty, cannot compute checkpoint",
            )
        })?;
        return Ok(Checkpoint {
            mailbox_domain: self.domain.id(),
            mailbox_address: self.outbox,
            root,
            index,
        });
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
