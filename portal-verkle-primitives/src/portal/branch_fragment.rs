use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    proof::TrieProof,
    ssz::{SparseVector, TriePath},
    Point, CRS,
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
        self.node.verify(commitment)?;
        // TODO: verify trie proof
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchFragmentNode {
    fragment_index: u8,
    children: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceLock<Point>,
}

impl BranchFragmentNode {
    pub fn new(
        fragment_index: u8,
        children: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    ) -> Self {
        Self {
            fragment_index,
            children,
            commitment: OnceLock::new(),
        }
    }

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
                    CRS::commit_single(index, child.map_to_scalar_field())
                })
                .sum()
        })
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
        if self.children.iter_set_items().any(|c| c.is_zero()) {
            return Err(NodeVerificationError::ZeroChild);
        }
        Ok(())
    }
}
