use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::{PORTAL_NETWORK_NODE_WIDTH, VERKLE_NODE_WIDTH},
    proof::{BundleProof, MultiProof, VerifierMultiQuery},
    ssz::{SparseVector, TriePathWithCommitments},
    Point, ScalarField,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchBundleNodeWithProof {
    pub node: BranchBundleNode,
    pub block_hash: B256,
    pub trie_path: TriePathWithCommitments,
    pub multiproof: MultiProof,
}

impl BranchBundleNodeWithProof {
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
        multi_query.add_trie_path_proof(self.trie_path.clone(), commitment);

        if self.multiproof.verify_portal_network_proof(multi_query) {
            Ok(())
        } else {
            Err(NodeVerificationError::InvalidMultiPointProof)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchBundleNode {
    fragments: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    bundle_proof: BundleProof,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceLock<Point>,
}

impl BranchBundleNode {
    pub fn new(
        fragments: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
        bundle_proof: BundleProof,
    ) -> Self {
        Self {
            fragments,
            bundle_proof,
            commitment: OnceLock::new(),
        }
    }

    pub fn fragments(&self) -> &SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH> {
        &self.fragments
    }

    pub fn bundle_proof(&self) -> &BundleProof {
        &self.bundle_proof
    }

    pub fn commitment(&self) -> &Point {
        self.commitment
            .get_or_init(|| self.fragments.iter_set_items().sum())
    }

    pub fn verify_bundle_proof(&self) -> Result<(), NodeVerificationError> {
        let mut multiquery = VerifierMultiQuery::new();
        for (fragment_index, fragment_commitment) in self.fragments.iter_enumerated_set_items() {
            multiquery.add_for_commitment(
                fragment_commitment,
                (0..VERKLE_NODE_WIDTH)
                    .filter(|child_index| child_index / PORTAL_NETWORK_NODE_WIDTH != fragment_index)
                    .map(|child_index| (child_index as u8, ScalarField::zero())),
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
