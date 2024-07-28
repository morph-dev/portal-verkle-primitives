use alloy_primitives::B256;

use crate::{
    constants::{PORTAL_NETWORK_NODE_WIDTH, VERKLE_NODE_WIDTH},
    portal::{
        BranchBundleNode, BranchBundleNodeWithProof, BranchFragmentNode,
        BranchFragmentNodeWithProof,
    },
    proof::{lagrange_basis::LagrangeBasis, BundleProof, MultiProof, ProverMultiQuery},
    ssz::{SparseVector, TriePathWithCommitments},
    utils::{array_long, array_short},
    Point, ScalarField, CRS,
};

use super::{branch::BranchNode, commitment::Commitment};

struct FragmentInfo<'a> {
    fragment_index: u8,
    commitment: Point,
    children: [&'a Commitment; PORTAL_NETWORK_NODE_WIDTH],
}

impl<'a> FragmentInfo<'a> {
    fn new(fragment_index: u8, children: [&'a Commitment; PORTAL_NETWORK_NODE_WIDTH]) -> Self {
        let commitment = CRS::commit_sparse(
            &children
                .iter()
                .enumerate()
                .filter_map(|(fragment_child_index, child)| {
                    if child.is_zero() {
                        None
                    } else {
                        let child_index =
                            get_child_index(fragment_index, fragment_child_index as u8);
                        Some((child_index, child.to_scalar()))
                    }
                })
                .collect::<Vec<_>>(),
        );
        Self {
            fragment_index,
            commitment,
            children,
        }
    }

    fn to_lagrange_basis(&self) -> LagrangeBasis {
        LagrangeBasis::new(array_long(|child_index| {
            if get_fragment_index(child_index) == self.fragment_index {
                self.children[get_fragment_child_index(child_index) as usize].to_scalar()
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
    node: &'a BranchNode,
    fragments: [FragmentInfo<'a>; PORTAL_NETWORK_NODE_WIDTH],
    trie_path: TriePathWithCommitments,
    trie_path_prover_multiquery: ProverMultiQuery,
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
                let child_index = get_child_index(fragment_index, fragment_child_index);
                node.get_child(child_index).commitment()
            });
            FragmentInfo::new(fragment_index, fragment_children)
        });

        let trie_path_with_commitments = trie_path
            .iter()
            .map(|(branch_node, child_index)| (branch_node.commitment().to_point(), *child_index))
            .collect::<Vec<_>>()
            .try_into()?;

        let mut trie_path_prover_multiquery = ProverMultiQuery::new();
        trie_path_prover_multiquery.add_trie_path_proof(trie_path.iter().map(
            |(branch_node, child_index)| {
                (
                    branch_node.commitment().to_point(),
                    branch_node.to_lagrange_basis(),
                    *child_index,
                )
            },
        ));
        Ok(Self {
            node,
            fragments,
            trie_path: trie_path_with_commitments,
            trie_path_prover_multiquery,
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
            let openings = (0..VERKLE_NODE_WIDTH).filter_map(|child_index| {
                if fragment.fragment_index == get_fragment_index(child_index as u8) {
                    None
                } else {
                    Some(child_index as u8)
                }
            });
            bundle_multiquery.add_vector(
                fragment.commitment.clone(),
                fragment.to_lagrange_basis(),
                openings,
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
            multiproof: MultiProof::create_portal_network_proof(
                self.trie_path_prover_multiquery.clone(),
            ),
        }
    }

    pub fn fragment_node(&self, fragment_index: u8) -> BranchFragmentNode {
        let fragment_children = self.fragments[fragment_index as usize]
            .children
            .each_ref()
            .map(|commitment| {
                if commitment.is_zero() {
                    None
                } else {
                    Some(commitment.to_point())
                }
            });
        BranchFragmentNode::new(fragment_index, SparseVector::new(fragment_children))
    }

    pub fn fragment_node_with_proof(
        &self,
        fragment_index: u8,
        block_hash: B256,
    ) -> BranchFragmentNodeWithProof {
        let bundle_commitment = self.node.commitment().to_point();

        let mut multiquery = self.trie_path_prover_multiquery.clone();
        multiquery.add_vector(
            bundle_commitment.clone(),
            self.node.to_lagrange_basis(),
            array_short(|fragment_child_index| {
                get_child_index(fragment_index, fragment_child_index)
            }),
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

fn get_child_index(fragment_index: u8, fragment_child_index: u8) -> u8 {
    fragment_child_index + fragment_index * PORTAL_NETWORK_NODE_WIDTH as u8
}

fn get_fragment_index(child_index: u8) -> u8 {
    child_index / PORTAL_NETWORK_NODE_WIDTH as u8
}

fn get_fragment_child_index(child_index: u8) -> u8 {
    child_index % PORTAL_NETWORK_NODE_WIDTH as u8
}
