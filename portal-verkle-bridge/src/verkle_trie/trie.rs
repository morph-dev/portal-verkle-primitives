use std::collections::HashSet;

use alloy_primitives::B256;
use portal_verkle_trie::nodes::portal::ssz::TriePath;
use verkle_core::{Stem, TrieKey, TrieValue};

use super::{
    error::VerkleTrieError,
    nodes::{branch::BranchNode, leaf::LeafNode, Node},
};
use crate::types::state_write::StateWrites;

/// Fully in memory implementation of the Verkle Trie.
///
/// Primary use case is to update the trie based on the `ExecutionWitness`.
pub struct VerkleTrie {
    root_node: BranchNode,
}

pub struct PathToLeaf<'a> {
    pub branches: Vec<&'a BranchNode>,
    pub leaf: &'a LeafNode,
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

    pub fn update(
        &mut self,
        state_writes: &StateWrites,
    ) -> Result<HashSet<TriePath>, VerkleTrieError> {
        let mut created_branches = HashSet::new();
        for stem_state_write in state_writes.iter() {
            if let Some(created_branch) = self.root_node.update(stem_state_write)? {
                created_branches.insert(created_branch);
            }
        }
        Ok(created_branches)
    }

    pub fn traverse_to_leaf<'me>(
        &'me self,
        stem: &Stem,
    ) -> Result<PathToLeaf<'me>, VerkleTrieError> {
        let mut branches = vec![];

        let mut node = &self.root_node;
        let mut depth = 0;

        loop {
            branches.push(node);
            node = match node.get_child(stem[depth] as usize) {
                Node::Empty => return Err(VerkleTrieError::NodeNotFound),
                Node::Branch(next_node) => next_node,
                Node::Leaf(leaf) => {
                    if leaf.stem() == stem {
                        return Ok(PathToLeaf { branches, leaf });
                    } else {
                        return Err(VerkleTrieError::UnexpectedStem {
                            expected: *stem,
                            actual: *leaf.stem(),
                        });
                    }
                }
            };
            depth += 1;
        }
    }
}

impl Default for VerkleTrie {
    fn default() -> Self {
        Self::new()
    }
}
