use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

pub use ipa::*;
pub use multiproof::*;
pub use prover_query::*;
pub use verifier_query::*;

mod ipa;
pub mod lagrange_basis;
mod multiproof;
pub mod precomputed_weights;
mod prover_query;
pub mod transcript;
mod verifier_query;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Constructor, Deref,
)]
#[serde(transparent)]
#[ssz(struct_behaviour = "transparent")]
pub struct BundleProof(MultiProof);
