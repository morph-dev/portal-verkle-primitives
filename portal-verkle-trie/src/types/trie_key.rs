use alloy_primitives::B256;
use derive_more::{Constructor, Deref, From, Index};

use super::stem::Stem;

#[derive(PartialEq, Eq, Clone, Copy, Constructor, Index, Deref, From)]
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

    pub fn stem(&self) -> Stem {
        self.into()
    }

    pub fn suffix(&self) -> u8 {
        self[self.len() - 1]
    }
}
