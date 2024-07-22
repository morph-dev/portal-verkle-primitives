use derive_more::{AsRef, Constructor, Deref};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

mod ipa;
pub mod lagrange_basis;
mod multiproof;
pub mod precomputed_weights;
pub mod transcript;

pub use ipa::IpaProof;
pub use multiproof::MultiPointProof;

#[derive(
    Constructor, AsRef, Deref, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(transparent)]
#[ssz(struct_behaviour = "transparent")]
pub struct BundleProof(MultiPointProof);
