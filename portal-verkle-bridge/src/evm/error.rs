use alloy_primitives::B256;
use thiserror::Error;

use crate::verkle_trie::error::VerkleTrieError;

#[derive(Debug, Error)]
pub enum EvmError {
    #[error("Expected block {expected}, but received {actual}")]
    UnexpectedBlock { expected: u64, actual: u64 },
    #[error("Wrong state root. Expected {expected}, but actual {actual}")]
    WrongStateRoot { expected: B256, actual: B256 },
    #[error("Trie error: {0}")]
    TrieError(VerkleTrieError),
}
