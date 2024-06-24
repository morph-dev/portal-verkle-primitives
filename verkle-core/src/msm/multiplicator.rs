use banderwagon::{msm::MSMPrecompWnaf, Element, Fr, Zero};
use once_cell::sync::Lazy;

use crate::{constants::VERKLE_NODE_WIDTH, msm::crs::CRS};

static PRECOMP_WNAF_WINDOW_SIZE: usize = 12;

/// Precomputed wNAF (w-ary non-adjacent form) tables for efficient scalar multiplication.
static WNAF_PRECOMP: Lazy<MSMPrecompWnaf> =
    Lazy::new(|| MSMPrecompWnaf::new(CRS.as_slice(), PRECOMP_WNAF_WINDOW_SIZE));

/// The trait fhe Multi-scalar Multiplication (MSM).
pub trait MultiScalarMultiplicator {
    /// Commit to a lagrange polynomial.
    ///
    /// The `evaluations.len()` must equal `CRS.len()` at the moment.
    fn commit_lagrange(&self, evaluations: &[Fr]) -> Element;

    /// Single scalar multiplication.
    fn scalar_mul(&self, index: usize, value: Fr) -> Element;

    /// Commit to sparse set of scalars.`
    fn commit_sparse(&self, evaluations: &[(usize, Fr)]) -> Element {
        // TODO: consider if 64 is good value
        if evaluations.len() >= 64 {
            let mut dense = [Fr::zero(); VERKLE_NODE_WIDTH];
            for (index, value) in evaluations {
                dense[*index] = *value;
            }
            self.commit_lagrange(&dense)
        } else {
            let mut result = Element::zero();
            for (index, value) in evaluations {
                result += self.scalar_mul(*index, *value)
            }
            result
        }
    }
}

/// Default implementation of [`MultiScalarMultiplicator`].
pub struct DefaultMsm;

impl MultiScalarMultiplicator for DefaultMsm {
    fn commit_lagrange(&self, evaluations: &[Fr]) -> Element {
        // Preliminary benchmarks indicate that the parallel version is faster
        // for vectors of length 64 or more
        if evaluations.len() >= 64 {
            WNAF_PRECOMP.mul_par(evaluations)
        } else {
            WNAF_PRECOMP.mul(evaluations)
        }
    }

    fn scalar_mul(&self, index: usize, value: Fr) -> Element {
        WNAF_PRECOMP.mul_index(value, index)
    }
}
