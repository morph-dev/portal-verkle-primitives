use std::{
    fmt::Debug,
    iter::{Product, Sum},
    ops,
};

use alloy_primitives::B256;
use ark_ff::batch_inversion_and_mul;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Field, Fr, One, PrimeField, Zero};
use derive_more::{Constructor, Deref, From, Into};
use overload::overload;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};

use crate::Stem;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Constructor, From, Into, Deref)]
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

    pub fn one() -> Self {
        Self(Fr::one())
    }

    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(Self)
    }

    /// Calculates inverse of all provided scalars, ignoring the ones with value zero.
    pub fn batch_inversion(scalars: &mut [Self]) {
        Self::batch_inverse_and_multiply(scalars, &Self::one())
    }

    /// Calculates inverses and multiplies them.
    ///
    /// Updates variable `values`: `v_i` => `m / v_i`.
    ///
    /// Ignores the zero values.
    pub fn batch_inverse_and_multiply(values: &mut [Self], m: &Self) {
        let mut frs = values.iter().map(|value| value.0).collect::<Vec<_>>();
        batch_inversion_and_mul(&mut frs, m);
        for (value, fr) in values.iter_mut().zip(frs.into_iter()) {
            value.0 = fr;
        }
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

overload!(- (me: ?ScalarField) -> ScalarField { ScalarField(-me.0) });

overload!((lhs: &mut ScalarField) += (rhs: ?ScalarField) { lhs.0 += &rhs.0; });
overload!((lhs: ScalarField) + (rhs: ?ScalarField) -> ScalarField {
    let mut lhs = lhs; lhs += rhs; lhs
});
overload!((lhs: &ScalarField) + (rhs: ?ScalarField) -> ScalarField { ScalarField(lhs.0) + rhs });

overload!((lhs: &mut ScalarField) -= (rhs: ?ScalarField) { lhs.0 -= &rhs.0; });
overload!((lhs: ScalarField) - (rhs: ?ScalarField) -> ScalarField {
    let mut lhs = lhs; lhs -= rhs; lhs
});
overload!((lhs: &ScalarField) - (rhs: ?ScalarField) -> ScalarField { ScalarField(lhs.0) - rhs });

overload!((lhs: &mut ScalarField) *= (rhs: ?ScalarField) { lhs.0 *= &rhs.0; });
overload!((lhs: ScalarField) * (rhs: ?ScalarField) -> ScalarField {
    let mut lhs = lhs; lhs *= rhs; lhs
});
overload!((lhs: &ScalarField) * (rhs: ?ScalarField) -> ScalarField { ScalarField(lhs.0) * rhs });

impl<'a> Sum<&'a Self> for ScalarField {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(ScalarField::zero(), |sum, item| sum + item)
    }
}

impl Sum for ScalarField {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ScalarField::zero(), |sum, item| sum + item)
    }
}

impl<'a> Product<&'a Self> for ScalarField {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(ScalarField::one(), |prod, item| prod * item)
    }
}

impl Product for ScalarField {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ScalarField::one(), |prod, item| prod * item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_inversion_and_multiplication() {
        let mut values = vec![
            ScalarField::from(1),
            ScalarField::from(10),
            ScalarField::from(123),
            ScalarField::from(0),
            ScalarField::from(1_000_000),
            ScalarField::from(1 << 30),
            ScalarField::from(1 << 60),
        ];
        let m = ScalarField::from(42);

        let expected = values
            .iter()
            .map(|v| {
                if v.is_zero() {
                    v.clone()
                } else {
                    &m * v.inverse().unwrap()
                }
            })
            .collect::<Vec<_>>();

        ScalarField::batch_inverse_and_multiply(&mut values, &m);

        assert_eq!(expected, values);
    }
}
