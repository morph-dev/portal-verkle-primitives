use std::cell::OnceCell;

use alloy_primitives::B256;
use ssz_derive::{Decode, Encode};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    proof::BundleProof,
    ssz::{SparseVector, TriePath, TrieProof},
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
pub struct BranchBundleNode {
    fragments: SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH>,
    bundle_proof: BundleProof,
    #[ssz(skip_serializing, skip_deserializing)]
    commitment: OnceCell<Point>,
}

impl BranchBundleNode {
    pub fn fragments(&self) -> &SparseVector<Point, PORTAL_NETWORK_NODE_WIDTH> {
        &self.fragments
    }

    pub fn bundle_proof(&self) -> &BundleProof {
        &self.bundle_proof
    }

    pub fn verify_bundle_proof(&self) -> Result<(), NodeVerificationError> {
        // TODO: add implementataion
        Ok(())
    }

    pub fn commitment(&self) -> &Point {
        self.commitment
            .get_or_init(|| self.fragments.iter_set_items().sum())
    }
}
