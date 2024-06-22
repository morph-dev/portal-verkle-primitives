use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Fr};
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer>(value: &Fr, serializer: S) -> Result<S::Ok, S::Error> {
    let mut bytes = B256::default();
    value
        .serialize_compressed(bytes.as_mut_slice())
        .map_err(serde::ser::Error::custom)?;
    bytes.reverse();
    serializer.serialize_bytes(bytes.as_slice())
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Fr, D::Error> {
    let mut bytes = B256::deserialize(deserializer)?;
    bytes.reverse();
    Fr::deserialize_compressed(bytes.as_slice()).map_err(serde::de::Error::custom)
}
