use std::array;

use verkle_core::{
    constants::{
        EXTENSION_C1_INDEX, EXTENSION_C2_INDEX, EXTENSION_MARKER_INDEX, EXTENSION_STEM_INDEX,
        VERKLE_NODE_WIDTH,
    },
    msm::{DefaultMsm, MultiScalarMultiplicator},
    Point, ScalarField, Stem, TrieValue, TrieValueSplit,
};

use crate::{types::state_write::StemStateWrite, verkle_trie::error::VerkleTrieError};

use super::commitment::Commitment;

pub struct LeafNode {
    marker: u64,
    stem: Stem,
    commitment: Commitment,
    pub c1: Commitment,
    pub c2: Commitment,
    values: [Option<TrieValue>; VERKLE_NODE_WIDTH],
}

impl LeafNode {
    pub fn new(stem: Stem) -> Self {
        let marker = 1;

        let commitment = DefaultMsm.commit_sparse(&[
            (EXTENSION_MARKER_INDEX, ScalarField::from(marker)),
            (EXTENSION_STEM_INDEX, ScalarField::from(&stem)),
        ]);

        Self {
            marker,
            stem,
            commitment: Commitment::new(commitment),
            c1: Commitment::zero(),
            c2: Commitment::zero(),
            values: array::from_fn(|_| None),
        }
    }

    pub fn marker(&self) -> u64 {
        self.marker
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.commitment()
    }

    pub fn commitment_hash(&mut self) -> ScalarField {
        self.commitment.commitment_hash()
    }

    pub fn get(&self, index: usize) -> Option<&TrieValue> {
        self.values[index].as_ref()
    }

    pub fn set(&mut self, index: usize, value: TrieValue) {
        let (new_low_value, new_high_value) = value.split();
        let old_value = self.values[index].replace(value);
        let (old_low_value, old_high_value) = old_value.split();

        let suffix_index = index % (VERKLE_NODE_WIDTH / 2);
        let suffix_commitment_diff = DefaultMsm
            .scalar_mul(2 * suffix_index, new_low_value - old_low_value)
            + DefaultMsm.scalar_mul(2 * suffix_index + 1, new_high_value - old_high_value);

        if index < VERKLE_NODE_WIDTH / 2 {
            let old_c1_commitment_hash = self.c1.commitment_hash();
            self.c1 += suffix_commitment_diff;
            self.commitment += DefaultMsm.scalar_mul(
                EXTENSION_C1_INDEX,
                self.c1.commitment_hash() - old_c1_commitment_hash,
            );
        } else {
            let old_c2_commitment_hash = self.c2.commitment_hash();
            self.c2 += suffix_commitment_diff;
            self.commitment += DefaultMsm.scalar_mul(
                EXTENSION_C2_INDEX,
                self.c2.commitment_hash() - old_c2_commitment_hash,
            );
        }
    }

    pub fn update(&mut self, state_write: &StemStateWrite) -> Result<(), VerkleTrieError> {
        if self.stem != state_write.stem {
            return Err(VerkleTrieError::UnexpectedStem {
                expected: self.stem,
                actual: state_write.stem,
            });
        }
        let old_c1_commitment_hash = self.c1.commitment_hash();
        let old_c2_commitment_hash = self.c2.commitment_hash();
        for suffix_write in state_write.suffix_writes.iter() {
            let index = suffix_write.suffix as usize;
            let old_value = suffix_write.old_value;
            if self.values[index] != old_value {
                return Err(VerkleTrieError::WrongOldValue {
                    stem: self.stem,
                    index: index as u8,
                    expected: self.values[index],
                    actual: old_value,
                });
            }
            let new_value = suffix_write.new_value;
            let (new_low_value, new_high_value) = new_value.split();
            let old_value = self.values[index].replace(new_value);
            let (old_low_value, old_high_value) = old_value.split();

            let suffix_index = index % (VERKLE_NODE_WIDTH / 2);
            let suffix_commitment_diff = DefaultMsm.commit_sparse(&[
                (2 * suffix_index, new_low_value - old_low_value),
                (2 * suffix_index + 1, new_high_value - old_high_value),
            ]);

            if index < VERKLE_NODE_WIDTH / 2 {
                self.c1 += suffix_commitment_diff;
            } else {
                self.c2 += suffix_commitment_diff;
            }
        }
        self.commitment += DefaultMsm.scalar_mul(
            EXTENSION_C1_INDEX,
            self.c1.commitment_hash() - old_c1_commitment_hash,
        );
        self.commitment += DefaultMsm.scalar_mul(
            EXTENSION_C2_INDEX,
            self.c2.commitment_hash() - old_c2_commitment_hash,
        );
        Ok(())
    }
}
