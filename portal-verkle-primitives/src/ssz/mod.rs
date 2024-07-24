use ssz_types::{typenum, VariableList};

pub use sparse_vector::SparseVector;

mod sparse_vector;

pub type TriePath = VariableList<u8, typenum::U30>;
