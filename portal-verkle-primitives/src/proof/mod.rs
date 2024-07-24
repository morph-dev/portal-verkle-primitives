use derive_more::{AsRef, Constructor, Deref};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, VariableList};

use crate::Point;

pub use ipa::IpaProof;
pub use multiproof::MultiPointProof;

mod ipa;
pub mod lagrange_basis;
mod multiproof;
pub mod precomputed_weights;
pub mod transcript;

#[derive(
    Constructor, AsRef, Deref, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(transparent)]
#[ssz(struct_behaviour = "transparent")]
pub struct BundleProof(MultiPointProof);

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TrieProof {
    pub commitments_by_path: VariableList<Point, typenum::U32>,
    pub multi_point_proof: MultiPointProof,
}
