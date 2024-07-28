use derive_more::{Constructor, Deref, DerefMut, IntoIterator};
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, VariableList};

use crate::{Point, Stem};

pub use sparse_vector::SparseVector;

mod sparse_vector;

pub type TriePath = VariableList<u8, typenum::U30>;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Encode, Decode, Constructor, Deref, DerefMut, IntoIterator,
)]
#[ssz(struct_behaviour = "transparent")]
pub struct TriePathCommitments(VariableList<Point, typenum::U31>);

impl TriePathCommitments {
    pub fn root(&self) -> Option<&Point> {
        self.first()
    }

    pub fn zip_with_stem(&self, stem: &Stem) -> impl IntoIterator<Item = (Point, u8)> {
        self.iter()
            .zip(stem)
            .map(|(commitment, child_index)| (commitment.clone(), *child_index))
            .collect::<Vec<_>>()
    }
}

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Encode, Decode, Constructor, Deref, DerefMut, IntoIterator,
)]
#[ssz(struct_behaviour = "transparent")]
pub struct TriePathWithCommitments(VariableList<(Point, u8), typenum::U30>);

impl TriePathWithCommitments {
    pub fn root(&self) -> Option<&Point> {
        self.first().map(|(commitment, _)| commitment)
    }
}

impl TryFrom<Vec<(Point, u8)>> for TriePathWithCommitments {
    type Error = String;

    fn try_from(value: Vec<(Point, u8)>) -> Result<Self, Self::Error> {
        VariableList::new(value)
            .map(Self)
            .map_err(|_| "Provided vector is too long".to_string())
    }
}
