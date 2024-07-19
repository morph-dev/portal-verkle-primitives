use std::array;

use banderwagon::msm::MSMPrecompWnaf;
use once_cell::sync::Lazy;

use crate::{constants::VERKLE_NODE_WIDTH, msm::crs::CRS, Point, ScalarField};

static PRECOMP_WNAF_WINDOW_SIZE: usize = 12;

/// Precomputed wNAF (w-ary non-adjacent form) tables for efficient scalar multiplication.
static WNAF_PRECOMP: Lazy<MSMPrecompWnaf> = Lazy::new(|| {
    MSMPrecompWnaf::new(
        &CRS.each_ref().map(|point| point.element()),
        PRECOMP_WNAF_WINDOW_SIZE,
    )
});

/// The trait fhe Multi-scalar Multiplication (MSM).
pub trait MultiScalarMultiplicator {
    /// Commit to a lagrange polynomial.
    ///
    /// The `evaluations.len()` must equal `CRS.len()` at the moment.
    fn commit_lagrange(&self, scalars: &[ScalarField; VERKLE_NODE_WIDTH]) -> Point;

    /// Single scalar multiplication.
    fn scalar_mul(&self, index: usize, scalar: ScalarField) -> Point;

    /// Commit to sparse set of scalars.`
    fn commit_sparse(&self, scalars: &[(usize, ScalarField)]) -> Point {
        // TODO: consider if 64 is good value
        if scalars.len() >= 64 {
            let mut dense: [ScalarField; VERKLE_NODE_WIDTH] =
                array::from_fn(|_| ScalarField::zero());
            for (index, value) in scalars {
                dense[*index] = value.clone();
            }
            self.commit_lagrange(&dense)
        } else {
            let mut result = Point::zero();
            for (index, value) in scalars {
                result += self.scalar_mul(*index, value.clone())
            }
            result
        }
    }
}

/// Default implementation of [`MultiScalarMultiplicator`].
pub struct DefaultMsm;

impl MultiScalarMultiplicator for DefaultMsm {
    fn commit_lagrange(&self, scalars: &[ScalarField; VERKLE_NODE_WIDTH]) -> Point {
        // Preliminary benchmarks indicate that the parallel version is faster
        // for vectors of length 64 or more
        let scalars = scalars.each_ref().map(|scalar| scalar.0);
        if scalars.len() >= 64 {
            WNAF_PRECOMP.mul_par(&scalars).into()
        } else {
            WNAF_PRECOMP.mul(&scalars).into()
        }
    }

    fn scalar_mul(&self, index: usize, scalar: ScalarField) -> Point {
        WNAF_PRECOMP.mul_index(scalar.0, index).into()
    }
}
