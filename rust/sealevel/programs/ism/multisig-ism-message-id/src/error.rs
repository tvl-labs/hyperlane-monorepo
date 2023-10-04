//! Hyperlane Sealevel mailbox contract specific errors.

use solana_program::program_error::ProgramError;

use multisig_ism::error::MultisigIsmError;

#[derive(Copy, Clone, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum Error {
    #[error("Account not found in the correct order")]
    AccountOutOfOrder = 1,
    #[error("Account is not owner")]
    AccountNotOwner = 2,
    #[error("Program ID is not owner")]
    ProgramIdNotOwner = 3,
    #[error("Account not initialized")]
    AccountNotInitialized = 4,
    #[error("Invalid signature recovery ID")]
    InvalidSignatureRecoveryId = 5,
    #[error("Invalid signature")]
    InvalidSignature = 6,
    #[error("Threshold not met")]
    ThresholdNotMet = 7,
    #[error("Invalid validators and threshold")]
    InvalidValidatorsAndThreshold = 8,
    #[error("Already initialized")]
    AlreadyInitialized = 9,
    #[error("Invalid metadata")]
    InvalidMetadata = 10,
}

impl From<MultisigIsmError> for Error {
    fn from(err: MultisigIsmError) -> Self {
        match err {
            MultisigIsmError::InvalidSignature => Error::InvalidSignature,
            MultisigIsmError::ThresholdNotMet => Error::ThresholdNotMet,
        }
    }
}

impl From<Error> for ProgramError {
    fn from(err: Error) -> Self {
        ProgramError::Custom(err as u32)
    }
}
