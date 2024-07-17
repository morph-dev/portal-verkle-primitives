use alloy_primitives::wrap_fixed_bytes;

use crate::TrieKey;

wrap_fixed_bytes!(pub struct Stem<31>;);

impl From<&TrieKey> for Stem {
    fn from(key: &TrieKey) -> Self {
        Stem::from_slice(&key[..Self::len_bytes()])
    }
}

impl From<TrieKey> for Stem {
    fn from(key: TrieKey) -> Self {
        Self::from(&key)
    }
}
