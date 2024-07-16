use std::cell::OnceCell;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    ssz::{SparseVector, TrieProof},
    Point, Stem, TrieValue, TrieValueSplit,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafFragmentNodeWithProof {
    pub node: LeafFragmentNode,
    pub block_hash: B256,
    pub proof: TrieProof,
}

impl LeafFragmentNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        _state_root: &B256,
        _stem: &Stem,
    ) -> Result<(), NodeVerificationError> {
        if commitment != self.node.commitment() {
            return Err(NodeVerificationError::new_wrong_commitment(
                commitment,
                self.node.commitment(),
            ));
        }
        // TODO: add implementataion
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct LeafFragmentNode {
    fragment_index: u8,
    children: SparseVector<TrieValue, PORTAL_NETWORK_NODE_WIDTH>,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceCell<Point>,
}

impl LeafFragmentNode {
    pub fn fragment_index(&self) -> usize {
        self.fragment_index as usize
    }

    pub fn children(&self) -> &SparseVector<TrieValue, PORTAL_NETWORK_NODE_WIDTH> {
        &self.children
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.get_or_init(|| {
            self.children
                .iter_enumerated_set_items()
                .map(|(child_index, child)| {
                    let (low_index, high_index) = self.bases_indices(child_index);
                    let (low_value, high_value) = child.split();
                    DefaultMsm.scalar_mul(low_index, low_value)
                        + DefaultMsm.scalar_mul(high_index, high_value)
                })
                .sum()
        })
    }

    /// Returns the bases indices that correspond to the child index.
    fn bases_indices(&self, child_index: usize) -> (usize, usize) {
        let fragment_starting_index =
            self.fragment_index() % (PORTAL_NETWORK_NODE_WIDTH / 2) * 2 * PORTAL_NETWORK_NODE_WIDTH;
        let low_index = fragment_starting_index + 2 * child_index;
        let high_index = low_index + 1;
        (low_index, high_index)
    }
}
