use alloy_primitives::B256;
use banderwagon::Element;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::utils::{deserialize_from_b256, serialize_to_b256};

pub fn serialize<S: Serializer>(value: &Element, serializer: S) -> Result<S::Ok, S::Error> {
    serialize_to_b256(value)
        .map_err(serde::ser::Error::custom)?
        .serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Element, D::Error> {
    deserialize_from_b256(B256::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}
