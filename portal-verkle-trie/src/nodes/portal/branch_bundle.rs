use std::array;

use verkle_core::{constants::PORTAL_NETWORK_NODE_WIDTH, Point, ScalarField};

pub struct BranchBundleNode {
    commitment: Point,
    children: [Point; PORTAL_NETWORK_NODE_WIDTH],
}

impl BranchBundleNode {
    pub fn new(children: [Point; PORTAL_NETWORK_NODE_WIDTH]) -> Self {
        Self {
            commitment: children
                .iter()
                .filter(|child| !child.is_zero())
                .cloned()
                .sum(),
            children,
        }
    }

    pub fn commitment(&self) -> &Point {
        &self.commitment
    }

    pub fn commitment_hash(&self) -> ScalarField {
        self.commitment.map_to_scalar_field()
    }

    pub fn set(&mut self, index: usize, child: Point) {
        self.commitment += child.clone() - &self.children[index];
        self.children[index] = child;
    }

    pub fn get(&self, index: usize) -> &Point {
        &self.children[index]
    }
}

impl Default for BranchBundleNode {
    fn default() -> Self {
        Self::new(array::from_fn(|_| Point::zero()))
    }
}
