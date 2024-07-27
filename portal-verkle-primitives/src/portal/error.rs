use alloy_primitives::B256;
use thiserror::Error;

use crate::Point;

#[derive(Debug, Error)]
pub enum NodeVerificationError {
    #[error("The root doesn't match. expected: {expected:?} actual: {actual:?}")]
    WrongRoot {
        expected: Box<B256>,
        actual: Box<B256>,
    },

    #[error("Commitment doesn't match. expected: {expected:?} actual: {actual:?}")]
    WrongCommitment {
        expected: Box<Point>,
        actual: Box<Point>,
    },

    #[error("Commitment is zero")]
    ZeroCommitment,

    #[error("Bundle node doesn't have fragments")]
    NoFragments,

    #[error("Child's value is zero")]
    ZeroChild,

    #[error("Bundle proof is invalid")]
    InvalidBundleProof,

    #[error("MultiPointProof is invalid")]
    InvalidMultiPointProof,

    #[error("Invalid fragment index: {0}")]
    InvalidFragmentIndex(u8),
}

impl NodeVerificationError {
    pub fn new_wrong_commitment(expected: &Point, actual: &Point) -> Self {
        Self::WrongCommitment {
            expected: expected.clone().into(),
            actual: actual.clone().into(),
        }
    }
    pub fn new_wrong_root(expected: B256, actual: B256) -> Self {
        Self::WrongRoot {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}
