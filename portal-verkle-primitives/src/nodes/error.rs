use thiserror::Error;

use crate::Point;

#[derive(Debug, Error)]
pub enum NodeVerificationError {
    #[error("Commitment doesn't match. expected: {expected:?} actual: {actual:?}")]
    WrongCommitment {
        expected: Box<Point>,
        actual: Box<Point>,
    },

    #[error("Bundle proof is invalid")]
    InvalidBundleProof,
}

impl NodeVerificationError {
    pub fn new_wrong_commitment(expected: &Point, actual: &Point) -> Self {
        Self::WrongCommitment {
            expected: expected.clone().into(),
            actual: actual.clone().into(),
        }
    }
}
