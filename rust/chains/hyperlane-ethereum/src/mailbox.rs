#![allow(clippy::enum_variant_names)]
#![allow(missing_docs)]

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::abi::AbiEncode;
use ethers::prelude::Middleware;
use ethers_contract::builders::ContractCall;
use tracing::instrument;

use hyperlane_core::accumulator::incremental::IncrementalMerkle;
use hyperlane_core::accumulator::TREE_DEPTH;
use hyperlane_core::{
    utils::fmt_bytes, BlockRange, ChainCommunicationError, ChainResult, Checkpoint,
    ContractLocator, HyperlaneAbi, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProtocolError, HyperlaneProvider, IndexRange, Indexer, LogMeta,
    Mailbox, MessageIndexer, RawHyperlaneMessage, TxCostEstimate, TxOutcome, H160, H256, U256,
};

use crate::contracts::arbitrum_node_interface::ArbitrumNodeInterface;
use crate::contracts::i_mailbox::{IMailbox as EthereumMailboxInternal, ProcessCall, IMAILBOX_ABI};
use crate::trait_builder::BuildableWithProvider;
use crate::tx::{fill_tx_gas_params, report_tx};
use crate::EthereumProvider;

/// derived from `forge inspect Mailbox storage --pretty`
const MERKLE_TREE_CONTRACT_SLOT: u32 = 152;

impl<M> std::fmt::Display for EthereumMailboxInternal<M>
where
    M: Middleware,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct MessageIndexerBuilder {
    pub finality_blocks: u32,
}

#[async_trait]
impl BuildableWithProvider for MessageIndexerBuilder {
    type Output = Box<dyn MessageIndexer>;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumMailboxIndexer::new(
            Arc::new(provider),
            locator,
            self.finality_blocks,
        ))
    }
}

pub struct DeliveryIndexerBuilder {
    pub finality_blocks: u32,
}

#[async_trait]
impl BuildableWithProvider for DeliveryIndexerBuilder {
    type Output = Box<dyn Indexer<H256>>;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumMailboxIndexer::new(
            Arc::new(provider),
            locator,
            self.finality_blocks,
        ))
    }
}

#[derive(Debug, Clone)]
/// Struct that retrieves event data for an Ethereum mailbox
pub struct EthereumMailboxIndexer<M>
where
    M: Middleware,
{
    contract: Arc<EthereumMailboxInternal<M>>,
    provider: Arc<M>,
    finality_blocks: u32,
}

impl<M> EthereumMailboxIndexer<M>
where
    M: Middleware + 'static,
{
    /// Create new EthereumMailboxIndexer
    pub fn new(provider: Arc<M>, locator: &ContractLocator, finality_blocks: u32) -> Self {
        let contract = Arc::new(EthereumMailboxInternal::new(
            locator.address,
            provider.clone(),
        ));
        Self {
            contract,
            provider,
            finality_blocks,
        }
    }

    #[instrument(level = "debug", err, ret, skip(self))]
    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        Ok(self
            .provider
            .get_block_number()
            .await
            .map_err(ChainCommunicationError::from_other)?
            .as_u32()
            .saturating_sub(self.finality_blocks))
    }
}

#[async_trait]
impl<M> Indexer<HyperlaneMessage> for EthereumMailboxIndexer<M>
where
    M: Middleware + 'static,
{
    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.get_finalized_block_number().await
    }

    #[instrument(err, skip(self))]
    async fn fetch_logs(&self, range: IndexRange) -> ChainResult<Vec<(HyperlaneMessage, LogMeta)>> {
        let BlockRange(range) = range else {
            return Err(ChainCommunicationError::from_other_str(
                "EthereumMailboxIndexer only supports block-based indexing",
            ))
        };

        let mut events: Vec<(HyperlaneMessage, LogMeta)> = self
            .contract
            .dispatch_filter()
            .from_block(*range.start())
            .to_block(*range.end())
            .query_with_meta()
            .await?
            .into_iter()
            .map(|(event, meta)| (HyperlaneMessage::from(event.message.to_vec()), meta.into()))
            .collect();

        events.sort_by(|a, b| a.0.nonce.cmp(&b.0.nonce));
        Ok(events)
    }
}

#[async_trait]
impl<M> MessageIndexer for EthereumMailboxIndexer<M>
where
    M: Middleware + 'static,
{
    #[instrument(err, skip(self))]
    async fn fetch_count_at_tip(&self) -> ChainResult<(u32, u32)> {
        let tip = Indexer::<HyperlaneMessage>::get_finalized_block_number(self as _).await?;
        let base_call = self.contract.count();
        let call_at_tip = base_call.block(u64::from(tip));
        let count = call_at_tip.call().await?;
        Ok((count, tip))
    }
}

#[async_trait]
impl<M> Indexer<H256> for EthereumMailboxIndexer<M>
where
    M: Middleware + 'static,
{
    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.get_finalized_block_number().await
    }

    #[instrument(err, skip(self))]
    async fn fetch_logs(&self, range: IndexRange) -> ChainResult<Vec<(H256, LogMeta)>> {
        let BlockRange(range) = range else {
            return Err(ChainCommunicationError::from_other_str(
                "EthereumMailboxIndexer only supports block-based indexing",
            ))
        };

        Ok(self
            .contract
            .process_id_filter()
            .from_block(*range.start())
            .to_block(*range.end())
            .query_with_meta()
            .await?
            .into_iter()
            .map(|(event, meta)| (H256::from(event.message_id), meta.into()))
            .collect())
    }
}
pub struct MailboxBuilder {}

#[async_trait]
impl BuildableWithProvider for MailboxBuilder {
    type Output = Box<dyn Mailbox>;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumMailbox::new(Arc::new(provider), locator))
    }
}

/// A reference to a Mailbox contract on some Ethereum chain
#[derive(Debug)]
pub struct EthereumMailbox<M>
where
    M: Middleware,
{
    contract: Arc<EthereumMailboxInternal<M>>,
    domain: HyperlaneDomain,
    provider: Arc<M>,
    arbitrum_node_interface: Option<Arc<ArbitrumNodeInterface<M>>>,
}

impl<M> EthereumMailbox<M>
where
    M: Middleware + 'static,
{
    /// Create a reference to a mailbox at a specific Ethereum address on some
    /// chain
    pub fn new(provider: Arc<M>, locator: &ContractLocator) -> Self {
        // Arbitrum Nitro based chains are a special case for transaction cost estimation.
        // The gas amount that eth_estimateGas returns considers both L1 and L2 gas costs.
        // We use the NodeInterface, found at address(0xC8), to isolate the L2 gas costs.
        // See https://developer.arbitrum.io/arbos/gas#nodeinterfacesol or https://github.com/OffchainLabs/nitro/blob/master/contracts/src/node-interface/NodeInterface.sol#L25
        let arbitrum_node_interface = locator.domain.is_arbitrum_nitro().then(|| {
            Arc::new(ArbitrumNodeInterface::new(
                H160::from_low_u64_be(0xC8),
                provider.clone(),
            ))
        });

        Self {
            contract: Arc::new(EthereumMailboxInternal::new(
                locator.address,
                provider.clone(),
            )),
            domain: locator.domain.clone(),
            provider,
            arbitrum_node_interface,
        }
    }

    /// Returns a ContractCall that processes the provided message.
    /// If the provided tx_gas_limit is None, gas estimation occurs.
    async fn process_contract_call(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<ContractCall<M, ()>> {
        let tx = self.contract.process(
            metadata.to_vec().into(),
            RawHyperlaneMessage::from(message).to_vec().into(),
        );
        fill_tx_gas_params(tx, tx_gas_limit, self.provider.clone(), message.destination).await
    }
}

impl<M> HyperlaneChain for EthereumMailbox<M>
where
    M: Middleware + 'static,
{
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(EthereumProvider::new(
            self.provider.clone(),
            self.domain.clone(),
        ))
    }
}

impl<M> HyperlaneContract for EthereumMailbox<M>
where
    M: Middleware + 'static,
{
    fn address(&self) -> H256 {
        self.contract.address().into()
    }
}

#[async_trait]
impl<M> Mailbox for EthereumMailbox<M>
where
    M: Middleware + 'static,
{
    #[instrument(skip(self))]
    async fn count(&self, maybe_lag: Option<NonZeroU64>) -> ChainResult<u32> {
        let base_call = self.contract.count();
        let call_with_lag = if let Some(lag) = maybe_lag {
            let tip = self
                .provider
                .get_block_number()
                .await
                .map_err(ChainCommunicationError::from_other)?
                .as_u64();
            base_call.block(tip.saturating_sub(lag.get()))
        } else {
            base_call
        };
        let count = call_with_lag.call().await?;
        Ok(count)
    }

    #[instrument(skip(self))]
    async fn delivered(&self, id: H256) -> ChainResult<bool> {
        Ok(self.contract.delivered(id.into()).call().await?)
    }

    #[instrument(skip(self))]
    async fn latest_checkpoint(&self, maybe_lag: Option<NonZeroU64>) -> ChainResult<Checkpoint> {
        let base_call = self.contract.latest_checkpoint();
        let call_with_lag = match maybe_lag {
            Some(lag) => {
                let tip = self
                    .provider
                    .get_block_number()
                    .await
                    .map_err(ChainCommunicationError::from_other)?
                    .as_u64();
                base_call.block(tip.saturating_sub(lag.get()))
            }
            None => base_call,
        };
        let (root, index) = call_with_lag.call().await?;
        Ok(Checkpoint {
            mailbox_address: self.address(),
            mailbox_domain: self.domain.id(),
            root: root.into(),
            index,
        })
    }

    #[instrument(skip(self))]
    #[allow(clippy::needless_range_loop)]
    async fn tree(&self, lag: Option<NonZeroU64>) -> ChainResult<IncrementalMerkle> {
        let lag = lag.map(|v| v.get()).unwrap_or(0).into();

        // use consistent block for all storage slot or view calls to prevent
        // race conditions where tree contents change between calls
        let fixed_block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(ChainCommunicationError::from_other)?
            .saturating_sub(lag)
            .into();

        let expected_root = self
            .contract
            .root()
            .block(fixed_block_number)
            .call()
            .await?
            .into();

        // TODO: migrate to single contract view call once mailbox is upgraded
        // see https://github.com/hyperlane-xyz/hyperlane-monorepo/issues/2250
        // let branch = self.contract.branch().block(block_number).call().await;

        let mut branch = [H256::zero(); TREE_DEPTH];

        for index in 0..TREE_DEPTH {
            let slot = U256::from(MERKLE_TREE_CONTRACT_SLOT) + index;
            let mut location = [0u8; 32];
            slot.to_big_endian(&mut location);

            branch[index] = self
                .provider
                .get_storage_at(
                    self.contract.address(),
                    location.into(),
                    Some(fixed_block_number),
                )
                .await
                .map(Into::into)
                .map_err(ChainCommunicationError::from_other)?;
        }

        let count = self
            .contract
            .count()
            .block(fixed_block_number)
            .call()
            .await? as usize;

        let tree = IncrementalMerkle::new(branch, count);

        // validate tree built from storage slot lookups matches expected
        // result from root() view call at consistent block
        assert_eq!(tree.root(), expected_root);

        Ok(tree)
    }

    #[instrument(skip(self))]
    async fn default_ism(&self) -> ChainResult<H256> {
        Ok(self.contract.default_ism().call().await?.into())
    }

    #[instrument(skip(self))]
    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        Ok(self
            .contract
            .recipient_ism(recipient.into())
            .call()
            .await?
            .into())
    }

    #[instrument(skip(self), fields(metadata=%fmt_bytes(metadata)))]
    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
        tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        let contract_call = self
            .process_contract_call(message, metadata, tx_gas_limit)
            .await?;
        let receipt = report_tx(contract_call).await?;
        Ok(receipt.into())
    }

    #[instrument(skip(self), fields(msg=%message, metadata=%fmt_bytes(metadata)))]
    async fn process_estimate_costs(
        &self,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<TxCostEstimate> {
        let contract_call = self.process_contract_call(message, metadata, None).await?;
        let gas_limit = contract_call
            .tx
            .gas()
            .copied()
            .ok_or(HyperlaneProtocolError::ProcessGasLimitRequired)?;

        // If we have a ArbitrumNodeInterface, we need to set the l2_gas_limit.
        let l2_gas_limit = if let Some(arbitrum_node_interface) = &self.arbitrum_node_interface {
            Some(
                arbitrum_node_interface
                    .estimate_retryable_ticket(
                        H160::zero().into(),
                        // Give the sender a deposit, otherwise it reverts
                        U256::MAX.into(),
                        self.contract.address(),
                        U256::zero().into(),
                        H160::zero().into(),
                        H160::zero().into(),
                        contract_call.calldata().unwrap_or_default(),
                    )
                    .estimate_gas()
                    .await?,
            )
        } else {
            None
        };

        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(ChainCommunicationError::from_other)?;

        Ok(TxCostEstimate {
            gas_limit: gas_limit.into(),
            gas_price: gas_price.into(),
            l2_gas_limit: l2_gas_limit.map(|v| v.into()),
        })
    }

    fn process_calldata(&self, message: &HyperlaneMessage, metadata: &[u8]) -> Vec<u8> {
        let process_call = ProcessCall {
            message: RawHyperlaneMessage::from(message).to_vec().into(),
            metadata: metadata.to_vec().into(),
        };

        AbiEncode::encode(process_call)
    }
}

pub struct EthereumMailboxAbi;

impl HyperlaneAbi for EthereumMailboxAbi {
    const SELECTOR_SIZE_BYTES: usize = 4;

    fn fn_map() -> HashMap<Vec<u8>, &'static str> {
        super::extract_fn_map(&IMAILBOX_ABI)
    }
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, sync::Arc};

    use ethers::{
        providers::{MockProvider, Provider},
        types::{Block, Transaction, U256 as EthersU256},
    };

    use hyperlane_core::{
        ContractLocator, HyperlaneDomain, HyperlaneMessage, KnownHyperlaneDomain, Mailbox,
        TxCostEstimate, H160, H256, U256,
    };

    use crate::EthereumMailbox;

    /// An amount of gas to add to the estimated gas
    const GAS_ESTIMATE_BUFFER: u32 = 50000;

    #[tokio::test]
    async fn test_process_estimate_costs_sets_l2_gas_limit_for_arbitrum() {
        let mock_provider = Arc::new(MockProvider::new());
        let provider = Arc::new(Provider::new(mock_provider.clone()));

        let mailbox = EthereumMailbox::new(
            provider.clone(),
            &ContractLocator {
                // An Arbitrum Nitro chain
                domain: &HyperlaneDomain::Known(KnownHyperlaneDomain::ArbitrumGoerli),
                // Address doesn't matter because we're using a MockProvider
                address: H256::default(),
            },
        );

        let message = HyperlaneMessage::default();
        let metadata: Vec<u8> = vec![];

        assert!(mailbox.arbitrum_node_interface.is_some());
        // Confirm `H160::from_low_u64_ne(0xC8)` does what's expected
        assert_eq!(
            H160::from(mailbox.arbitrum_node_interface.as_ref().unwrap().address()),
            H160::from_str("0x00000000000000000000000000000000000000C8").unwrap(),
        );

        // The MockProvider responses we push are processed in LIFO
        // order, so we start with the final RPCs and work toward the first
        // RPCs

        // RPC 4: eth_gasPrice by process_estimate_costs
        // Return 15 gwei
        let gas_price: U256 =
            EthersU256::from(ethers::utils::parse_units("15", "gwei").unwrap()).into();
        mock_provider.push(gas_price).unwrap();

        // RPC 3: eth_estimateGas to the ArbitrumNodeInterface's estimateRetryableTicket function by process_estimate_costs
        let l2_gas_limit = U256::from(200000); // 200k gas
        mock_provider.push(l2_gas_limit).unwrap();

        // RPC 2: eth_getBlockByNumber from the estimate_eip1559_fees call in process_contract_call
        mock_provider.push(Block::<Transaction>::default()).unwrap();

        // RPC 1: eth_estimateGas from the estimate_gas call in process_contract_call
        // Return 1M gas
        let gas_limit = U256::from(1000000u32);
        mock_provider.push(gas_limit).unwrap();

        let tx_cost_estimate = mailbox
            .process_estimate_costs(&message, &metadata)
            .await
            .unwrap();

        // The TxCostEstimat's gas limit includes the buffer
        let estimated_gas_limit = gas_limit.saturating_add(GAS_ESTIMATE_BUFFER.into());

        assert_eq!(
            tx_cost_estimate,
            TxCostEstimate {
                gas_limit: estimated_gas_limit,
                gas_price,
                l2_gas_limit: Some(l2_gas_limit),
            },
        );
    }
}
