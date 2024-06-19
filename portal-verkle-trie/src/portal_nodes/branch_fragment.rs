use std::array;

use banderwagon::{Element, Fr, Zero};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
};

pub struct BranchFragment {
    parent_index: usize,
    commitment: Element,
    children: [Fr; PORTAL_NETWORK_NODE_WIDTH],
}

impl BranchFragment {
    pub fn new(parent_index: usize) -> Self {
        Self::new_with_children(parent_index, array::from_fn(|_| Fr::zero()))
    }

    pub fn new_with_children(
        parent_index: usize,
        children: [Fr; PORTAL_NETWORK_NODE_WIDTH],
    ) -> Self {
        if parent_index >= PORTAL_NETWORK_NODE_WIDTH {
            panic!("Invalid parent index: {parent_index}")
        }

        let commitment = DefaultMsm.commit_sparse(
            children
                .iter()
                .enumerate()
                .filter_map(|(child_index, child)| {
                    if child.is_zero() {
                        None
                    } else {
                        Some((Self::bases_index(parent_index, child_index), *child))
                    }
                })
                .collect::<Vec<(usize, Fr)>>()
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

    pub fn set(&mut self, child_index: usize, child: Fr) {
        self.commitment += DefaultMsm.scalar_mul(
            Self::bases_index(self.parent_index, child_index),
            child - self.children[child_index],
        );
        self.children[child_index] = child;
    }

    pub fn get(&self, child_index: usize) -> &Fr {
        &self.children[child_index]
    }

    fn bases_index(parent_index: usize, child_index: usize) -> usize {
        parent_index * PORTAL_NETWORK_NODE_WIDTH + child_index
    }
}
