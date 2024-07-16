use alloy_primitives::{bytes, wrap_fixed_bytes};

use crate::Stem;

wrap_fixed_bytes!(pub struct TrieKey<32>;);

impl TrieKey {
    pub fn from_stem_and_suffix(stem: &Stem, suffix: u8) -> Self {
        let mut key = Self::right_padding_from(stem.as_slice());
        key[Self::len_bytes() - 1] = suffix;
        key
    }

    pub fn starts_with_stem(&self, stem: &Stem) -> bool {
        self.starts_with(stem.as_slice())
    }

    pub fn stem(&self) -> Stem {
        self.into()
    }

    pub fn suffix(&self) -> u8 {
        self[Self::suffix_index()]
    }

    pub fn set_suffix(&mut self, suffix: u8) {
        self[Self::suffix_index()] = suffix
    }

    fn suffix_index() -> usize {
        Self::len_bytes() - 1
    }
}
