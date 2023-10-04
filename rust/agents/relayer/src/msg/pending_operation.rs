use std::cmp::Ordering;
use std::time::Instant;

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use eyre::Report;

use hyperlane_core::HyperlaneDomain;

#[allow(unused_imports)] // required for enum_dispatch
use super::pending_message::PendingMessage;

/// A pending operation that will be run by the submitter and cause a
/// transaction to be sent.
///
/// There are three stages to the lifecycle of a pending operation:
///
/// 1) Prepare: This is called before every submission and will usually have a
/// very short gap between it and the submit call. It can be used to confirm it
/// is ready to be submitted and it can also prepare any data that will be
/// needed for the submission. This way, the preparation can be done while
/// another transaction is being submitted.
///
/// 2) Submit: This is called to submit the operation to the destination
/// blockchain and report if it was successful or not. This is usually the act
/// of submitting a transaction. Ideally this step only sends the transaction
/// and waits for it to be included.
///
/// 3) Confirm: This is called after the operation has been submitted and is
/// responsible for checking if the operation has reached a point at which we
/// consider it safe from reorgs.
#[async_trait]
#[enum_dispatch]
pub trait PendingOperation {
    /// The domain this operation will take place on.
    fn domain(&self) -> &HyperlaneDomain;

    /// Prepare to submit this operation. This will be called before every
    /// submission and will usually have a very short gap between it and the
    /// submit call.
    async fn prepare(&mut self) -> PendingOperationResult;

    /// Submit this operation to the blockchain and report if it was successful
    /// or not.
    async fn submit(&mut self) -> PendingOperationResult;

    /// This will be called after the operation has been submitted and is
    /// responsible for checking if the operation has reached a point at
    /// which we consider it safe from reorgs.
    async fn confirm(&mut self) -> PendingOperationResult;

    /// Get the earliest instant at which this should next be attempted.
    ///
    /// This is only used for sorting, the functions are responsible for
    /// returning `NotReady` if it is too early and matters.
    fn _next_attempt_after(&self) -> Option<Instant>;

    #[cfg(test)]
    /// Set the number of times this operation has been retried.
    fn set_retries(&mut self, retries: u32);
}

/// A "dynamic" pending operation implementation which knows about the
/// different sub types and can properly implement PartialEq and
/// PartialOrd for them.
#[enum_dispatch(PendingOperation)]
#[derive(Debug, PartialEq, Eq)]
pub enum DynPendingOperation {
    PendingMessage,
}

impl PartialOrd for DynPendingOperation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Sort by their next allowed attempt time and if no allowed time is set,
/// then put it in front of those with a time (they have been tried
/// before) and break ties between ones that have not been tried with
/// the nonce.
impl Ord for DynPendingOperation {
    fn cmp(&self, other: &Self) -> Ordering {
        use DynPendingOperation::*;
        use Ordering::*;
        match (self._next_attempt_after(), other._next_attempt_after()) {
            (Some(a), Some(b)) => a.cmp(&b),
            // No time means it should come before
            (None, Some(_)) => Less,
            (Some(_), None) => Greater,
            (None, None) => match (self, other) {
                (PendingMessage(a), PendingMessage(b)) => {
                    if a.message.origin == b.message.origin {
                        // Should execute in order of nonce for the same origin
                        a.message.nonce.cmp(&b.message.nonce)
                    } else {
                        // There is no priority between these messages, so arbitrarily use the id
                        a.message.id().cmp(&b.message.id())
                    }
                }
            },
        }
    }
}

pub enum PendingOperationResult {
    /// Promote to the next step
    Success,
    /// This operation is not ready to be attempted again yet
    NotReady,
    /// Operation needs to be started from scratch again
    Reprepare,
    /// Do not attempt to run the operation again, forget about it
    Drop,
    /// Pass the error up the chain, this is non-recoverable and indicates a
    /// system failure.
    CriticalFailure(Report),
}

/// create a `op_try!` macro for the `on_retry` handler.
macro_rules! make_op_try {
    ($on_retry:expr) => {
        /// Handle a result and either return early with retry or a critical failure on
        /// error.
        macro_rules! op_try {
                                    (critical: $e:expr, $ctx:literal) => {
                                        match $e {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!(error=?e, concat!("Error when ", $ctx));
                                                return PendingOperationResult::CriticalFailure(
                                                    Err::<(), _>(e)
                                                        .context(concat!("When ", $ctx))
                                                        .unwrap_err()
                                                );
                                            }
                                        }
                                    };
                                    ($e:expr, $ctx:literal) => {
                                        match $e {
                                            Ok(v) => v,
                                            Err(e) => {
                                                warn!(error=?e, concat!("Error when ", $ctx));
                                                return $on_retry();
                                            }
                                        }
                                    };
                                }
    };
}

pub(super) use make_op_try;
