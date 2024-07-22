use std::array;

use derive_more::{Constructor, Deref, Index, IndexMut};
use itertools::zip_eq;
use ssz::{Decode, Encode, BYTES_PER_LENGTH_OFFSET};

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Index, IndexMut, Deref)]
pub struct SparseVector<T, const N: usize> {
    items: [Option<T>; N],
}

impl<T, const N: usize> SparseVector<T, N> {
    pub fn iter_set_items(&self) -> impl Iterator<Item = &T> {
        self.iter().filter_map(|opt_item| opt_item.as_ref())
    }

    pub fn iter_enumerated_set_items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.iter()
            .enumerate()
            .filter_map(|(i, opt_item)| opt_item.as_ref().map(|item| (i, item)))
    }

    pub fn num_set_items(&self) -> usize {
        self.iter_set_items().count()
    }

    fn bitmap_bytes_len() -> usize {
        usize::max(1, (N + 7) / 8)
    }
}

impl<T, const N: usize> Default for SparseVector<T, N> {
    fn default() -> Self {
        Self {
            items: array::from_fn(|_| None),
        }
    }
}

impl<T: Encode, const N: usize> Encode for SparseVector<T, N> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        if T::is_ssz_fixed_len() {
            Self::bitmap_bytes_len() + self.num_set_items() * T::ssz_fixed_len()
        } else {
            Self::bitmap_bytes_len()
                + self
                    .iter_set_items()
                    .map(|item| BYTES_PER_LENGTH_OFFSET + item.ssz_bytes_len())
                    .sum::<usize>()
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let mut bitmap = vec![0u8; Self::bitmap_bytes_len()];
        let mut set_items = Vec::with_capacity(N);
        for (i, item) in self.iter_enumerated_set_items() {
            bitmap[i / 8] |= 1 << (i % 8);
            set_items.push(item);
        }

        buf.reserve(bitmap.ssz_bytes_len() + set_items.ssz_bytes_len());
        bitmap.ssz_append(buf);
        set_items.ssz_append(buf);
    }
}

impl<T: Decode, const N: usize> Decode for SparseVector<T, N> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let bitmap_bytes = vec![0; Self::bitmap_bytes_len()];
        if bytes.len() < bitmap_bytes.len() {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: bitmap_bytes.len(),
            });
        }

        let (bitmap_bytes, items_bytes) = bytes.split_at(Self::bitmap_bytes_len());
        let bitmap = Vec::<u8>::from_ssz_bytes(bitmap_bytes)?;
        let items = Vec::<T>::from_ssz_bytes(items_bytes)?;

        let set_indices = bitmap
            .iter()
            .enumerate()
            .flat_map(|(bitmap_index, bitmap)| {
                (0u8..8).filter_map(move |i| {
                    if bitmap & (1 << i) == 0 {
                        None
                    } else {
                        Some(i as usize + bitmap_index * 8)
                    }
                })
            })
            .collect::<Vec<_>>();

        if set_indices.iter().max().unwrap_or(&0) >= &N {
            return Err(ssz::DecodeError::BytesInvalid(
                "Bitmap has item outside the range".into(),
            ));
        }
        if set_indices.len() != items.len() {
            return Err(ssz::DecodeError::BytesInvalid(format!(
                "Number of bits in bitmap ({}) doesn't match number of items ({})",
                set_indices.len(),
                items.len()
            )));
        }

        let mut sparse_vector = Self::default();
        for (i, item) in zip_eq(set_indices, items) {
            sparse_vector[i] = Some(item);
        }
        Ok(sparse_vector)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Bytes;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::empty(vec![])]
    #[case::partial(vec![(3, 1 << 30), (4, (1 << 40) - 1), (5, 1 << 50), (6, (1 << 60) - 1)])]
    #[case::full(vec![
        (0, u64::pow(10, 0)),
        (1, u64::pow(10, 1)),
        (2, u64::pow(10, 2)),
        (3, u64::pow(10, 3)),
        (4, u64::pow(10, 4)),
        (5, u64::pow(10, 5)),
        (6, u64::pow(10, 6)),
        (7, u64::pow(10, 7)),
        (8, u64::pow(10, 8)),
        (9, u64::pow(10, 9)),
    ])]
    fn encode_decode_fixed_size(#[case] input: Vec<(usize, u64)>) {
        let mut sparse_vector = SparseVector::<u64, 10>::default();
        for (i, value) in input {
            sparse_vector[i] = Some(value);
        }

        let encoded = sparse_vector.as_ssz_bytes();

        assert_eq!(
            sparse_vector,
            SparseVector::<u64, 10>::from_ssz_bytes(&encoded).unwrap()
        );
    }

    #[rstest]
    #[case::empty(vec![])]
    #[case::partial(vec![(3, "333"), (4, "4444"), (5, "55555"), (8, "88888888")])]
    #[case::full(vec![
        (0, ""),
        (1, "1"),
        (2, "22"),
        (3, "333"),
        (4, "4444"),
        (5, "55555"),
        (6, "666666"),
        (7, "7777777"),
        (8, "88888888"),
        (9, "999999999"),
    ])]
    fn encode_decode_variable_size(#[case] input: Vec<(usize, &str)>) {
        let mut sparse_vector = SparseVector::<Bytes, 10>::default();
        for (i, value) in input {
            sparse_vector[i] = Some(Bytes::copy_from_slice(value.as_bytes()));
        }

        let encoded = sparse_vector.as_ssz_bytes();

        assert_eq!(
            sparse_vector,
            SparseVector::<Bytes, 10>::from_ssz_bytes(&encoded).unwrap()
        );
    }
}
