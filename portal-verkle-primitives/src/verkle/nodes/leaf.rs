use std::collections::HashMap;

use crate::{
    constants::{
        LEAF_C1_INDEX, LEAF_C2_INDEX, LEAF_MARKER_INDEX, LEAF_STEM_INDEX, VERKLE_NODE_WIDTH,
    },
    proof::lagrange_basis::LagrangeBasis,
    ssz::SparseVector,
    utils::array_long_const,
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
            let suffix_value_index = index % (VERKLE_NODE_WIDTH / 2) as u8;
            let suffix_commitment_diff = if *index < (VERKLE_NODE_WIDTH / 2) as u8 {
                &mut c1_diff
            } else {
                &mut c2_diff
            };

            let (new_low_value, new_high_value) = new_value.split();
            let old_value = self.values[*index as usize].replace(*new_value);
            let (old_low_value, old_high_value) = old_value.split();

            suffix_commitment_diff.push((2 * suffix_value_index, new_low_value - old_low_value));
            suffix_commitment_diff
                .push((2 * suffix_value_index + 1, new_high_value - old_high_value));
        }
        self.commitment.update(&[
            (LEAF_C1_INDEX, self.c1.update(&c1_diff)),
            (LEAF_C2_INDEX, self.c2.update(&c2_diff)),
        ])
    }

    pub fn to_lagrange_basis(&self) -> LagrangeBasis {
        let mut scalars = array_long_const(ScalarField::zero());
        scalars[LEAF_MARKER_INDEX as usize] = ScalarField::from(self.marker);
        scalars[LEAF_STEM_INDEX as usize] = ScalarField::from(&self.stem);
        scalars[LEAF_C1_INDEX as usize] = self.c1.to_scalar();
        scalars[LEAF_C2_INDEX as usize] = self.c2.to_scalar();

        LagrangeBasis::new(scalars)
    }

    pub fn to_c1_lagrange_basis(&self) -> LagrangeBasis {
        let mut scalars = array_long_const(ScalarField::zero());
        for (suffix_value_index, value) in self.values[..VERKLE_NODE_WIDTH / 2]
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.as_ref().map(|value| (index, value)))
        {
            let (low_value, high_value) = value.split();
            scalars[2 * suffix_value_index] = low_value;
            scalars[2 * suffix_value_index + 1] = high_value;
        }
        LagrangeBasis::new(scalars)
    }

    pub fn to_c2_lagrange_basis(&self) -> LagrangeBasis {
        let mut scalars = array_long_const(ScalarField::zero());
        for (suffix_value_index, value) in self.values[VERKLE_NODE_WIDTH / 2..]
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.as_ref().map(|value| (index, value)))
        {
            let (low_value, high_value) = value.split();
            scalars[2 * suffix_value_index] = low_value;
            scalars[2 * suffix_value_index + 1] = high_value;
        }
        LagrangeBasis::new(scalars)
    }
}
