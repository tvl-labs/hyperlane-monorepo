use async_trait::async_trait;
use ethers::prelude::{Address, Signature};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::transaction::eip712::Eip712;
use ethers_core::{
    k256::{
        ecdsa::{
            recoverable::{Id, Signature as RSig},
            Signature as KSig, VerifyingKey,
        },
        FieldBytes,
    },
    types::U256,
};
use ethers_signers::{AwsSigner, AwsSignerError, LocalWallet, Signer, WalletError};

use hyperlane_core::{HyperlaneSigner, HyperlaneSignerError, H160, H256};

/// Ethereum-supported signer types
#[derive(Debug, Clone)]
pub enum Signers {
    /// A wallet instantiated with a locally stored private key
    Local(LocalWallet),
    /// A signer using a key stored in aws kms
    Aws(AwsSigner),
}

impl From<LocalWallet> for Signers {
    fn from(s: LocalWallet) -> Self {
        Signers::Local(s)
    }
}

impl From<AwsSigner> for Signers {
    fn from(s: AwsSigner) -> Self {
        Signers::Aws(s)
    }
}

/// NOTE: PORTED PRIVATE CODE FROM ETHER-RS AWS TO SIGN RAW DIGEST
/// WE WOULDN'T NEED THIS IF `sign_digest_with_eip155` WAS PUBLIC.
///
/// Makes a trial recovery to check whether an RSig corresponds to a known
/// `VerifyingKey`
fn check_candidate(sig: &RSig, digest: [u8; 32], vk: &VerifyingKey) -> bool {
    if let Ok(key) = sig.recover_verifying_key_from_digest_bytes(digest.as_ref().into()) {
        key == *vk
    } else {
        false
    }
}
/// Recover an rsig from a signature under a known key by trial/error
fn rsig_from_digest_bytes_trial_recovery(sig: &KSig, digest: [u8; 32], vk: &VerifyingKey) -> RSig {
    let sig_0 = RSig::new(sig, Id::new(0).unwrap()).unwrap();
    let sig_1 = RSig::new(sig, Id::new(1).unwrap()).unwrap();

    if check_candidate(&sig_0, digest, vk) {
        sig_0
    } else if check_candidate(&sig_1, digest, vk) {
        sig_1
    } else {
        panic!("bad sig");
    }
}
fn rsig_to_ethsig(sig: &RSig) -> Signature {
    let v: u8 = sig.recovery_id().into();
    let v = (v + 27) as u64;
    let r_bytes: FieldBytes = sig.r().into();
    let s_bytes: FieldBytes = sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());
    Signature { r, s, v }
}
fn apply_eip155(sig: &mut Signature, chain_id: u64) {
    let v = (chain_id * 2 + 35) + ((sig.v - 1) % 2);
    sig.v = v;
}
/// END OF PORTED PRIVATE CODE FROM ETHER-RS AWS TO SIGN RAW DIGEST

impl Signers {
    /// Sign a raw digest without hashing it further.
    /// Useful for Blake2b checkpoints from EVM to Cardano.
    pub async fn sign_digest(&self, digest: H256) -> Result<Signature, SignersError> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_hash(digest)),
            Signers::Aws(signer) => {
                let sig = signer
                    .sign_digest(digest.into())
                    .await
                    .map_err(|err| AwsSignerError::from(err))?;
                let pubkey = signer
                    .get_pubkey()
                    .await
                    .map_err(|err| AwsSignerError::from(err))?;
                let sig = rsig_from_digest_bytes_trial_recovery(&sig, digest.into(), &pubkey);
                let mut sig = rsig_to_ethsig(&sig);
                apply_eip155(&mut sig, signer.chain_id());
                Ok(sig)
            }
        }
    }
}

#[async_trait]
impl Signer for Signers {
    type Error = SignersError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_message(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_message(message).await?),
        }
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_transaction(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_transaction(message).await?),
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::Aws(signer) => Ok(signer.sign_typed_data(payload).await?),
        }
    }

    fn address(&self) -> Address {
        match self {
            Signers::Local(signer) => signer.address(),
            Signers::Aws(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match self {
            Signers::Local(signer) => signer.chain_id(),
            Signers::Aws(signer) => signer.chain_id(),
        }
    }

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            Signers::Local(signer) => signer.with_chain_id(chain_id).into(),
            Signers::Aws(signer) => signer.with_chain_id(chain_id).into(),
        }
    }
}

#[async_trait]
impl HyperlaneSigner for Signers {
    fn eth_address(&self) -> H160 {
        Signer::address(self)
    }

    async fn sign_hash(
        &self,
        hash: &H256,
        // Sign a raw digest without hashing it further.
        // Useful for Blake2b checkpoints from EVM to Cardano.
        is_digest: bool,
    ) -> Result<Signature, HyperlaneSignerError> {
        let mut signature = if is_digest {
            Signers::sign_digest(self, *hash).await
        } else {
            Signer::sign_message(self, hash).await
        }
        .map_err(|err| HyperlaneSignerError::from(Box::new(err) as Box<_>))?;
        signature.v = 28 - (signature.v % 2);
        Ok(signature)
    }
}

/// Error types for Signers
#[derive(Debug, thiserror::Error)]
pub enum SignersError {
    /// AWS Signer Error
    #[error("{0}")]
    AwsSignerError(#[from] AwsSignerError),
    /// Wallet Signer Error
    #[error("{0}")]
    WalletError(#[from] WalletError),
}

impl From<std::convert::Infallible> for SignersError {
    fn from(_error: std::convert::Infallible) -> Self {
        panic!("infallible")
    }
}

#[cfg(test)]
mod test {
    use hyperlane_core::{Checkpoint, HyperlaneSigner, HyperlaneSignerExt, H256};

    use crate::signers::Signers;

    #[test]
    fn it_sign() {
        let t = async {
            let signer: Signers =
                "1111111111111111111111111111111111111111111111111111111111111111"
                    .parse::<ethers::signers::LocalWallet>()
                    .unwrap()
                    .into();
            let message = Checkpoint {
                mailbox_address: H256::repeat_byte(2),
                mailbox_domain: 5,
                root: H256::repeat_byte(1),
                index: 123,
            };

            let signed = signer.sign(message, false).await.expect("!sign");
            assert!(signed.signature.v == 27 || signed.signature.v == 28);
            signed.verify(signer.eth_address()).expect("!verify");
        };
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(t)
    }
}
