use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, Element, SerializationError};

pub fn serialize_to_b256(value: &Element) -> Result<B256, SerializationError> {
    if value.compressed_size() != B256::len_bytes() {
        return Err(SerializationError::InvalidData);
    }

    let mut result = B256::ZERO;
    value.serialize_compressed(result.as_mut_slice())?;
    Ok(result)
}

pub fn deserialize_from_b256(data: B256) -> Result<Element, SerializationError> {
    Element::deserialize_compressed(data.as_slice())
}
