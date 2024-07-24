use std::sync::OnceLock;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    proof::{BundleProof, TrieProof},
    ssz::{SparseVector, TriePath},
    Point,
};

use super::NodeVerificationError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BranchBundleNodeWithProof {
    pub node: BranchBundleNode,
    pub block_hash: B256,
    pub path: TriePath,
    pub proof: TrieProof,
}

impl BranchBundleNodeWithProof {
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
