use derive_more::{AsRef, Deref, From, Index};
use ssz::{Decode, Encode};

use super::trie_key::TrieKey;

#[derive(PartialEq, Eq, AsRef, Deref, From, Index)]
pub struct Stem([u8; Self::STEM_LENGTH]);

impl Stem {
    pub const STEM_LENGTH: usize = 31;
}

impl From<&TrieKey> for Stem {
    fn from(key: &TrieKey) -> Self {
        let mut stem = [0u8; Self::STEM_LENGTH];
        stem.copy_from_slice(&key[..Self::STEM_LENGTH]);
        Stem(stem)
    }
}

impl From<TrieKey> for Stem {
    fn from(key: TrieKey) -> Self {
        Self::from(&key)
    }
}

impl Encode for Stem {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        Self::STEM_LENGTH
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend(self.as_slice())
    }

    fn ssz_bytes_len(&self) -> usize {
        self.len()
    }
}

impl Decode for Stem {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        Self::STEM_LENGTH
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        match <[u8; 31]>::try_from(bytes) {
            Ok(stem) => Ok(Self(stem)),
            Err(_) => Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: Self::STEM_LENGTH,
            }),
        }
    }
}
