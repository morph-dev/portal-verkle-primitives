use alloy_primitives::B256;
use banderwagon::{CanonicalDeserialize, CanonicalSerialize, SerializationError};

pub fn serialize_to_b256<T: CanonicalSerialize>(data: T) -> Result<B256, SerializationError> {
    if data.compressed_size() != B256::len_bytes() {
        return Err(SerializationError::InvalidData);
    }

    let mut result = B256::ZERO;
    data.serialize_compressed(result.as_mut_slice())?;
    Ok(result)
}

pub fn deserialize_from_b256<T: CanonicalDeserialize>(data: B256) -> Result<T, SerializationError> {
    T::deserialize_compressed(data.as_slice())
}
