use std::array;

use alloy_primitives::U256;
use banderwagon::{Element, Fr, One, PrimeField};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    types::Stem,
};

pub struct LeafHigher {
    version: U256,
    stem: Stem,
    commitment: Element,
    c1: Element,
    c2: Element,
    children: [Element; PORTAL_NETWORK_NODE_WIDTH],
}

impl LeafHigher {
    const EXTENSION_MARKER_INDEX: usize = 0;
    const STEM_INDEX: usize = 1;
    const C1_INDEX: usize = 2;
    const C2_INDEX: usize = 3;

    pub fn new(stem: Stem) -> Self {
        Self::new_with_childrent(stem, array::from_fn(|_| Element::zero()))
    }

    pub fn new_with_childrent(stem: Stem, children: [Element; PORTAL_NETWORK_NODE_WIDTH]) -> Self {
        let (first_half, second_half) = children.split_at(PORTAL_NETWORK_NODE_WIDTH / 2);

        let c1: Element = first_half
            .iter()
            .filter(|child| !child.is_zero())
            .cloned()
            .sum();
        let c2: Element = second_half
            .iter()
            .filter(|child| !child.is_zero())
            .cloned()
            .sum();

        let commitment = DefaultMsm.commit_sparse(&[
            (Self::EXTENSION_MARKER_INDEX, Fr::one()), // Extension marker
            (
                Self::STEM_INDEX,
                Fr::from_le_bytes_mod_order(stem.as_slice()),
            ),
            (Self::C1_INDEX, c1.map_to_scalar_field()),
            (Self::C2_INDEX, c2.map_to_scalar_field()),
        ]);

        Self {
            version: U256::ZERO,
            stem,
            commitment,
            c1,
            c2,
            children,
        }
    }

    pub fn version(&self) -> &U256 {
        &self.version
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    pub fn commitment(&self) -> Element {
        self.commitment
    }

    pub fn commitment_hash(&self) -> Fr {
        self.commitment.map_to_scalar_field()
    }

    pub fn set(&mut self, index: usize, child: Element) {
        let diff = child - self.children[index];
        self.children[index] = child;
        if index < self.children.len() / 2 {
            self.c1 += diff;
            self.commitment += DefaultMsm.scalar_mul(Self::C1_INDEX, self.c1.map_to_scalar_field());
        } else {
            self.c2 += diff;
            self.commitment += DefaultMsm.scalar_mul(Self::C2_INDEX, self.c2.map_to_scalar_field());
        }
    }

    pub fn get(&self, index: usize) -> &Element {
        &self.children[index]
    }
}
