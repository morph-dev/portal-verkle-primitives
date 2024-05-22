use std::array;

use banderwagon::{Element, Fr};

use crate::constants::PORTAL_NETWORK_NODE_WIDTH;

pub struct BranchHigher {
    commitment: Element,
    children: [Element; PORTAL_NETWORK_NODE_WIDTH],
}

impl BranchHigher {
    pub fn new(children: [Element; PORTAL_NETWORK_NODE_WIDTH]) -> Self {
        Self {
            commitment: children
                .iter()
                .filter(|child| !child.is_zero())
                .cloned()
                .sum(),
            children,
        }
    }

    pub fn commitment(&self) -> Element {
        self.commitment
    }

    pub fn commitment_hash(&self) -> Fr {
        self.commitment.map_to_scalar_field()
    }

    pub fn set(&mut self, index: usize, child: Element) {
        self.commitment += child - self.children[index];
        self.children[index] = child;
    }

    pub fn get(&self, index: usize) -> &Element {
        &self.children[index]
    }
}

impl Default for BranchHigher {
    fn default() -> Self {
        Self::new(array::from_fn(|_| Element::zero()))
    }
}
