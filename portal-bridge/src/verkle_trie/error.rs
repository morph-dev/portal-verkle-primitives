use thiserror::Error;
use verkle_core::{Stem, TrieValue};

#[derive(Debug, Error)]
pub enum VerkleTrieError {
    #[error("Expected stem {expected}, but received {actual}")]
    UnexpectedStem { expected: Stem, actual: Stem },
    #[error(
        "Wrong old value for stem {stem} at index {index}. Expected {expected:?}, received {actual:?}"    )]
    WrongOldValue {
        stem: Stem,
        index: u8,
        expected: Option<TrieValue>,
        actual: Option<TrieValue>,
    },
    #[error("Node not found during the trie traversal")]
    NodeNotFound,
}
