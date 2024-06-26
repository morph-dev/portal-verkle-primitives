use alloy_primitives::B256;
use derive_more::{Constructor, Deref, DerefMut, From, Index, IndexMut};
use serde::{Deserialize, Serialize};

use crate::Stem;

#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Constructor,
    Index,
    IndexMut,
    Deref,
    DerefMut,
    From,
    Serialize,
    Deserialize,
)]
pub struct TrieKey(B256);

impl TrieKey {
    pub fn from_stem_and_last_byte(stem: &Stem, suffix: u8) -> Self {
        let mut key = B256::right_padding_from(stem.as_slice());
        key[B256::len_bytes() - 1] = suffix;
        key.into()
    }

    pub fn len_bytes() -> usize {
        B256::len_bytes()
    }

    pub fn has_stem(&self, stem: &Stem) -> bool {
        self.starts_with(stem.as_slice())
    }

    pub fn stem(&self) -> Stem {
        self.into()
    }

    pub fn suffix(&self) -> u8 {
        self[self.len() - 1]
    }
}
