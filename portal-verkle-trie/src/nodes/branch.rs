use std::array;

use verkle_core::{
    constants::VERKLE_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    Point, ScalarField,
};

pub struct BranchNode {
    commitment: Point,
    children: [Point; VERKLE_NODE_WIDTH],
}

impl BranchNode {
    pub fn new(children: [Point; VERKLE_NODE_WIDTH]) -> Self {
        let commitment = DefaultMsm
            .commit_lagrange(&children.each_ref().map(|child| child.map_to_scalar_field()));
        Self {
            commitment,
            children,
        }
    }

    pub fn commitment(&self) -> &Point {
        &self.commitment
    }

    pub fn commitment_hash(&self) -> ScalarField {
        self.commitment.map_to_scalar_field()
    }

    pub fn set(&mut self, child_index: usize, child: Point) {
        self.commitment += DefaultMsm.scalar_mul(
            child_index,
            child.map_to_scalar_field() - self.children[child_index].map_to_scalar_field(),
        );
        self.children[child_index] = child;
    }

    pub fn get(&self, child_index: usize) -> &Point {
        &self.children[child_index]
    }
}

impl Default for BranchNode {
    fn default() -> Self {
        Self::new(array::from_fn(|_| Point::zero()))
    }
}
