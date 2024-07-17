use std::cell::OnceCell;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    ssz::{SparseVector, TriePath, TrieProof},
    Point,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchFragmentNodeWithProof {
    pub node: BranchFragmentNode,
    pub block_hash: B256,
    pub path: TriePath,
    pub proof: TrieProof,
}

impl BranchFragmentNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        _state_root: &B256,
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
pub struct BranchFragmentNode {
    fragment_index: u8,
    children: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceCell<Point>,
}

impl BranchFragmentNode {
    pub fn fragment_index(&self) -> usize {
        self.fragment_index as usize
    }

    pub fn children(&self) -> &SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH> {
        &self.children
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.get_or_init(|| {
            self.children
                .iter_enumerated_set_items()
                .map(|(child_index, child)| {
                    let index = child_index + self.fragment_index() * PORTAL_NETWORK_NODE_WIDTH;
                    DefaultMsm.scalar_mul(index, child.map_to_scalar_field())
                })
                .sum()
        })
    }
}
