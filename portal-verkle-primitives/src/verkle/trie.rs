use std::collections::{HashMap, HashSet};

use alloy_primitives::B256;

use super::{
    nodes::{branch::BranchNode, Node},
    PathToLeaf, StemStateWrite,
};
use crate::{
    ssz::TriePath,
    verkle::{error::VerkleTrieError, StateWrites},
    Point, Stem, TrieKey, TrieValue,
};

/// Fully in-memory implementation of the Verkle Trie.
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

    pub fn root_commitment(&self) -> &Point {
        self.root_node.commitment().as_point()
    }

    pub fn root(&self) -> B256 {
        self.root_commitment().into()
    }

    pub fn get(&self, key: &TrieKey) -> Option<&TrieValue> {
        self.root_node.get(key)
    }

    pub fn insert(&mut self, key: &TrieKey, value: TrieValue) {
        let stem_state_write = StemStateWrite {
            stem: key.stem(),
            writes: HashMap::from([(key.suffix(), value)]),
        };
        self.root_node.update(&stem_state_write);
    }

    pub fn update(&mut self, state_writes: &StateWrites) -> HashSet<TriePath> {
        let mut created_branches = HashSet::new();
        for stem_state_write in state_writes.iter() {
            if let Some(created_branch) = self.root_node.update(stem_state_write).1 {
                created_branches.insert(created_branch);
            }
        }
        created_branches
    }

    pub fn traverse_to_leaf<'me>(
        &'me self,
        stem: &Stem,
    ) -> Result<PathToLeaf<'me>, VerkleTrieError> {
        let mut branches = vec![];

        let mut node = &self.root_node;
        let mut depth = 0;

        loop {
            let child_index = stem[depth];
            branches.push((node, child_index));
            node = match node.get_child(child_index) {
                Node::Empty => return Err(VerkleTrieError::NodeNotFound { stem: *stem, depth }),
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, str::FromStr};

    use alloy_primitives::{address, b256, keccak256};

    use crate::{
        constants::{
            BALANCE_LEAF_KEY, CODE_KECCAK_LEAF_KEY, HEADER_STORAGE_OFFSET, NONCE_LEAF_KEY,
            VERSION_LEAF_KEY,
        },
        verkle::{genesis_config::GenesisConfig, storage::AccountStorageLayout},
    };

    use super::*;

    fn read_genesis() -> GenesisConfig {
        let reader = BufReader::new(File::open("../testdata/genesis.json").unwrap());
        serde_json::from_reader(reader).unwrap()
    }

    #[test]
    fn devnet6_genesis() {
        let genesis_config = read_genesis();

        let mut trie = VerkleTrie::new();
        trie.update(&genesis_config.into_state_writes());

        assert_eq!(trie.root(), GenesisConfig::DEVNET6_STATE_ROOT)
    }

    #[test]
    fn devnet6_block1() {
        let genesis_config = read_genesis();
        let block1_state_root =
            b256!("5a65582e323fb83ed40438a0c33fa6ebfbc7f45e4c29d112b0142cfeb63f82af");

        let mut trie = VerkleTrie::new();
        trie.update(&genesis_config.into_state_writes());

        let storage_layout =
            AccountStorageLayout::new(address!("fffffffffffffffffffffffffffffffffffffffe"));
        let stem_state_write = StemStateWrite {
            stem: *storage_layout.account_storage_stem(),
            writes: HashMap::from([
                (VERSION_LEAF_KEY, TrieValue::ZERO),
                (BALANCE_LEAF_KEY, TrieValue::ZERO),
                (NONCE_LEAF_KEY, TrieValue::ZERO),
                (CODE_KECCAK_LEAF_KEY, TrieValue::from(keccak256([]))),
                (
                    HEADER_STORAGE_OFFSET.byte(0),
                    TrieValue::from_str(
                        "0x3fe165c03e7a77d1e3759362ebeeb16fd964cb411ce11fbe35c7032fab5b9a8a",
                    )
                    .unwrap(),
                ),
            ]),
        };

        let new_branch_nodes = trie.update(&StateWrites(vec![stem_state_write]));
        assert_eq!(
            new_branch_nodes,
            [TriePath::new(vec![0x5b]).unwrap()].into()
        );
        assert_eq!(trie.root(), block1_state_root);
    }
}
