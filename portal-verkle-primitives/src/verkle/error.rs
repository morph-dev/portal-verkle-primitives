use thiserror::Error;

use crate::Stem;

#[derive(Debug, Error)]
pub enum VerkleTrieError {
    #[error("Expected stem {expected}, but received {actual}")]
    UnexpectedStem { expected: Stem, actual: Stem },
    #[error("Node not found at depth {depth} for stem {stem} during the trie traversal")]
    NodeNotFound { stem: Stem, depth: usize },
}
