use alloy_primitives::B256;
use verkle_core::{TrieKey, TrieValue};

use super::{error::VerkleTrieError, nodes::branch::BranchNode};
use crate::types::state_write::StateWrite;

/// Fully in memory implementation of the Verkle Trie.
///
/// Primary use case is to update the trie based on the `ExecutionWitness`.
pub struct VerkleTrie {
    root_node: BranchNode,
}

impl VerkleTrie {
    pub fn new() -> Self {
        Self {
            root_node: BranchNode::new(/* depth= */ 0),
        }
    }

    pub(super) fn root_node(&self) -> &BranchNode {
        &self.root_node
    }

    pub fn root(&self) -> B256 {
        self.root_node.commitment().into()
    }

    pub fn get(&self, key: &TrieKey) -> Option<&TrieValue> {
        self.root_node.get(key)
    }

    pub fn insert(&mut self, key: &TrieKey, value: TrieValue) {
        self.root_node.insert(key, value)
    }

    pub fn update(&mut self, state_write: &StateWrite) -> Result<(), VerkleTrieError> {
        for stem_state_write in state_write.iter() {
            self.root_node.update(stem_state_write)?;
        }
        Ok(())
    }
}

impl Default for VerkleTrie {
    fn default() -> Self {
        Self::new()
    }
}
