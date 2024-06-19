use std::array;

use banderwagon::{Element, Zero};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
};

pub struct BranchFragmentNode {
    parent_index: usize,
    commitment: Element,
    children: [Element; PORTAL_NETWORK_NODE_WIDTH],
}

impl BranchFragmentNode {
    pub fn new(parent_index: usize) -> Self {
        Self::new_with_children(parent_index, array::from_fn(|_| Element::zero()))
    }

    pub fn new_with_children(
        parent_index: usize,
        children: [Element; PORTAL_NETWORK_NODE_WIDTH],
    ) -> Self {
        if parent_index >= PORTAL_NETWORK_NODE_WIDTH {
            panic!("Invalid parent index: {parent_index}")
        }

        let commitment = DefaultMsm.commit_sparse(
            children
                .iter()
                .enumerate()
                .filter_map(|(child_index, child)| {
                    let child_commitment_hash = child.map_to_scalar_field();
                    if child_commitment_hash.is_zero() {
                        None
                    } else {
                        Some((
                            Self::bases_index(parent_index, child_index),
                            child_commitment_hash,
                        ))
                    }
                })
                .collect::<Vec<_>>()
                .as_slice(),
        );

        Self {
            parent_index,
            commitment,
            children,
        }
    }

    pub fn commitment(&self) -> Element {
        self.commitment
    }

    pub fn set(&mut self, child_index: usize, child: Element) {
        self.commitment += DefaultMsm.scalar_mul(
            Self::bases_index(self.parent_index, child_index),
            child.map_to_scalar_field() - self.children[child_index].map_to_scalar_field(),
        );
        self.children[child_index] = child;
    }

    pub fn get(&self, child_index: usize) -> &Element {
        &self.children[child_index]
    }

    fn bases_index(parent_index: usize, child_index: usize) -> usize {
        parent_index * PORTAL_NETWORK_NODE_WIDTH + child_index
    }
}
