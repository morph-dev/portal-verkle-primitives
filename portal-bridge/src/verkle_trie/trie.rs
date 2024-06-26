use alloy_primitives::B256;
use banderwagon::Element;
use verkle_core::utils::serialize_to_b256;

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

    pub fn commitment(&self) -> Element {
        *self.root_node.commitment()
    }

    pub(super) fn root_node(&self) -> &BranchNode {
        &self.root_node
    }

    pub fn root(&self) -> B256 {
        serialize_to_b256(&self.commitment()).unwrap()
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
