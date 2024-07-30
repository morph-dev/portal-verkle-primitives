use std::{
    fmt::Debug,
    iter::{self, Product, Sum},
    ops::{self, Mul},
};

use alloy_primitives::B256;
use ark_ff::batch_inversion_and_mul;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Field, Fr, One, PrimeField, Zero};
use derive_more::Constructor;
use itertools::{zip_eq, Itertools};
use overload::overload;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};

use crate::Stem;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct ScalarField(Fr);

impl ScalarField {
    pub(crate) fn inner(&self) -> Fr {
        self.0
    }

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
        self.0
            .serialize_compressed(result.as_mut_slice())
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

    // Returns powers of provided scalar: `[x^0, x^1, x^2, ..., x^(n-1)]`.
    pub fn powers_of(x: &Self, n: usize) -> Vec<Self> {
        iter::successors(Some(Self::one()), |prev| Some(prev * x))
            .take(n)
            .collect()
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

impl From<u8> for ScalarField {
    fn from(value: u8) -> Self {
        Self(value.into())
    }
}

impl From<u64> for ScalarField {
    fn from(value: u64) -> Self {
        Self(value.into())
    }
}

impl From<usize> for ScalarField {
    fn from(value: usize) -> Self {
        Self::from(value as u64)
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

impl Sum for ScalarField {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_else(Self::zero)
    }
}

impl<'a> Sum<&'a Self> for ScalarField {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(ScalarField::zero(), |sum, item| sum + item)
    }
}

impl Product for ScalarField {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a * b).unwrap_or_else(Self::one)
    }
}

impl<'a> Product<&'a Self> for ScalarField {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(ScalarField::one(), |prod, item| prod * item)
    }
}

pub trait BatchInversion<T> {
    /// Calculates inverses, ignoring zero.
    fn batch_inverse(self) -> Self;

    /// Calculates inverses and scales them, ignoring zero.
    ///
    /// ```text
    /// v_i => coeff / v_i
    /// ```
    fn batch_inverse_and_mul(self, coeff: &T) -> Self;
}

impl<T> BatchInversion<ScalarField> for T
where
    T: Sized + AsMut<[ScalarField]>,
{
    fn batch_inverse(self) -> T {
        self.batch_inverse_and_mul(&ScalarField::one())
    }

    fn batch_inverse_and_mul(mut self, coeff: &ScalarField) -> T {
        let mut values = self
            .as_mut()
            .iter()
            .map(|value| value.inner())
            .collect_vec();
        batch_inversion_and_mul(&mut values, &coeff.inner());
        for (original, inverted) in zip_eq(self.as_mut(), values) {
            original.0 = inverted;
        }
        self
    }
}

pub trait DotProduct<A, B>: Sized + Sum
where
    A: Mul<B, Output = Self>,
{
    fn dot_product(a: impl IntoIterator<Item = A>, b: impl IntoIterator<Item = B>) -> Self {
        zip_eq(a, b).map(|(a, b)| a * b).sum()
    }
}

impl<'b> DotProduct<ScalarField, &'b ScalarField> for ScalarField {}

impl<'a, 'b> DotProduct<&'a ScalarField, &'b ScalarField> for ScalarField {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_inversion_and_multiplication() {
        let values = vec![
            ScalarField::from(1u64),
            ScalarField::from(10u64),
            ScalarField::from(123u64),
            ScalarField::from(0u64),
            ScalarField::from(1_000_000u64),
            ScalarField::from(1u64 << 30),
            ScalarField::from(1u64 << 60),
        ];
        let m = ScalarField::from(42u64);

        let expected = values
            .iter()
            .map(|v| {
                if v.is_zero() {
                    v.clone()
                } else {
                    &m * v.inverse().unwrap()
                }
            })
            .collect_vec();
        let inverted = values.batch_inverse_and_mul(&m);

        assert_eq!(inverted, expected);
    }
}
