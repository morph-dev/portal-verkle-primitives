use std::array;

use banderwagon::msm::MSMPrecompWnaf;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

use crate::{constants::VERKLE_NODE_WIDTH, Point, ScalarField};

const PEDERSEN_SEED: &[u8] = b"eth_verkle_oct_2021";

static PRECOMP_WNAF_WINDOW_SIZE: usize = 12;

/// The CRS (Common Reference String) that contains Verkle trie relevant constants
pub struct CRS {
    bases: [Point; VERKLE_NODE_WIDTH],
    generator: Point,
    /// Precomputed wNAF (w-ary non-adjacent form) tables for efficient scalar multiplication.
    wnaf_precomp: MSMPrecompWnaf,
}

static INSTANCE: Lazy<CRS> = Lazy::new(CRS::new);

impl CRS {
    fn new() -> Self {
        let mut generated_elements = 0;
        let mut elements: [Point; VERKLE_NODE_WIDTH] = array::from_fn(|_| Point::zero());

        for i in 0u64.. {
            if generated_elements == elements.len() {
                break;
            }

            let hash = Sha256::new_with_prefix(PEDERSEN_SEED)
                .chain_update(i.to_be_bytes())
                .finalize();

            if let Some(p) = banderwagon::try_reduce_to_element(&hash) {
                elements[generated_elements] = Point::new(p);
                generated_elements += 1;
            }
        }

        let wnaf_precomp = MSMPrecompWnaf::new(
            &elements.each_ref().map(|point| point.inner()),
            PRECOMP_WNAF_WINDOW_SIZE,
        );

        Self {
            bases: elements,
            generator: Point::prime_subgroup_generator(),
            wnaf_precomp,
        }
    }

    pub fn bases() -> &'static [Point; VERKLE_NODE_WIDTH] {
        &INSTANCE.bases
    }

    pub fn generator() -> &'static Point {
        &INSTANCE.generator
    }

    /// Commit to a full vector.
    pub fn commit(scalars: &[ScalarField; VERKLE_NODE_WIDTH]) -> Point {
        // Preliminary benchmarks indicate that the parallel version is faster
        // for vectors of length 64 or more
        let scalars = scalars.each_ref().map(|scalar| scalar.inner());
        if scalars.len() >= 64 {
            Point::new(INSTANCE.wnaf_precomp.mul_par(&scalars))
        } else {
            Point::new(INSTANCE.wnaf_precomp.mul(&scalars))
        }
    }

    /// Single scalar multiplication.
    pub fn commit_single(index: u8, scalar: ScalarField) -> Point {
        if scalar.is_zero() {
            Point::zero()
        } else {
            Point::new(
                INSTANCE
                    .wnaf_precomp
                    .mul_index(scalar.inner(), index as usize),
            )
        }
    }

    /// Commit to sparse set of scalars.
    pub fn commit_sparse(scalars: &[(u8, ScalarField)]) -> Point {
        // TODO: consider if 64 is good value
        if scalars.len() >= 64 {
            let mut dense: [ScalarField; VERKLE_NODE_WIDTH] =
                array::from_fn(|_| ScalarField::zero());
            for (index, value) in scalars {
                dense[*index as usize] = value.clone();
            }
            Self::commit(&dense)
        } else {
            scalars
                .iter()
                .map(|(index, value)| Self::commit_single(*index, value.clone()))
                .sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::B256;
    use ark_serialize::Valid;

    use super::*;

    // Taken from:
    // https://github.com/crate-crypto/go-ipa/blob/b1e8a79f509c5dd26b44d64c5f4aff67d7e69ed0/ipa/ipa_test.go#L210

    const FIRST_POINT: &str = "0x01587ad1336675eb912550ec2a28eb8923b824b490dd2ba82e48f14590a298a0";
    const LAST_POINT: &str = "0x3de2be346b539395b0c0de56a5ccca54a317f1b5c80107b0802af9a62276a4d8";
    const ALL_POINTS_SHA: &str =
        "0x1fcaea10bf24f750200e06fa473c76ff0468007291fa548e2d99f09ba9256fdb";

    #[test]
    fn first_point() -> anyhow::Result<()> {
        assert_eq!(B256::from(&CRS::bases()[0]), B256::from_str(FIRST_POINT)?);
        Ok(())
    }

    #[test]
    fn last_point() -> anyhow::Result<()> {
        assert_eq!(B256::from(&CRS::bases()[255]), B256::from_str(LAST_POINT)?);
        Ok(())
    }

    #[test]
    fn all_points_hash() -> anyhow::Result<()> {
        let mut hasher = Sha256::new();
        for p in CRS::bases() {
            hasher.update(B256::from(p));
        }

        assert_eq!(
            B256::from_slice(&hasher.finalize()),
            B256::from_str(ALL_POINTS_SHA)?
        );
        Ok(())
    }

    #[test]
    fn valid() {
        for (i, p) in CRS::bases().iter().enumerate() {
            assert!(
                p.inner().check().is_ok(),
                "point {p:?} at index {i} should be valid"
            );
        }
    }
}
