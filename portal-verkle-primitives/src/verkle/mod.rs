use std::collections::{HashMap, HashSet};

use derive_more::{Constructor, Deref, Index};

use crate::{ssz::TriePath, Stem, TrieValue};

use nodes::{branch::BranchNode, leaf::LeafNode};
pub use trie::VerkleTrie;

pub mod error;
pub mod genesis_config;
pub mod nodes;
pub mod storage;
mod trie;
pub mod trie_printer;

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, Index)]
pub struct StateWrites(Vec<StemStateWrite>);

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct StemStateWrite {
    pub stem: Stem,
    pub writes: HashMap<u8, TrieValue>,
}

pub type NewBranchNode = Option<TriePath>;

#[derive(Clone)]
pub struct AuxiliaryTrieModifications {
    pub new_branch_nodes: HashSet<TriePath>,
}

#[derive(Clone)]
pub struct PathToLeaf<'a> {
    pub branches: Vec<&'a BranchNode>,
    pub leaf: &'a LeafNode,
}
