use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    proof::{MultiProof, VerifierMultiQuery},
    ssz::{SparseVector, TriePathWithCommitments},
    Point, ScalarField, CRS,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchFragmentNodeWithProof {
    pub node: BranchFragmentNode,
    pub block_hash: B256,
    pub bundle_commitment: Point,
    pub trie_path: TriePathWithCommitments,
    pub multiproof: MultiProof,
}

impl BranchFragmentNodeWithProof {
    pub fn verify(
        &self,
        commitment: &Point,
        state_root: &B256,
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

        // 3.1. Verify trie path
        multi_query.add_trie_path_proof(self.trie_path.clone(), &self.bundle_commitment);

        // 3.2. Verify children openings to bundle commitment
        multi_query.add_for_commitment(
            &self.bundle_commitment,
            self.node
                .children
                .iter()
                .enumerate()
                .map(|(child_index, child)| {
                    (
                        child_index as u8,
                        child
                            .as_ref()
                            .map(Point::map_to_scalar_field)
                            .unwrap_or_else(ScalarField::zero),
                    )
                }),
        );

        if self.multiproof.verify_portal_network_proof(multi_query) {
            Ok(())
        } else {
            Err(NodeVerificationError::InvalidMultiPointProof)
        }
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

    pub fn fragment_index(&self) -> u8 {
        self.fragment_index
    }

    pub fn children(&self) -> &SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH> {
        &self.children
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.get_or_init(|| {
            self.children
                .iter_enumerated_set_items()
                .map(|(child_index, child)| {
                    let index =
                        child_index as u8 + self.fragment_index * PORTAL_NETWORK_NODE_WIDTH as u8;
                    CRS::commit_single(index, child.map_to_scalar_field())
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
        if self.children.iter_set_items().any(|c| c.is_zero()) {
            return Err(NodeVerificationError::ZeroChild);
        }
        if self.fragment_index >= PORTAL_NETWORK_NODE_WIDTH as u8 {
            return Err(NodeVerificationError::InvalidFragmentIndex(
                self.fragment_index,
            ));
        }
        Ok(())
    }
}
