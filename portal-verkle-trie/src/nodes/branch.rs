use std::array;

use banderwagon::{Element, Fr};

use crate::{
    constants::VERKLE_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
};

pub struct BranchNode {
    commitment: Element,
    children: [Element; VERKLE_NODE_WIDTH],
}

impl BranchNode {
    pub fn new(children: [Element; VERKLE_NODE_WIDTH]) -> Self {
        let commitment = DefaultMsm
            .commit_lagrange(children.map(|child| child.map_to_scalar_field()).as_slice());
        Self {
            commitment,
            children,
        }
    }

    pub fn commitment(&self) -> Element {
        self.commitment
    }

    pub fn commitment_hash(&self) -> Fr {
        self.commitment.map_to_scalar_field()
    }

    pub fn set(&mut self, child_index: usize, child: Element) {
        self.commitment += DefaultMsm.scalar_mul(
            child_index,
            child.map_to_scalar_field() - self.children[child_index].map_to_scalar_field(),
        );
        self.children[child_index] = child;
    }

    pub fn get(&self, child_index: usize) -> &Element {
        &self.children[child_index]
    }
}

impl Default for BranchNode {
    fn default() -> Self {
        Self::new(array::from_fn(|_| Element::zero()))
    }
}
