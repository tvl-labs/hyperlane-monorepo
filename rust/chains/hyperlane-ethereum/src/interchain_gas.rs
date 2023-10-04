#![allow(missing_docs)]

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::prelude::Middleware;
use tracing::instrument;

use hyperlane_core::{
    BlockRange, ChainCommunicationError, ChainResult, ContractLocator, HyperlaneAbi,
    HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider, IndexRange, Indexer,
    InterchainGasPaymaster, InterchainGasPayment, LogMeta, H160, H256,
};

use crate::contracts::i_interchain_gas_paymaster::{
    IInterchainGasPaymaster as EthereumInterchainGasPaymasterInternal, IINTERCHAINGASPAYMASTER_ABI,
};
use crate::trait_builder::BuildableWithProvider;
use crate::EthereumProvider;

impl<M> Display for EthereumInterchainGasPaymasterInternal<M>
where
    M: Middleware,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct InterchainGasPaymasterIndexerBuilder {
    pub mailbox_address: H160,
    pub finality_blocks: u32,
}

#[async_trait]
impl BuildableWithProvider for InterchainGasPaymasterIndexerBuilder {
    type Output = Box<dyn Indexer<InterchainGasPayment>>;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumInterchainGasPaymasterIndexer::new(
            Arc::new(provider),
            locator,
            self.finality_blocks,
        ))
    }
}

#[derive(Debug)]
/// Struct that retrieves event data for an Ethereum InterchainGasPaymaster
pub struct EthereumInterchainGasPaymasterIndexer<M>
where
    M: Middleware,
{
    contract: Arc<EthereumInterchainGasPaymasterInternal<M>>,
    provider: Arc<M>,
    finality_blocks: u32,
}

impl<M> EthereumInterchainGasPaymasterIndexer<M>
where
    M: Middleware + 'static,
{
    /// Create new EthereumInterchainGasPaymasterIndexer
    pub fn new(provider: Arc<M>, locator: &ContractLocator, finality_blocks: u32) -> Self {
        Self {
            contract: Arc::new(EthereumInterchainGasPaymasterInternal::new(
                locator.address,
                provider.clone(),
            )),
            provider,
            finality_blocks,
        }
    }
}

#[async_trait]
impl<M> Indexer<InterchainGasPayment> for EthereumInterchainGasPaymasterIndexer<M>
where
    M: Middleware + 'static,
{
    #[instrument(err, skip(self))]
    async fn fetch_logs(
        &self,
        range: IndexRange,
    ) -> ChainResult<Vec<(InterchainGasPayment, LogMeta)>> {
        let BlockRange(range) = range else {
            return Err(ChainCommunicationError::from_other_str(
                "EthereumInterchainGasPaymasterIndexer only supports block-based indexing",
            ));
        };

        let events = self
            .contract
            .gas_payment_filter()
            .from_block(*range.start())
            .to_block(*range.end())
            .query_with_meta()
            .await?;

        Ok(events
            .into_iter()
            .map(|(log, log_meta)| {
                (
                    InterchainGasPayment {
                        message_id: H256::from(log.message_id),
                        payment: log.payment.into(),
                        gas_amount: log.gas_amount.into(),
                    },
                    log_meta.into(),
                )
            })
            .collect())
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

pub struct InterchainGasPaymasterBuilder {}

#[async_trait]
impl BuildableWithProvider for InterchainGasPaymasterBuilder {
    type Output = Box<dyn InterchainGasPaymaster>;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumInterchainGasPaymaster::new(
            Arc::new(provider),
            locator,
        ))
    }
}

/// A reference to an InterchainGasPaymaster contract on some Ethereum chain
#[derive(Debug)]
pub struct EthereumInterchainGasPaymaster<M>
where
    M: Middleware,
{
    contract: Arc<EthereumInterchainGasPaymasterInternal<M>>,
    domain: HyperlaneDomain,
}

impl<M> EthereumInterchainGasPaymaster<M>
where
    M: Middleware + 'static,
{
    /// Create a reference to a mailbox at a specific Ethereum address on some
    /// chain
    pub fn new(provider: Arc<M>, locator: &ContractLocator) -> Self {
        Self {
            contract: Arc::new(EthereumInterchainGasPaymasterInternal::new(
                locator.address,
                provider,
            )),
            domain: locator.domain.clone(),
        }
    }
}

impl<M> HyperlaneChain for EthereumInterchainGasPaymaster<M>
where
    M: Middleware + 'static,
{
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(EthereumProvider::new(
            self.contract.client(),
            self.domain.clone(),
        ))
    }
}

impl<M> HyperlaneContract for EthereumInterchainGasPaymaster<M>
where
    M: Middleware + 'static,
{
    fn address(&self) -> H256 {
        self.contract.address().into()
    }
}

#[async_trait]
impl<M> InterchainGasPaymaster for EthereumInterchainGasPaymaster<M> where M: Middleware + 'static {}

pub struct EthereumInterchainGasPaymasterAbi;

impl HyperlaneAbi for EthereumInterchainGasPaymasterAbi {
    const SELECTOR_SIZE_BYTES: usize = 4;

    fn fn_map() -> HashMap<Vec<u8>, &'static str> {
        super::extract_fn_map(&IINTERCHAINGASPAYMASTER_ABI)
    }
}
