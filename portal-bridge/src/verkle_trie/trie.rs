use alloy_primitives::B256;
use verkle_core::{TrieKey, TrieValue};

use super::{error::VerkleTrieError, nodes::branch::BranchNode};
use crate::types::witness::StemStateDiff;

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

    pub fn update(&mut self, state_diffs: &[StemStateDiff]) -> Result<(), VerkleTrieError> {
        for state_diff in state_diffs.iter() {
            // Check that there is at least one state write
            if state_diff
                .suffix_diffs
                .iter()
                .any(|suffix_diff| suffix_diff.new_value.is_some())
            {
                self.root_node.update(state_diff)?;
            }
        }
        Ok(())
    }
}

impl Default for VerkleTrie {
    fn default() -> Self {
        Self::new()
    }
}
