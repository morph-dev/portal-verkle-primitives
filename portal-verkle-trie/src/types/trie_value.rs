use alloy_primitives::B256;
use derive_more::{Constructor, Deref, From, Index};

#[derive(PartialEq, Eq, Clone, Copy, Constructor, Index, Deref, From)]
pub struct TrieValue(B256);
