use std::collections::HashMap;

use crate::{
    constants::{
        LEAF_C1_INDEX, LEAF_C2_INDEX, LEAF_MARKER_INDEX, LEAF_STEM_INDEX, VERKLE_NODE_WIDTH,
    },
    ssz::SparseVector,
    ScalarField, Stem, TrieValue, TrieValueSplit, CRS,
};

use super::commitment::Commitment;

pub struct LeafNode {
    marker: u64,
    stem: Stem,
    commitment: Commitment,
    c1: Commitment,
    c2: Commitment,
    values: SparseVector<TrieValue, VERKLE_NODE_WIDTH>,
}

impl LeafNode {
    pub fn new(stem: Stem) -> Self {
        let marker = 1;

        let commitment = CRS::commit_sparse(&[
            (LEAF_MARKER_INDEX, ScalarField::from(marker)),
            (LEAF_STEM_INDEX, ScalarField::from(&stem)),
        ]);

        Self {
            marker,
            stem,
            commitment: Commitment::new(commitment),
            c1: Commitment::zero(),
            c2: Commitment::zero(),
            values: SparseVector::default(),
        }
    }

    pub fn marker(&self) -> u64 {
        self.marker
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    pub fn commitment(&self) -> &Commitment {
        &self.commitment
    }

    pub fn c1(&self) -> &Commitment {
        &self.c1
    }

    pub fn c2(&self) -> &Commitment {
        &self.c2
    }

    pub fn get(&self, index: u8) -> Option<&TrieValue> {
        self.values[index as usize].as_ref()
    }

    /// Sets trie values and returns by how much the commitment hash changed.
    pub fn update(&mut self, writes: &HashMap<u8, TrieValue>) -> ScalarField {
        // Contains the changes of c1 and c2
        let mut c1_diff = vec![];
        let mut c2_diff = vec![];

        for (index, new_value) in writes.iter() {
            let suffix_index = index % (VERKLE_NODE_WIDTH / 2) as u8;
            let suffix_commitment_diff = if *index < (VERKLE_NODE_WIDTH / 2) as u8 {
                &mut c1_diff
            } else {
                &mut c2_diff
            };

            let (new_low_value, new_high_value) = new_value.split();
            let old_value = self.values[*index as usize].replace(*new_value);
            let (old_low_value, old_high_value) = old_value.split();

            suffix_commitment_diff.push((2 * suffix_index, new_low_value - old_low_value));
            suffix_commitment_diff.push((2 * suffix_index + 1, new_high_value - old_high_value));
        }
        self.commitment.update(&[
            (LEAF_C1_INDEX, self.c1.update(&c1_diff)),
            (LEAF_C2_INDEX, self.c2.update(&c2_diff)),
        ])
    }
}
