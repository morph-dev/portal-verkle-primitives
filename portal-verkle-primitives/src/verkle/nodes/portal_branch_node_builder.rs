use alloy_primitives::B256;
use itertools::Itertools;

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    portal::{
        BranchBundleNode, BranchBundleNodeWithProof, BranchFragmentNode,
        BranchFragmentNodeWithProof,
    },
    proof::{lagrange_basis::LagrangeBasis, BundleProof, MultiProof, ProverMultiQuery},
    ssz::{SparseVector, TriePathWithCommitments},
    utils::{array_long, array_short, branch_utils},
    Point, ScalarField, CRS,
};

use super::{branch::BranchNode, commitment::Commitment};

struct FragmentInfo<'a> {
    fragment_index: u8,
    commitment: Point,
    children: SparseVector<&'a Commitment, PORTAL_NETWORK_NODE_WIDTH>,
}

impl<'a> FragmentInfo<'a> {
    fn new(
        fragment_index: u8,
        children: SparseVector<&'a Commitment, PORTAL_NETWORK_NODE_WIDTH>,
    ) -> Self {
        let commitment = CRS::commit_sparse(
            &children
                .iter_enumerated_set_items()
                .map(|(fragment_child_index, child)| {
                    let child_index =
                        branch_utils::child_index(fragment_index, fragment_child_index as u8);
                    (child_index, child.to_scalar())
                })
                .collect_vec(),
        );
        Self {
            fragment_index,
            commitment,
            children,
        }
    }

    fn to_lagrange_basis(&self) -> LagrangeBasis {
        LagrangeBasis::new(array_long(|child_index| {
            if branch_utils::fragment_index(child_index) == self.fragment_index {
                self.children[branch_utils::fragment_child_index(child_index) as usize]
                    .map(Commitment::to_scalar)
                    .unwrap_or_else(ScalarField::zero)
            } else {
                ScalarField::zero()
            }
        }))
    }

    fn is_zero(&self) -> bool {
        self.commitment.is_zero()
    }
}
pub struct PortalBranchNodeBuilder<'a> {
    branch_node: &'a BranchNode,
    fragments: [FragmentInfo<'a>; PORTAL_NETWORK_NODE_WIDTH],
    trie_path: TriePathWithCommitments,
    trie_path_multiquery: ProverMultiQuery,
}

impl<'a> PortalBranchNodeBuilder<'a> {
    pub fn new(node: &'a BranchNode, trie_path: &[(&BranchNode, u8)]) -> Result<Self, String> {
        if node.depth() != trie_path.len() {
            return Err(format!(
                "The length of the trie_path ({}) doesn't match node's depth ({})",
                trie_path.len(),
                node.depth(),
            ));
        }

        let fragments = array_short(|fragment_index| {
            let fragment_children = array_short(|fragment_child_index| {
                let child_index = branch_utils::child_index(fragment_index, fragment_child_index);
                let commitment = node.get_child(child_index).commitment();
                if commitment.is_zero() {
                    None
                } else {
                    Some(commitment)
                }
            });
            FragmentInfo::new(fragment_index, SparseVector::new(fragment_children))
        });

        let mut trie_path_multiquery = ProverMultiQuery::new();
        trie_path_multiquery.add_trie_path_proof(trie_path.iter().map(
            |(branch_node, child_index)| {
                (
                    branch_node.commitment().to_point(),
                    branch_node.to_lagrange_basis(),
                    *child_index,
                )
            },
        ));
        Ok(Self {
            branch_node: node,
            fragments,
            trie_path: trie_path.iter().cloned().collect(),
            trie_path_multiquery,
        })
    }

    pub fn bundle_node(&self) -> BranchBundleNode {
        let fragment_commitments = self.fragments.each_ref().map(|fragment| {
            if fragment.is_zero() {
                None
            } else {
                Some(fragment.commitment.clone())
            }
        });

        let mut bundle_multiquery = ProverMultiQuery::new();
        for fragment in &self.fragments {
            if fragment.is_zero() {
                continue;
            }
            bundle_multiquery.add_vector(
                fragment.commitment.clone(),
                fragment.to_lagrange_basis(),
                branch_utils::openings_for_bundle(fragment.fragment_index),
            );
        }

        BranchBundleNode::new(
            SparseVector::new(fragment_commitments),
            BundleProof::new(MultiProof::create_portal_network_proof(bundle_multiquery)),
        )
    }

    pub fn bundle_node_with_proof(&self, block_hash: B256) -> BranchBundleNodeWithProof {
        BranchBundleNodeWithProof {
            node: self.bundle_node(),
            block_hash,
            trie_path: self.trie_path.clone(),
            multiproof: MultiProof::create_portal_network_proof(self.trie_path_multiquery.clone()),
        }
    }

    pub fn fragment_commitment(&self, fragment_index: u8) -> &Point {
        &self.fragments[fragment_index as usize].commitment
    }

    pub fn fragment_node(&self, fragment_index: u8) -> BranchFragmentNode {
        let fragment = &self.fragments[fragment_index as usize];
        let fragment_children = fragment
            .children
            .each_ref()
            .map(|commitment| commitment.map(Commitment::to_point));
        BranchFragmentNode::new(fragment_index, SparseVector::new(fragment_children))
    }

    pub fn fragment_node_with_proof(
        &self,
        fragment_index: u8,
        block_hash: B256,
    ) -> BranchFragmentNodeWithProof {
        let bundle_commitment = self.branch_node.commitment().to_point();

        let mut multiquery = self.trie_path_multiquery.clone();
        multiquery.add_vector(
            bundle_commitment.clone(),
            self.branch_node.to_lagrange_basis(),
            branch_utils::openings(fragment_index),
        );

        BranchFragmentNodeWithProof {
            node: self.fragment_node(fragment_index),
            block_hash,
            bundle_commitment,
            trie_path: self.trie_path.clone(),
            multiproof: MultiProof::create_portal_network_proof(multiquery),
        }
    }
}
