use alloy_primitives::B256;
use banderwagon::{Fr, PrimeField};
use derive_more::{Constructor, Deref, From, Index};

#[derive(Default, PartialEq, Eq, Clone, Copy, Constructor, Index, Deref, From)]
pub struct TrieValue(B256);

impl TrieValue {
    /// Splits self into low (first 16 bytes) and high (second 16 bytes) values, and converts them
    /// to `Fr` scalar field.
    ///
    /// It also adds 2^128 to the low value.
    pub fn split(&self) -> (Fr, Fr) {
        let (low_value, high_value) = self.split_at(16);
        let mut low_value = Vec::from(low_value);
        low_value.push(1);
        (
            Fr::from_le_bytes_mod_order(low_value.as_slice()),
            Fr::from_le_bytes_mod_order(high_value),
        )
    }
}
