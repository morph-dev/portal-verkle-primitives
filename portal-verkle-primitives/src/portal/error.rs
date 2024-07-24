use thiserror::Error;

use crate::Point;

#[derive(Debug, Error)]
pub enum NodeVerificationError {
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
}

impl NodeVerificationError {
    pub fn wrong_commitment(expected: &Point, actual: &Point) -> Self {
        Self::WrongCommitment {
            expected: expected.clone().into(),
            actual: actual.clone().into(),
        }
    }
}
