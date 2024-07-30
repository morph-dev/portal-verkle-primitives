use derive_more::{Deref, DerefMut, IntoIterator};
use itertools::Itertools;
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, VariableList};

use crate::{verkle::nodes::branch::BranchNode, Point, Stem};

pub use sparse_vector::SparseVector;

mod sparse_vector;

pub type TriePath = VariableList<u8, typenum::U30>;

#[derive(Default, Debug, Clone, PartialEq, Eq, Encode, Decode, Deref, DerefMut, IntoIterator)]
#[ssz(struct_behaviour = "transparent")]
pub struct TriePathCommitments(VariableList<Point, typenum::U31>);

impl TriePathCommitments {
    pub fn new(trie_path: Vec<Point>) -> Self {
        Self(VariableList::new(trie_path).expect("trie path shouldn't be too long"))
    }

    pub fn root(&self) -> Option<&Point> {
        self.first()
    }

    pub fn zip_with_stem(&self, stem: &Stem) -> impl IntoIterator<Item = (Point, u8)> {
        self.iter()
            .zip(stem)
            .map(|(commitment, child_index)| (commitment.clone(), *child_index))
            .collect_vec()
    }
}

impl<'a> FromIterator<(&'a BranchNode, u8)> for TriePathCommitments {
    fn from_iter<T: IntoIterator<Item = (&'a BranchNode, u8)>>(iter: T) -> Self {
        Self::new(
            iter.into_iter()
                .map(|(branch_node, _)| branch_node.commitment().to_point())
                .collect_vec(),
        )
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Encode, Decode, Deref, DerefMut, IntoIterator)]
#[ssz(struct_behaviour = "transparent")]
pub struct TriePathWithCommitments(VariableList<(Point, u8), typenum::U30>);

impl TriePathWithCommitments {
    pub fn new(trie_path: Vec<(Point, u8)>) -> Self {
        Self(VariableList::new(trie_path).expect("trie path shouldn't be too long"))
    }

    pub fn root(&self) -> Option<&Point> {
        self.first().map(|(commitment, _)| commitment)
    }
}

impl<'a> FromIterator<(&'a BranchNode, u8)> for TriePathWithCommitments {
    fn from_iter<T: IntoIterator<Item = (&'a BranchNode, u8)>>(iter: T) -> Self {
        Self::new(
            iter.into_iter()
                .map(|(branch_node, child_index)| {
                    (branch_node.commitment().to_point(), child_index)
                })
                .collect_vec(),
        )
    }
}
