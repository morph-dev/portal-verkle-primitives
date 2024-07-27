use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::{
        LEAF_C1_INDEX, LEAF_C2_INDEX, LEAF_MARKER_INDEX, LEAF_STEM_INDEX,
        PORTAL_NETWORK_NODE_WIDTH, VERKLE_NODE_WIDTH,
    },
    proof::{BundleProof, MultiProof, VerifierMultiQuery},
    ssz::{SparseVector, TriePathCommitments},
    Point, ScalarField, Stem, CRS,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafBundleNodeWithProof {
    pub node: LeafBundleNode,
    pub block_hash: B256,
    pub trie_path: TriePathCommitments,
    pub multiproof: MultiProof,
}

impl LeafBundleNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        state_root: &B256,
    ) -> Result<(), NodeVerificationError> {
        // 1. Verify node
        self.node.verify(commitment)?;

        // 2. Verify State root
        let root = B256::from(self.trie_path.root().unwrap_or(commitment));
        if state_root != &root {
            return Err(NodeVerificationError::new_wrong_root(*state_root, root));
        }

        // 3. Verify multiproof
        let mut multi_query = VerifierMultiQuery::new();
        // Verify trie path
        multi_query.add_trie_path_proof(self.trie_path.zip_with_stem(self.node.stem()), commitment);

        if self.multiproof.verify_portal_network_proof(multi_query) {
            Ok(())
        } else {
            Err(NodeVerificationError::InvalidMultiPointProof)
        }
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
    pub fn new(
        marker: u64,
        stem: Stem,
        fragments: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
        bundle_proof: BundleProof,
    ) -> Self {
        Self {
            marker,
            stem,
            fragments,
            bundle_proof,
            commitment: OnceLock::new(),
        }
    }

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
            let (c1, c2) = self.c1_c2();
            CRS::commit_sparse(&[
                (LEAF_MARKER_INDEX, ScalarField::from(self.marker)),
                (LEAF_STEM_INDEX, ScalarField::from(&self.stem)),
                (LEAF_C1_INDEX, c1.map_to_scalar_field()),
                (LEAF_C2_INDEX, c2.map_to_scalar_field()),
            ])
        })
    }

    pub fn c1_c2(&self) -> (Point, Point) {
        let (first_half, second_half) = self.fragments.split_at(PORTAL_NETWORK_NODE_WIDTH / 2);
        let sum_of_optional_points = |fragments: &[Option<Point>]| {
            fragments
                .iter()
                .flat_map(|fragment| fragment.as_ref())
                .sum::<Point>()
        };
        let c1 = sum_of_optional_points(first_half);
        let c2 = sum_of_optional_points(second_half);
        (c1, c2)
    }

    pub fn verify_bundle_proof(&self) -> Result<(), NodeVerificationError> {
        let mut multiquery = VerifierMultiQuery::new();
        for (fragment_index, fragment_commitment) in self.fragments.iter_enumerated_set_items() {
            multiquery.add_for_commitment(
                fragment_commitment,
                (0..VERKLE_NODE_WIDTH)
                    .filter(|child_index| child_index / PORTAL_NETWORK_NODE_WIDTH != fragment_index)
                    .flat_map(|child_index| {
                        let suffix_child_index = (child_index % (VERKLE_NODE_WIDTH / 2)) as u8;
                        let low_index = suffix_child_index * 2;
                        let high_index = low_index + 1;
                        [
                            (low_index, ScalarField::zero()),
                            (high_index, ScalarField::zero()),
                        ]
                    }),
            );
        }
        if self.bundle_proof.verify_portal_network_proof(multiquery) {
            Ok(())
        } else {
            Err(NodeVerificationError::InvalidBundleProof)
        }
    }

    pub fn verify(&self, commitment: &Point) -> Result<(), NodeVerificationError> {
        if commitment != self.commitment() {
            return Err(NodeVerificationError::new_wrong_commitment(
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
