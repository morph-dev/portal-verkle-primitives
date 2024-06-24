use alloy_primitives::{B256, U256};
use banderwagon::{Fr, PrimeField, Zero};
use derive_more::{Constructor, Deref, From, Index};
use serde::{Deserialize, Serialize};

#[derive(
    Default,
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Constructor,
    Index,
    Deref,
    From,
    Serialize,
    Deserialize,
)]
pub struct TrieValue(B256);

impl From<U256> for TrieValue {
    fn from(value: U256) -> Self {
        Self(value.to_le_bytes().into())
    }
}

pub trait TrieValueSplit {
    /// Splits self into low (first 16 bytes) and high (second 16 bytes) values, and converts them
    /// to `Fr` scalar field.
    fn split(&self) -> (Fr, Fr);
}

impl TrieValueSplit for TrieValue {
    fn split(&self) -> (Fr, Fr) {
        let (low_value, high_value) = self.split_at(16);
        let mut low_value = Vec::from(low_value);
        low_value.push(1);
        (
            Fr::from_le_bytes_mod_order(low_value.as_slice()),
            Fr::from_le_bytes_mod_order(high_value),
        )
    }
}

impl<T: TrieValueSplit> TrieValueSplit for Option<T> {
    fn split(&self) -> (Fr, Fr) {
        match self {
            None => (Fr::zero(), Fr::zero()),
            Some(value) => value.split(),
        }
    }
}
