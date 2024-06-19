use banderwagon::{Element, Fr, PrimeField, Zero};

use crate::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    types::TrieValue,
};

pub struct LeafFragment {
    parent_index: usize,
    commitment: Element,
    children: [Option<TrieValue>; PORTAL_NETWORK_NODE_WIDTH],
}

impl LeafFragment {
    pub fn new(parent_index: usize) -> Self {
        Self::new_with_children(parent_index, <_>::default())
    }

    pub fn new_with_children(
        parent_index: usize,
        children: [Option<TrieValue>; PORTAL_NETWORK_NODE_WIDTH],
    ) -> Self {
        if parent_index >= PORTAL_NETWORK_NODE_WIDTH {
            panic!("Invalid parent index: {parent_index}")
        }

        let commitment = DefaultMsm.commit_sparse(
            children
                .iter()
                .enumerate()
                .flat_map(|(child_index, child)| match child {
                    None => {
                        vec![]
                    }
                    Some(child) => {
                        let (low_index, high_index) =
                            Self::bases_indices(parent_index, child_index);
                        let (low_value, high_value) = Self::split_trie_value(child);
                        vec![(low_index, low_value), (high_index, high_value)]
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

    pub fn set(&mut self, child_index: usize, child: TrieValue) {
        let (low_index, high_index) = Self::bases_indices(self.parent_index, child_index);
        let (old_low_value, old_high_value) = self.children[child_index].map_or_else(
            || (Fr::zero(), Fr::zero()),
            |child_value| Self::split_trie_value(&child_value),
        );
        let (new_low_value, new_high_value) = Self::split_trie_value(&child);

        self.commitment += DefaultMsm.scalar_mul(low_index, new_low_value - old_low_value);
        self.commitment += DefaultMsm.scalar_mul(high_index, new_high_value - old_high_value);
        self.children[child_index] = Some(child);
    }

    pub fn get(&self, index: usize) -> Option<&TrieValue> {
        self.children[index].as_ref()
    }

    /// Returns the bases indices that correspond to the child index.
    fn bases_indices(parent_index: usize, child_index: usize) -> (usize, usize) {
        let starting_index =
            parent_index % (PORTAL_NETWORK_NODE_WIDTH / 2) * 2 * PORTAL_NETWORK_NODE_WIDTH;
        let low_index = starting_index + 2 * child_index;
        let high_index = low_index + 1;
        (low_index, high_index)
    }

    /// Splits trie value (32 bytes) into low (first 16 bytes) and high (second 16 bytes), and
    /// converts them to `Fr` scalar field.
    ///
    /// It also adds 2^128 to the low value.
    fn split_trie_value(value: &TrieValue) -> (Fr, Fr) {
        let (low_value, high_value) = value.split_at(16);
        let mut low_value = Vec::from(low_value);
        low_value.push(1);
        (
            Fr::from_le_bytes_mod_order(low_value.as_slice()),
            Fr::from_le_bytes_mod_order(high_value),
        )
    }
}
