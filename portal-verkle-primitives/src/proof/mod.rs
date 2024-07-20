use derive_more::{AsRef, Constructor, Deref};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, FixedVector};

use crate::{Point, ScalarField};

pub mod lagrange_basis;
pub mod precomputed_weights;
pub mod transcript;

#[derive(
    Constructor, AsRef, Deref, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(transparent)]
#[ssz(struct_behaviour = "transparent")]
pub struct BundleProof(MultiPointProof);

/// The multi-point proof based on IPA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(deny_unknown_fields)]
pub struct MultiPointProof {
    #[serde(alias = "ipaProof")]
    pub ipa_proof: IpaProof,
    #[serde(alias = "gX")]
    pub g_x: Point,
}

/// The inner product argument proof.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(deny_unknown_fields)]
pub struct IpaProof {
    pub cl: FixedVector<Point, typenum::U8>,
    pub cr: FixedVector<Point, typenum::U8>,
    #[serde(alias = "finalEvaluation")]
    pub final_evaluation: ScalarField,
}

// TODO: Add this to Derive once ssz_types updates.
impl Eq for IpaProof {}
