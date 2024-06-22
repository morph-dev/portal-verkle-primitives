use banderwagon::Element;
use derive_more::{AsRef, Deref, Index};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

use crate::constants::VERKLE_NODE_WIDTH;

const PEDERSEN_SEED: &[u8] = b"eth_verkle_oct_2021";

/// The CRS (Common Reference String) used as a bases for Verkle trie.
pub static CRS: Lazy<Bases> = Lazy::new(Bases::new);

#[derive(AsRef, Deref, Index)]
pub struct Bases([Element; VERKLE_NODE_WIDTH]);

impl Bases {
    fn new() -> Self {
        let mut generated_elements = 0;
        let mut elements = [Element::zero(); VERKLE_NODE_WIDTH];

        for i in 0u64.. {
            if generated_elements == elements.len() {
                break;
            }

            let hash = Sha256::new_with_prefix(PEDERSEN_SEED)
                .chain_update(i.to_be_bytes())
                .finalize();

            if let Some(p) = banderwagon::try_reduce_to_element(&hash) {
                elements[generated_elements] = p;
                generated_elements += 1;
            }
        }

        Self(elements)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::B256;
    use ark_serialize::Valid;
    use banderwagon::CanonicalSerialize;

    use crate::utils::serialize_to_b256;

    use super::*;

    // Taken from:
    // https://github.com/crate-crypto/go-ipa/blob/b1e8a79f509c5dd26b44d64c5f4aff67d7e69ed0/ipa/ipa_test.go#L210

    const FIRST_POINT: &str = "0x01587ad1336675eb912550ec2a28eb8923b824b490dd2ba82e48f14590a298a0";
    const LAST_POINT: &str = "0x3de2be346b539395b0c0de56a5ccca54a317f1b5c80107b0802af9a62276a4d8";
    const ALL_POINTS_SHA: &str =
        "0x1fcaea10bf24f750200e06fa473c76ff0468007291fa548e2d99f09ba9256fdb";

    #[test]
    fn first_point() -> anyhow::Result<()> {
        assert_eq!(serialize_to_b256(&CRS[0])?, B256::from_str(FIRST_POINT)?);
        Ok(())
    }

    #[test]
    fn last_point() -> anyhow::Result<()> {
        assert_eq!(serialize_to_b256(&CRS[255])?, B256::from_str(LAST_POINT)?);
        Ok(())
    }

    #[test]
    fn all_points_hash() -> anyhow::Result<()> {
        let mut hasher = Sha256::new();
        for p in CRS.iter() {
            p.serialize_compressed(&mut hasher)?;
        }

        assert_eq!(
            B256::from_slice(&hasher.finalize()),
            B256::from_str(ALL_POINTS_SHA)?
        );
        Ok(())
    }

    #[test]
    fn valid() {
        for (i, p) in CRS.iter().enumerate() {
            assert!(
                p.check().is_ok(),
                "point {p:?} at index {i} should be valid"
            );
        }
    }
}
