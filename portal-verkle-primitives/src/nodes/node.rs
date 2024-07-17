use ssz_derive::{Decode, Encode};

use crate::Point;

use super::{
    BranchBundleNode, BranchBundleNodeWithProof, BranchFragmentNode, BranchFragmentNodeWithProof,
    LeafBundleNode, LeafBundleNodeWithProof, LeafFragmentNode, LeafFragmentNodeWithProof,
    NodeVerificationError,
};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[ssz(enum_behaviour = "union")]
pub enum PortalVerkleNode {
    BranchBundle(BranchBundleNode),
    BranchFragment(BranchFragmentNode),
    LeafBundle(LeafBundleNode),
    LeafFragment(LeafFragmentNode),
}

impl PortalVerkleNode {
    pub fn commitment(&self) -> &Point {
        match &self {
            Self::BranchBundle(node) => node.commitment(),
            Self::BranchFragment(node) => node.commitment(),
            Self::LeafBundle(node) => node.commitment(),
            Self::LeafFragment(node) => node.commitment(),
        }
    }

    pub fn verify(&self, commitment: &Point) -> Result<(), NodeVerificationError> {
        match &self {
            Self::BranchBundle(node) => node.verify(commitment),
            Self::BranchFragment(node) => node.verify(commitment),
            Self::LeafBundle(node) => node.verify(commitment),
            Self::LeafFragment(node) => node.verify(commitment),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[ssz(enum_behaviour = "union")]
pub enum PortalVerkleNodeWithProof {
    BranchBundle(BranchBundleNodeWithProof),
    BranchFragment(BranchFragmentNodeWithProof),
    LeafBundle(LeafBundleNodeWithProof),
    LeafFragment(LeafFragmentNodeWithProof),
}

impl PortalVerkleNodeWithProof {
    pub fn commitment(&self) -> &Point {
        match &self {
            Self::BranchBundle(node_with_proof) => node_with_proof.node.commitment(),
            Self::BranchFragment(node_with_proof) => node_with_proof.node.commitment(),
            Self::LeafBundle(node_with_proof) => node_with_proof.node.commitment(),
            Self::LeafFragment(node_with_proof) => node_with_proof.node.commitment(),
        }
    }

    pub fn into_node(self) -> PortalVerkleNode {
        match self {
            Self::BranchBundle(node_with_proof) => {
                PortalVerkleNode::BranchBundle(node_with_proof.node)
            }
            Self::BranchFragment(node_with_proof) => {
                PortalVerkleNode::BranchFragment(node_with_proof.node)
            }
            Self::LeafBundle(node_with_proof) => PortalVerkleNode::LeafBundle(node_with_proof.node),
            Self::LeafFragment(node_with_proof) => {
                PortalVerkleNode::LeafFragment(node_with_proof.node)
            }
        }
    }
}
