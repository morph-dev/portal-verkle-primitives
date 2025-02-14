use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::{LEAF_MARKER_INDEX, LEAF_STEM_INDEX, PORTAL_NETWORK_NODE_WIDTH},
    proof::{MultiProof, VerifierMultiQuery},
    ssz::{SparseVector, TriePathCommitments},
    utils::{array_short, leaf_utils},
    Point, Stem, TrieValue, TrieValueSplit, CRS,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafFragmentNodeWithProof {
    pub node: LeafFragmentNode,
    /// The hash of a block.
    pub block_hash: B256,
    /// The marker of the leaf bundle node.
    pub marker: u64,
    /// The commitment of the leaf bundle node.
    pub bundle_commitment: Point,
    /// The c1 or c2 commitment that corresponds to the fragment node.
    pub suffix_commitment: Point,
    pub trie_path: TriePathCommitments,
    pub multiproof: MultiProof,
}

impl LeafFragmentNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        state_root: &B256,
        stem: &Stem,
    ) -> Result<(), NodeVerificationError> {
        // 1. Verify node
        self.node.verify(commitment)?;

        // 2. Verify State root
        let root = B256::from(self.trie_path.root().unwrap_or(&self.bundle_commitment));
        if state_root != &root {
            return Err(NodeVerificationError::new_wrong_root(*state_root, root));
        }

        // 3. Verify multiproof
        let mut multi_query = VerifierMultiQuery::new();

        // 3.1 Verify trie proof
        multi_query
            .add_trie_path_proof(self.trie_path.zip_with_stem(stem), &self.bundle_commitment);

        // 3.2 Verify [marker, stem, c1 /c2] openings to bundle commitment
        multi_query.add_for_commitment(
            &self.bundle_commitment,
            [
                (LEAF_MARKER_INDEX, self.marker.into()),
                (LEAF_STEM_INDEX, stem.into()),
                (
                    leaf_utils::leaf_suffix_index(self.node.fragment_index),
                    self.suffix_commitment.map_to_scalar_field(),
                ),
            ],
        );

        // 3.3 Verify children openings to suffix commitment (c1 or c2)
        multi_query.add_for_commitment(
            &self.suffix_commitment,
            array_short(|child_index| {
                let [low_index, high_index] =
                    leaf_utils::suffix_indices(self.node.fragment_index, child_index);
                let (low_value, high_value) = self.node.children[child_index as usize].split();
                [(low_index, low_value), (high_index, high_value)]
            })
            .into_iter()
            .flatten(),
        );

        if self.multiproof.verify_portal_network_proof(multi_query) {
            Ok(())
        } else {
            Err(NodeVerificationError::InvalidMultiPointProof)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafFragmentNode {
    fragment_index: u8,
    children: SparseVector<TrieValue, PORTAL_NETWORK_NODE_WIDTH>,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceLock<Point>,
}

impl LeafFragmentNode {
    pub fn new(
        fragment_index: u8,
        children: SparseVector<TrieValue, PORTAL_NETWORK_NODE_WIDTH>,
    ) -> Self {
        Self {
            fragment_index,
            children,
            commitment: OnceLock::new(),
        }
    }

    pub fn fragment_index(&self) -> u8 {
        self.fragment_index
    }

    pub fn children(&self) -> &SparseVector<TrieValue, PORTAL_NETWORK_NODE_WIDTH> {
        &self.children
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.get_or_init(|| {
            self.children
                .iter_enumerated_set_items()
                .map(|(child_index, child)| {
                    let [low_index, high_index] =
                        leaf_utils::suffix_indices(self.fragment_index, child_index as u8);
                    let (low_value, high_value) = child.split();
                    CRS::commit_sparse(&[(low_index, low_value), (high_index, high_value)])
                })
                .sum()
        })
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
        if self.fragment_index >= PORTAL_NETWORK_NODE_WIDTH as u8 {
            return Err(NodeVerificationError::InvalidFragmentIndex(
                self.fragment_index,
            ));
        }
        Ok(())
    }
}
