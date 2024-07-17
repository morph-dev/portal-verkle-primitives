use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::{
        LEAF_C1_INDEX, LEAF_C2_INDEX, LEAF_MARKER_INDEX, LEAF_STEM_INDEX, PORTAL_NETWORK_NODE_WIDTH,
    },
    msm::{DefaultMsm, MultiScalarMultiplicator},
    proof::BundleProof,
    ssz::{SparseVector, TrieProof},
    Point, ScalarField, Stem,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafBundleNodeWithProof {
    pub node: LeafBundleNode,
    pub block_hash: B256,
    pub proof: TrieProof,
}

impl LeafBundleNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        _state_root: &B256,
    ) -> Result<(), NodeVerificationError> {
        self.node.verify(commitment)?;
        // TODO: verify trie proof
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafBundleNode {
    marker: u64,
    stem: Stem,
    fragments: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    bundle_proof: BundleProof,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceLock<Point>,
}

impl LeafBundleNode {
    pub fn marker(&self) -> u64 {
        self.marker
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    pub fn fragments(&self) -> &SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH> {
        &self.fragments
    }

    pub fn bundle_proof(&self) -> &BundleProof {
        &self.bundle_proof
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.get_or_init(|| {
            let (first_half, second_half) = self.fragments.split_at(PORTAL_NETWORK_NODE_WIDTH / 2);
            let sum_of_optional_points = |fragments: &[Option<Point>]| {
                fragments
                    .iter()
                    .flat_map(|fragment| fragment.as_ref())
                    .sum::<Point>()
            };
            let c1 = sum_of_optional_points(first_half);
            let c2 = sum_of_optional_points(second_half);
            DefaultMsm.commit_sparse(&[
                (LEAF_MARKER_INDEX, ScalarField::from(self.marker)),
                (LEAF_STEM_INDEX, ScalarField::from(&self.stem)),
                (LEAF_C1_INDEX, c1.map_to_scalar_field()),
                (LEAF_C2_INDEX, c2.map_to_scalar_field()),
            ])
        })
    }

    pub fn verify_bundle_proof(&self) -> Result<(), NodeVerificationError> {
        // TODO: add implementataion
        Ok(())
    }

    pub fn verify(&self, commitment: &Point) -> Result<(), NodeVerificationError> {
        if commitment != self.commitment() {
            return Err(NodeVerificationError::wrong_commitment(
                commitment,
                self.commitment(),
            ));
        }
        if self.commitment().is_zero() {
            return Err(NodeVerificationError::ZeroCommitment);
        }
        if self.fragments.len() == 0 {
            return Err(NodeVerificationError::NoFragments);
        }
        if self.fragments.iter_set_items().any(|c| c.is_zero()) {
            return Err(NodeVerificationError::ZeroChild);
        }
        self.verify_bundle_proof()?;
        Ok(())
    }
}
