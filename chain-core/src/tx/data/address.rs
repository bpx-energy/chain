use parity_scale_codec::{Decode, Encode, Error, Input, Output};

use serde::{Deserialize, Serialize};

use std::fmt;

use std::str::FromStr;

use crate::common::H256;

use crate::init::address::{CroAddress, CroAddressError};

use bech32::{self, u5, FromBase32, ToBase32};

use crate::init::network::{get_bech32_human_part_from_network, get_network, Network};

type TreeRoot = H256;

/// MAST of Or operations (records the root).
/// Root of a Merkle tree where leafs are X-only
/// (potentially summed up / combined) pubkeys
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub enum ExtendedAddr {
    /// ref: https://blockstream.com/2015/08/24/en-treesignatures/
    /// but each operation is "OR"
    /// (root of such tree)
    OrTree(TreeRoot),
}

impl Encode for ExtendedAddr {
    fn encode_to<EncOut: Output>(&self, dest: &mut EncOut) {
        match *self {
            ExtendedAddr::OrTree(ref aa) => {
                dest.push_byte(0);
                dest.push(aa);
            }
        }
    }

    fn size_hint(&self) -> usize {
        (match self {
            ExtendedAddr::OrTree(ref aa) => aa.size_hint(),
        }) + 1
    }
}

impl Decode for ExtendedAddr {
    fn decode<DecIn: Input>(input: &mut DecIn) -> Result<Self, Error> {
        let tag = input.read_byte()?;
        // NOTE: tag 1 may be used for other address types -- e.g. one to denote
        // requiring a different witness type (leaf may be a combination of root + timelock)
        match tag {
            0 => Ok(ExtendedAddr::OrTree({
                let address: TreeRoot = Decode::decode(input)?;
                address
            })),
            _ => Err("No such variant in enum ExtendedAddr".into()),
        }
    }
}

impl CroAddress<ExtendedAddr> for ExtendedAddr {
    fn to_cro(&self, network: Network) -> Result<String, CroAddressError> {
        match self {
            ExtendedAddr::OrTree(hash) => {
                let checked_data: Vec<u5> = hash.to_vec().to_base32();
                let encoded =
                    bech32::encode(get_bech32_human_part_from_network(network), checked_data)
                        .expect("bech32 encoding error");
                Ok(encoded)
            }
        }
    }

    fn from_cro(encoded_addr: &str, network: Network) -> Result<Self, CroAddressError> {
        if !encoded_addr.starts_with(get_bech32_human_part_from_network(network)) {
            return Err(CroAddressError::InvalidNetwork);
        }

        bech32::decode(encoded_addr)
            .map_err(|e| CroAddressError::Bech32Error(e.to_string()))
            .and_then(|decoded| {
                Vec::from_base32(&decoded.1).map_err(|_e| CroAddressError::ConvertError)
            })
            .map(|hash| {
                let mut tree_root_hash: TreeRoot = [0 as u8; 32];
                tree_root_hash.copy_from_slice(&hash.as_slice());
                ExtendedAddr::OrTree(tree_root_hash)
            })
    }
}

impl fmt::Display for ExtendedAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_cro(get_network()).unwrap())
    }
}

impl FromStr for ExtendedAddr {
    type Err = CroAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ExtendedAddr::from_cro(s, get_network()).map_err(|_e| CroAddressError::ConvertError)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_be_correct_textual_address() {
        let network = Network::Devnet;

        let mut tree_root_hash = [0; 32];
        tree_root_hash.copy_from_slice(
            &hex::decode("0e7c045110b8dbf29765047380898919c5cb56f400112233445566778899aabb")
                .unwrap(),
        );
        let extended_addr = ExtendedAddr::OrTree(tree_root_hash);
        let bech32_addr = extended_addr.to_cro(network).unwrap();
        assert_eq!(
            bech32_addr,
            "dcro1pe7qg5gshrdl99m9q3ecpzvfr8zuk4h5qqgjyv6y24n80zye42as88x8tg"
        );

        let restored_extended_addr = ExtendedAddr::from_cro(&bech32_addr, network).unwrap();
        assert_eq!(restored_extended_addr, extended_addr);
    }

    #[test]
    fn should_be_correct_hex_address() {
        let mut tree_root_hash = [0; 32];
        tree_root_hash.copy_from_slice(
            &hex::decode("0e7c045110b8dbf29765047380898919c5cb56f400112233445566778899aabb")
                .unwrap(),
        );
        let extended_addr_from_hash = ExtendedAddr::OrTree(tree_root_hash);
        let extended_addr_from_str = ExtendedAddr::from_str(
            "dcro1pe7qg5gshrdl99m9q3ecpzvfr8zuk4h5qqgjyv6y24n80zye42as88x8tg",
        )
        .unwrap();
        assert_eq!(extended_addr_from_hash, extended_addr_from_str);
    }

    mod from_cro {
        use super::*;

        #[test]
        fn should_return_invalid_network_error_when_prefix_is_incorrect() {
            let result = ExtendedAddr::from_cro(
                "dcro1pe7qg5gshrdl99m9q3ecpzvfr8zuk4h5qqgjyv6y24n80zye42as88x8tg",
                Network::Mainnet,
            );

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), CroAddressError::InvalidNetwork);
        }

        #[test]
        fn should_work_when_prefix_is_correct() {
            let result = ExtendedAddr::from_cro(
                "dcro1pe7qg5gshrdl99m9q3ecpzvfr8zuk4h5qqgjyv6y24n80zye42as88x8tg",
                Network::Devnet,
            );

            assert!(result.is_ok());
        }
    }
}
