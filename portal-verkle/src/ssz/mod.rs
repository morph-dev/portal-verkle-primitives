pub use sparse_vector::SparseVector;
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, VariableList};

use crate::{proof::MultiPointProof, Point};

mod sparse_vector;

pub type TriePath = VariableList<u8, typenum::U30>;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TrieProof {
    pub commitments_by_path: VariableList<Point, typenum::U32>,
    pub multi_point_proof: MultiPointProof,
}
