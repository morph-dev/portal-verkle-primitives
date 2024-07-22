use std::{array, iter::Sum, ops};

use itertools::Itertools;
use overload::overload;

use crate::{constants::VERKLE_NODE_WIDTH, DotProduct, Point, ScalarField, CRS};

use super::precomputed_weights::PrecomputedWeights;

/// The polynomial expressed using Lagrange's form.
///
/// Some operations are non-optimal when polynomials are stored in coeficient (monomial) basis:
/// ```text
/// P(x) = v0 + v_1 * X + ... + x_n * X^n
/// ```
///
/// Another way to represent the polynomial is using Langrange basis, in which we store the value
/// that polynomial has on a given domain (in our case domain is `[0, 255]`). More precisely:
///
/// ```text
/// P(x) = y_0 * L_0(x) + y_1 * L_1(x) + y_n * L_n(x)
/// ```
///
/// Where `y_i` is the evaluation of the polynomial at `i`: `y_i = P(i)` and `L_i(x)` is Lagrange
/// polynomial:
///
/// ```text
///              x-j
/// L_i(x) = ∏  -----
///         j≠i  i-j
/// ```
#[derive(Clone, Debug)]
pub struct LagrangeBasis {
    y: [ScalarField; VERKLE_NODE_WIDTH],
}

impl LagrangeBasis {
    pub fn new(values: [ScalarField; VERKLE_NODE_WIDTH]) -> Self {
        Self { y: values }
    }

    pub fn new_const(value: &ScalarField) -> Self {
        Self::new(array::from_fn(|_| value.clone()))
    }

    pub fn zero() -> Self {
        Self::new_const(&ScalarField::zero())
    }

    pub fn commit(&self) -> Point {
        CRS::commit(&self.y)
    }

    /// Returns evaluations on the domain
    pub fn evaluations(&self) -> &[ScalarField; VERKLE_NODE_WIDTH] {
        &self.y
    }

    /// Calculates `P(z)` for `z` in domain
    pub fn evaluate_in_domain(&self, z: usize) -> &ScalarField {
        &self.y[z]
    }

    /// Calculates `P(z)` for `z` outside domain
    pub fn evaluate_outside_domain(&self, z: &ScalarField) -> ScalarField {
        // Lagrange polinomials: L_i(z)
        let l = PrecomputedWeights::evaluate_lagrange_polynomials(z);
        ScalarField::dot_product(l, &self.y)
    }

    /// Divides the polynomial `P(x)-P(k)` with `x-k`, where `k` is in domain.
    ///
    /// Let's call new polynomial `Q(x)`. We evaluate it on domain manually:
    ///
    /// - `i ≠ k`
    /// ```text
    /// Q(i) = (y_i - y_k) / (i - k)
    /// ```
    /// - `i = k` - This case can be transofrmed using non obvious math tricks into:
    /// ```text
    /// Q(k) = ∑ -Q(j) * A'(k) / A'(j)
    ///       j≠k
    /// ```
    pub fn divide_on_domain(&self, k: usize) -> Self {
        let mut q = array::from_fn(|_| ScalarField::zero());
        for i in 0..VERKLE_NODE_WIDTH {
            // 1/(i-k)
            let inverse = match i {
                i if i < k => -PrecomputedWeights::domain_inv(k - i),
                i if i == k => continue,
                i if i > k => PrecomputedWeights::domain_inv(i - k).clone(),
                _ => unreachable!(),
            };

            // Q(i) = (y_i-y_k) / (i-k)
            q[i] = (&self.y[i] - &self.y[k]) * inverse;

            q[k] -= &q[i] * PrecomputedWeights::a_prime(k) * PrecomputedWeights::a_prime_inv(i);
        }

        Self::new(q)
    }
}

impl From<&[ScalarField]> for LagrangeBasis {
    fn from(other: &[ScalarField]) -> Self {
        assert!(other.len() == VERKLE_NODE_WIDTH);
        Self {
            y: array::from_fn(|i| other[i].clone()),
        }
    }
}

overload!((lhs: &mut LagrangeBasis) += (rhs: ?LagrangeBasis) {
    lhs.y.iter_mut().zip_eq(&rhs.y).for_each(|(lhs, rhs)| *lhs += rhs)
});
overload!((lhs: LagrangeBasis) + (rhs: ?LagrangeBasis) -> LagrangeBasis {
    let mut lhs = lhs; lhs += rhs; lhs
});

overload!((lhs: &mut LagrangeBasis) -= (rhs: ?LagrangeBasis) {
    lhs.y.iter_mut().zip_eq(&rhs.y).for_each(|(lhs, rhs)| *lhs -= rhs)
});
overload!((lhs: LagrangeBasis) - (rhs: ?LagrangeBasis) -> LagrangeBasis {
    let mut lhs = lhs; lhs -= rhs; lhs
});

overload!((lhs: &mut LagrangeBasis) *= (rhs: ScalarField) {
    lhs.y.iter_mut().for_each(|lhs| *lhs *= &rhs)
});
overload!((lhs: &mut LagrangeBasis) *= (rhs: &ScalarField) {
    lhs.y.iter_mut().for_each(|lhs| *lhs *= rhs)
});
overload!((lhs: LagrangeBasis) * (rhs: ?ScalarField) -> LagrangeBasis {
    let mut lhs = lhs; lhs *= rhs; lhs
});

impl Sum for LagrangeBasis {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_else(Self::zero)
    }
}

impl<'a> Sum<&'a LagrangeBasis> for LagrangeBasis {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(LagrangeBasis::zero(), |sum, poly| sum + poly)
    }
}
