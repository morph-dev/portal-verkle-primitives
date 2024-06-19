use std::array;

use banderwagon::{Element, Fr, PrimeField, Zero};

use crate::{
    constants::VERKLE_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    types::{Stem, TrieValue},
};

pub struct LeafNode {
    marker: u64,
    stem: Stem,
    commitment: Element,
    c1: Element,
    c2: Element,
    children: [Option<TrieValue>; VERKLE_NODE_WIDTH],
}

impl LeafNode {
    pub const EXTENSION_MARKER_INDEX: usize = 0;
    pub const STEM_INDEX: usize = 1;
    pub const C1_INDEX: usize = 2;
    pub const C2_INDEX: usize = 3;

    pub fn new(stem: Stem) -> Self {
        Self::new_with_childrent(stem, array::from_fn(|_| None))
    }

    pub fn new_with_childrent(
        stem: Stem,
        children: [Option<TrieValue>; VERKLE_NODE_WIDTH],
    ) -> Self {
        let (children_first_half, children_second_half) = children.split_at(VERKLE_NODE_WIDTH / 2);

        let suffix_commitment = |suffix_children: &[Option<TrieValue>]| {
            DefaultMsm.commit_sparse(
                suffix_children
                    .iter()
                    .enumerate()
                    .filter_map(|(suffix_index, child)| {
                        child.as_ref().map(|child| (suffix_index, child))
                    })
                    .flat_map(|(suffix_index, child)| {
                        let (low_value, high_value) = Self::split_trie_value(child);
                        [
                            (2 * suffix_index, low_value),
                            (2 * suffix_index + 1, high_value),
                        ]
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
        };

        let c1 = suffix_commitment(children_first_half);
        let c2 = suffix_commitment(children_second_half);

        let marker = 1; // Extension marker
        let commitment = DefaultMsm.commit_sparse(&[
            (Self::EXTENSION_MARKER_INDEX, Fr::from(marker)),
            (
                Self::STEM_INDEX,
                Fr::from_le_bytes_mod_order(stem.as_slice()),
            ),
            (Self::C1_INDEX, c1.map_to_scalar_field()),
            (Self::C2_INDEX, c2.map_to_scalar_field()),
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

    pub fn set(&mut self, index: usize, child: TrieValue) {
        let (new_low_value, new_high_value) = Self::split_trie_value(&child);
        let (old_low_value, old_high_value) = self.children[index].replace(child).map_or_else(
            || (Fr::zero(), Fr::zero()),
            |old_child| Self::split_trie_value(&old_child),
        );

        let suffix_index = index % (VERKLE_NODE_WIDTH / 2);
        let suffix_commitment_diff = DefaultMsm
            .scalar_mul(2 * suffix_index, new_low_value - old_low_value)
            + DefaultMsm.scalar_mul(2 * suffix_index + 1, new_high_value - old_high_value);

        if index < VERKLE_NODE_WIDTH / 2 {
            self.c1 += suffix_commitment_diff;
            self.commitment += DefaultMsm.scalar_mul(Self::C1_INDEX, self.c1.map_to_scalar_field());
        } else {
            self.c2 += suffix_commitment_diff;
            self.commitment += DefaultMsm.scalar_mul(Self::C2_INDEX, self.c2.map_to_scalar_field());
        }
    }

    pub fn get(&self, index: usize) -> Option<&TrieValue> {
        self.children[index].as_ref()
    }

    /// Splits trie value (32 bytes) into low (first 16 bytes) and high (second 16 bytes), and
    /// converts them to `Fr` scalar field.
    ///
    /// It also adds 2^128 to the low value.
    pub fn split_trie_value(value: &TrieValue) -> (Fr, Fr) {
        let (low_value, high_value) = value.split_at(16);
        let mut low_value = Vec::from(low_value);
        low_value.push(1);
        (
            Fr::from_le_bytes_mod_order(low_value.as_slice()),
            Fr::from_le_bytes_mod_order(high_value),
        )
    }
}
