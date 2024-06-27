use std::fmt::Debug;

use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Fr, PrimeField, Zero};
use derive_more::{Add, Constructor, Deref, From, Into, Neg, Sub};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};

use crate::Stem;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Constructor, From, Into, Deref, Neg, Add, Sub)]
pub struct ScalarField(pub(crate) Fr);

impl ScalarField {
    pub fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        Self(Fr::from_le_bytes_mod_order(bytes))
    }

    pub(crate) fn from_be_bytes(bytes: B256) -> Self {
        Self(
            Fr::deserialize_compressed(bytes.as_slice())
                .expect("ScalarFieldValue should deserialize from B256"),
        )
    }

    pub(crate) fn to_be_bytes(&self) -> B256 {
        let mut result = B256::ZERO;
        self.serialize_compressed(result.as_mut_slice())
            .expect("ScalarFieldValue should serialize to B256");
        result
    }

    pub fn zero() -> Self {
        Self(Fr::zero())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<B256> for ScalarField {
    fn from(mut value: B256) -> Self {
        // Reverse from little-ending ordering
        value.reverse();
        Self::from_be_bytes(value)
    }
}

impl From<&ScalarField> for B256 {
    fn from(value: &ScalarField) -> Self {
        let mut result = value.to_be_bytes();
        // Reverse to little-ending ordering
        result.reverse();
        result
    }
}

impl From<&Stem> for ScalarField {
    fn from(stem: &Stem) -> Self {
        Self::from_le_bytes_mod_order(stem.as_slice())
    }
}

impl From<u64> for ScalarField {
    fn from(value: u64) -> Self {
        Self(value.into())
    }
}

impl Serialize for ScalarField {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        B256::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ScalarField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        B256::deserialize(deserializer).map(Self::from)
    }
}

impl Encode for ScalarField {
    fn is_ssz_fixed_len() -> bool {
        <B256 as Encode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <B256 as Encode>::ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        B256::from(self).ssz_append(buf)
    }

    fn ssz_bytes_len(&self) -> usize {
        <B256 as Encode>::ssz_fixed_len()
    }
}

impl Decode for ScalarField {
    fn is_ssz_fixed_len() -> bool {
        <B256 as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <B256 as Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        B256::from_ssz_bytes(bytes).map(Self::from)
    }
}

impl Debug for ScalarField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        B256::from(self).fmt(f)
    }
}
