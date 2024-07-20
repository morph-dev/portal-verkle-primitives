use std::{fmt::Debug, iter::Sum, ops};

use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Element};
use derive_more::{Constructor, Deref, From, Into};
use overload::overload;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ssz::{Decode, Encode};

use crate::ScalarField;

#[derive(Clone, PartialEq, Eq, Constructor, From, Into, Deref)]
pub struct Point(Element);

impl Point {
    pub fn prime_subgroup_generator() -> Self {
        Self(Element::prime_subgroup_generator())
    }

    pub(crate) fn element(&self) -> Element {
        self.0
    }

    pub fn map_to_scalar_field(&self) -> ScalarField {
        self.0.map_to_scalar_field().into()
    }

    pub fn zero() -> Self {
        Self(Element::zero())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<&B256> for Point {
    fn from(value: &B256) -> Self {
        Self(
            Element::deserialize_compressed(value.as_slice())
                .expect("ScalarFieldValue should deserialize from B256"),
        )
    }
}

impl From<&Point> for B256 {
    fn from(ec_point: &Point) -> Self {
        let mut result = B256::ZERO;
        ec_point
            .serialize_compressed(result.as_mut_slice())
            .expect("EllipticCurvePoint should serialize to B256");
        result
    }
}

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        B256::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(&B256::deserialize(deserializer)?))
    }
}

impl Encode for Point {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        Element::compressed_serialized_size()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.serialize_compressed(buf)
            .expect("EllipticCurvePoint should serialize");
    }

    fn ssz_bytes_len(&self) -> usize {
        self.compressed_size()
    }
}

impl Decode for Point {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    fn ssz_fixed_len() -> usize {
        Element::compressed_serialized_size()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        Element::deserialize_compressed(bytes)
            .map(Self::new)
            .map_err(|err| {
                ssz::DecodeError::BytesInvalid(format!("Error decoding EllipticCurvePoint: {err}"))
            })
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        B256::from(self).fmt(f)
    }
}

overload!(- (me: ?Point) -> Point { Point(-me.0) });

overload!((lhs: &mut Point) += (rhs: ?Point) { lhs.0 += rhs.0 });
overload!((lhs: Point) + (rhs: ?Point) -> Point {
    let mut lhs = lhs; lhs += rhs; lhs
});

overload!((lhs: &mut Point) -= (rhs: ?Point) { lhs.0 = lhs.0 - rhs.0 });
overload!((lhs: Point) - (rhs: ?Point) -> Point {
    let mut lhs = lhs; lhs -= rhs; lhs
});

impl Sum for Point {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |sum, item| sum + item)
    }
}

impl<'a> Sum<&'a Self> for Point {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |sum, item| sum + item)
    }
}
