use ethers_core::abi::AbiEncode;
use sha3::{digest::Update, Digest, Keccak256};
use std::fmt::{Debug, Display, Formatter};

use crate::accumulator::Blake2b256;
use crate::utils::{fmt_address_for_domain, fmt_domain};
use crate::{Decode, Encode, HyperlaneProtocolError, KnownHyperlaneDomain, H256};

const HYPERLANE_MESSAGE_PREFIX_LEN: usize = 77;

/// A message ID that has been delivered to the destination
pub type Delivery = H256;

/// A Stamped message that has been committed at some nonce
pub type RawHyperlaneMessage = Vec<u8>;

impl From<&HyperlaneMessage> for RawHyperlaneMessage {
    fn from(m: &HyperlaneMessage) -> Self {
        let mut message_vec = vec![];
        m.write_to(&mut message_vec).expect("!write_to");
        message_vec
    }
}

/// A full Hyperlane message between chains
#[derive(Default, Clone)]
pub struct HyperlaneMessage {
    /// 1   Hyperlane version number
    pub version: u8,
    /// 4   Message nonce
    pub nonce: u32,
    /// 4   Origin domain ID
    pub origin: u32,
    /// 32  Address in origin convention
    pub sender: H256,
    /// 4   Destination domain ID
    pub destination: u32,
    /// 32  Address in destination convention
    pub recipient: H256,
    /// 0+  Message contents
    pub body: Vec<u8>,
}

impl Debug for HyperlaneMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HyperlaneMessage {{ id: {:?}, version: {}, nonce: {}, origin: {}, sender: {}, destination: {}, recipient: {}, body: 0x{} }}",
            self.id(),
            self.version,
            self.nonce,
            fmt_domain(self.origin),
            fmt_address_for_domain(self.origin, self.sender),
            fmt_domain(self.destination),
            fmt_address_for_domain(self.destination, self.recipient),
            hex::encode(&self.body)
        )
    }
}

impl Display for HyperlaneMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HyperlaneMessage {{ id: {:?}, nonce: {}, .. }}",
            self.id(),
            self.nonce
        )
    }
}

impl From<RawHyperlaneMessage> for HyperlaneMessage {
    fn from(m: RawHyperlaneMessage) -> Self {
        HyperlaneMessage::from(&m)
    }
}

impl From<&RawHyperlaneMessage> for HyperlaneMessage {
    fn from(m: &RawHyperlaneMessage) -> Self {
        let version = m[0];
        let nonce: [u8; 4] = m[1..5].try_into().unwrap();
        let origin: [u8; 4] = m[5..9].try_into().unwrap();
        let sender: [u8; 32] = m[9..41].try_into().unwrap();
        let destination: [u8; 4] = m[41..45].try_into().unwrap();
        let recipient: [u8; 32] = m[45..77].try_into().unwrap();
        let body = m[77..].try_into().unwrap();
        Self {
            version,
            nonce: u32::from_be_bytes(nonce),
            origin: u32::from_be_bytes(origin),
            sender: H256::from(sender),
            destination: u32::from_be_bytes(destination),
            recipient: H256::from(recipient),
            body,
        }
    }
}

impl Encode for HyperlaneMessage {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&self.nonce.to_be_bytes())?;
        writer.write_all(&self.origin.to_be_bytes())?;
        writer.write_all(self.sender.as_ref())?;
        writer.write_all(&self.destination.to_be_bytes())?;
        writer.write_all(self.recipient.as_ref())?;
        writer.write_all(&self.body)?;
        Ok(HYPERLANE_MESSAGE_PREFIX_LEN + self.body.len())
    }
}

impl Decode for HyperlaneMessage {
    fn read_from<R>(reader: &mut R) -> Result<Self, HyperlaneProtocolError>
    where
        R: std::io::Read,
    {
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;

        let mut nonce = [0u8; 4];
        reader.read_exact(&mut nonce)?;

        let mut origin = [0u8; 4];
        reader.read_exact(&mut origin)?;

        let mut sender = H256::zero();
        reader.read_exact(sender.as_mut())?;

        let mut destination = [0u8; 4];
        reader.read_exact(&mut destination)?;

        let mut recipient = H256::zero();
        reader.read_exact(recipient.as_mut())?;

        let mut body = vec![];
        reader.read_to_end(&mut body)?;

        Ok(Self {
            version: u8::from_be_bytes(version),
            nonce: u32::from_be_bytes(nonce),
            origin: u32::from_be_bytes(origin),
            sender,
            destination: u32::from_be_bytes(destination),
            recipient,
            body,
        })
    }
}

impl HyperlaneMessage {
    /// Convert the message to a message id
    pub fn id(&self) -> H256 {
        // TODO[cardano]: a better condition.
        let destination_domain = KnownHyperlaneDomain::try_from(self.destination);
        if destination_domain.is_ok()
            && destination_domain.unwrap() == KnownHyperlaneDomain::CardanoTest1
        {
            return H256::from_slice(Blake2b256::new().chain(self.to_vec()).finalize().as_slice());
        }
        H256::from_slice(Keccak256::new().chain(self.to_vec()).finalize().as_slice())
    }

    pub fn id_for_merkle_tree(&self) -> H256 {
        // TODO[cardano]: a better condition.
        let origin_domain = KnownHyperlaneDomain::try_from(self.origin);
        if origin_domain.is_ok() && origin_domain.unwrap() == KnownHyperlaneDomain::CardanoTest1 {
            return H256::from_slice(Blake2b256::new().chain(self.to_vec()).finalize().as_slice());
        }
        H256::from_slice(Keccak256::new().chain(self.to_vec()).finalize().as_slice())
    }

    /* FIXME don't need?
        /// Formats a hyperlane message as a sealevel vm log "event".
        pub fn format(&self) -> Result<String, HyperlaneProtocolError> {
            let mut serialized = vec![];
            self.write_to(&mut serialized)?;
            let encoded = bs58::encode(serialized).into_string();
            Ok(encoded)
        }

        /// Parses a hyperlane message from a sealevel vm log "event".
        pub fn parse(formatted: &[u8]) -> Result<Self, HyperlaneProtocolError> {
            let decoded = bs58::decode(formatted).into_vec().map_err(|err| {
                HyperlaneProtocolError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err
                ))
            })?;
            Self::read_from(&mut std::io::Cursor::new(decoded))
        }
    */
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;
    use std::str::FromStr;

    #[test]
    fn it_calculates_message_id() {
        let message = HyperlaneMessage {
            version: 0,
            nonce: 42,
            origin: 112233,
            sender: H256::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000CA1",
            )
            .unwrap(),
            destination: 43113,
            recipient: H256::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000EF1",
            )
            .unwrap(),
            body: Vec::from_hex("abcdef").unwrap(),
        };
        let id = message.id();
        assert_eq!(
            id,
            H256::from_str("0x4effd736ec49d8ed6cecdc39a76be0ab896c7dcae94cd2d95d797b55ec2edeab")
                .unwrap()
        );
    }
}
