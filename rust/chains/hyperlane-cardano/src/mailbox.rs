use crate::cardano::Keypair;
use crate::provider::CardanoProvider;
use crate::rpc::CardanoRpc;
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
    pub outbox: H256,
    domain: HyperlaneDomain,
    cardano_rpc: CardanoRpc,
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
            cardano_rpc: CardanoRpc::new(&conf.url),
        })
    }

    pub async fn finalized_block_number(&self) -> Result<u32, ChainCommunicationError> {
        let finalized_block_number = self
            .cardano_rpc
            .get_finalized_block_number()
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(finalized_block_number)
    }

    pub async fn tree_and_tip(
        &self,
        lag: Option<NonZeroU64>,
    ) -> ChainResult<(IncrementalMerkle, u32)> {
        assert!(lag.is_none(), "Cardano always returns the finalized result");
        let merkle_tree_response = self
            .cardano_rpc
            .get_latest_merkle_tree()
            .await
            .map_err(ChainCommunicationError::from_other)?;
        let merkle_tree = merkle_tree_response.merkle_tree;
        let branch: [H256; TREE_DEPTH] = merkle_tree
            .branches
            .iter()
            .map(
                |b| H256::from_str(b).unwrap(), /* TODO[cardano]: better error handling for RPC output */
            )
            .collect::<Vec<H256>>()
            .try_into()
            .unwrap();
        let count = merkle_tree.count as usize;
        Ok((
            IncrementalMerkle::new(branch, count),
            merkle_tree_response.block_number as u32,
        ))
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
        self.tree_and_tip(lag).await.map(|(tree, _)| tree)
    }

    async fn count(&self, lag: Option<NonZeroU64>) -> ChainResult<u32> {
        self.tree(lag).await.map(|tree| tree.count() as u32)
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
        let res = self
            .cardano_rpc
            .is_inbox_message_delivered(id)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(res.is_delivered)
    }

    async fn default_ism(&self) -> ChainResult<H256> {
        // ISM on Cardano is a minting policy, not an address
        // TODO[cardano]: We could return the minting policy hash here?
        Ok(H256::zero())
    }

    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        // All messages share the same ISM at the moment
        self.default_ism().await
    }

    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        let res = self
            .cardano_rpc
            .submit_inbox_message(message, metadata)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(TxOutcome {
            txid: H256::from_str(res.tx_id.as_str()).unwrap(),
            executed: true,
            gas_used: U256::from(res.fee_lovelace),
            // NOTE: There's no "dynamic" gas price on Cardano
            gas_price: U256::from(res.fee_lovelace),
        })
    }

    async fn process_estimate_costs(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<TxCostEstimate> {
        let res = self
            .cardano_rpc
            .estimate_inbox_message_fee(message, metadata)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        let fee_lovelace = res.fee_lovelace as u32;
        Ok(TxCostEstimate {
            gas_limit: U256::from(fee_lovelace),
            // NOTE: There's no "dynamic" gas price on Cardano
            gas_price: U256::from(fee_lovelace),
            l2_gas_limit: None,
        })
    }

    fn process_calldata(&self, message: &HyperlaneMessage, metadata: &[u8]) -> Vec<u8> {
        todo!("Relayer") // TODO[cardano]
    }
}
