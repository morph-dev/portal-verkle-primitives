use std::array;

use banderwagon::{Element, Fr, PrimeField};
use verkle_core::{
    constants::{
        EXTENSION_C1_INDEX, EXTENSION_C2_INDEX, EXTENSION_MARKER_INDEX, EXTENSION_STEM_INDEX,
        PORTAL_NETWORK_NODE_WIDTH,
    },
    msm::{DefaultMsm, MultiScalarMultiplicator},
    Stem,
};

pub struct LeafBundleNode {
    marker: u64,
    stem: Stem,
    commitment: Element,
    c1: Element,
    c2: Element,
    children: [Element; PORTAL_NETWORK_NODE_WIDTH],
}

impl LeafBundleNode {
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

        let marker = 1; // Extension marker
        let commitment = DefaultMsm.commit_sparse(&[
            (EXTENSION_MARKER_INDEX, Fr::from(marker)),
            (
                EXTENSION_STEM_INDEX,
                Fr::from_le_bytes_mod_order(stem.as_slice()),
            ),
            (EXTENSION_C1_INDEX, c1.map_to_scalar_field()),
            (EXTENSION_C2_INDEX, c2.map_to_scalar_field()),
        ]);

        Self {
            marker,
            stem,
            commitment,
            c1,
            c2,
            children,
        }
    }

    pub fn version(&self) -> u64 {
        self.marker
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
            let old_c1_commitment_hash = self.c1.map_to_scalar_field();
            self.c1 += diff;
            self.commitment += DefaultMsm.scalar_mul(
                EXTENSION_C1_INDEX,
                self.c1.map_to_scalar_field() - old_c1_commitment_hash,
            );
        } else {
            let old_c2_commitment_hash = self.c2.map_to_scalar_field();
            self.c2 += diff;
            self.commitment += DefaultMsm.scalar_mul(
                EXTENSION_C2_INDEX,
                self.c2.map_to_scalar_field() - old_c2_commitment_hash,
            );
        }
    }

    pub fn get(&self, index: usize) -> &Element {
        &self.children[index]
    }
}
