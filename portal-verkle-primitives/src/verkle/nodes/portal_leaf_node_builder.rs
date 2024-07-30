use alloy_primitives::B256;
use itertools::Itertools;

use crate::{
    constants::{
        LEAF_C1_INDEX, LEAF_C2_INDEX, LEAF_MARKER_INDEX, LEAF_STEM_INDEX, PORTAL_NETWORK_NODE_WIDTH,
    },
    portal::{
        LeafBundleNode, LeafBundleNodeWithProof, LeafFragmentNode, LeafFragmentNodeWithProof,
    },
    proof::{lagrange_basis::LagrangeBasis, BundleProof, MultiProof, ProverMultiQuery},
    ssz::{SparseVector, TriePathCommitments},
    utils::{array_long_const, array_short, leaf_utils},
    verkle::PathToLeaf,
    Point, ScalarField, TrieValue, CRS,
};

use super::{commitment::Commitment, leaf::LeafNode};

struct FragmentInfo<'a> {
    fragment_index: u8,
    commitment: Point,
    suffix_commitment: &'a Commitment,
    values: SparseVector<&'a TrieValue, PORTAL_NETWORK_NODE_WIDTH>,
}

impl<'a> FragmentInfo<'a> {
    fn new(
        fragment_index: u8,
        suffix_commitment: &'a Commitment,
        values: SparseVector<&'a TrieValue, PORTAL_NETWORK_NODE_WIDTH>,
    ) -> Self {
        let commitment = CRS::commit_sparse(
            &values
                .iter_enumerated_set_items()
                .flat_map(|(fragment_child_index, value)| {
                    let [low_index, high_index] =
                        leaf_utils::suffix_indices(fragment_index, fragment_child_index as u8);
                    let (low_value, high_value) = value.split();
                    vec![(low_index, low_value), (high_index, high_value)]
                })
                .collect_vec(),
        );
        Self {
            fragment_index,
            commitment,
            suffix_commitment,
            values,
        }
    }

    fn to_lagrange_basis(&self) -> LagrangeBasis {
        let mut scalars = array_long_const(ScalarField::zero());
        for (fragment_child_index, value) in self.values.iter_enumerated_set_items() {
            let [low_index, high_index] =
                leaf_utils::suffix_indices(self.fragment_index, fragment_child_index as u8);
            let (low_value, high_value) = value.split();
            scalars[low_index as usize] = low_value;
            scalars[high_index as usize] = high_value;
        }
        LagrangeBasis::new(scalars)
    }

    fn is_zero(&self) -> bool {
        self.commitment.is_zero()
    }
}

pub struct PortalLeafNodeBuilder<'a> {
    leaf_node: &'a LeafNode,
    fragments: [FragmentInfo<'a>; PORTAL_NETWORK_NODE_WIDTH],
    trie_path: TriePathCommitments,
    trie_path_multiquery: ProverMultiQuery,
}

impl<'a> PortalLeafNodeBuilder<'a> {
    pub fn new(path_to_leaf: &PathToLeaf<'a>) -> Self {
        let PathToLeaf {
            trie_path: trie_path_branches,
            leaf,
        } = path_to_leaf;

        let fragments = array_short(|fragment_index| {
            let fragment_values = array_short(|fragment_child_index| {
                let child_index =
                    fragment_child_index + fragment_index * PORTAL_NETWORK_NODE_WIDTH as u8;
                leaf.get(child_index)
            });
            let suffix_commitment = if fragment_index < PORTAL_NETWORK_NODE_WIDTH as u8 / 2 {
                leaf.c1()
            } else {
                leaf.c2()
            };
            FragmentInfo::new(
                fragment_index,
                suffix_commitment,
                SparseVector::new(fragment_values),
            )
        });

        let mut trie_path_multiquery = ProverMultiQuery::new();
        trie_path_multiquery.add_trie_path_proof(trie_path_branches.iter().map(
            |(branch_node, child_index)| {
                (
                    branch_node.commitment().to_point(),
                    branch_node.to_lagrange_basis(),
                    *child_index,
                )
            },
        ));

        Self {
            leaf_node: leaf,
            fragments,
            trie_path: trie_path_branches.iter().cloned().collect(),
            trie_path_multiquery,
        }
    }

    pub fn bundle_node(&self) -> LeafBundleNode {
        let fragments = self.fragments.each_ref().map(|fragment| {
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
                leaf_utils::suffix_openings_for_bundle(fragment.fragment_index),
            );
        }

        LeafBundleNode::new(
            self.leaf_node.marker(),
            *self.leaf_node.stem(),
            SparseVector::new(fragments),
            BundleProof::new(MultiProof::create_portal_network_proof(bundle_multiquery)),
        )
    }

    pub fn bundle_node_with_proof(&self, block_hash: B256) -> LeafBundleNodeWithProof {
        LeafBundleNodeWithProof {
            node: self.bundle_node(),
            block_hash,
            trie_path: self.trie_path.clone(),
            multiproof: MultiProof::create_portal_network_proof(self.trie_path_multiquery.clone()),
        }
    }

    pub fn fragment_commitment(&self, fragment_index: u8) -> &Point {
        &self.fragments[fragment_index as usize].commitment
    }

    pub fn fragment_node(&self, fragment_index: u8) -> LeafFragmentNode {
        let fragment = &self.fragments[fragment_index as usize];
        LeafFragmentNode::new(
            fragment_index,
            SparseVector::new(fragment.values.each_ref().map(|value| value.cloned())),
        )
    }

    pub fn fragment_node_with_proof(
        &self,
        fragment_index: u8,
        block_hash: B256,
    ) -> LeafFragmentNodeWithProof {
        let fragment = &self.fragments[fragment_index as usize];

        let mut multiquery = self.trie_path_multiquery.clone();
        // Open [marker, stem, c1/c2] for bundle commitment
        multiquery.add_vector(
            self.leaf_node.commitment().to_point(),
            self.leaf_node.to_lagrange_basis(),
            [
                LEAF_MARKER_INDEX,
                LEAF_STEM_INDEX,
                leaf_utils::leaf_suffix_index(fragment.fragment_index),
            ],
        );
        // Open children for suffix commitment (c1/c2)
        let poly = match leaf_utils::leaf_suffix_index(fragment.fragment_index) {
            LEAF_C1_INDEX => self.leaf_node.to_c1_lagrange_basis(),
            LEAF_C2_INDEX => self.leaf_node.to_c2_lagrange_basis(),
            _ => unreachable!(),
        };
        multiquery.add_vector(
            fragment.suffix_commitment.to_point(),
            poly,
            leaf_utils::suffix_openings(fragment.fragment_index),
        );

        LeafFragmentNodeWithProof {
            node: self.fragment_node(fragment_index),
            block_hash,
            marker: self.leaf_node.marker(),
            bundle_commitment: self.leaf_node.commitment().to_point(),
            suffix_commitment: fragment.suffix_commitment.to_point(),
            trie_path: self.trie_path.clone(),
            multiproof: MultiProof::create_portal_network_proof(multiquery),
        }
    }
}
