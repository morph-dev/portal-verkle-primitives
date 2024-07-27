use std::array;

use once_cell::sync::Lazy;

use crate::{constants::VERKLE_NODE_WIDTH, BatchInversion, ScalarField};

/// Precomputed weights for Lagrange polynomial (`L_i`) related calculations.
///
/// Domain `D` is `[0, 255]` => `d = 256``.
/// ```text
/// Lagrange polynomials:
///              x-j
/// L_i(x) = ∏  -----
///         j≠i  i-j
///
/// A(x) = ∏ (x-i) = (x-0)(x-1)...(x-(d-1))
///        i
///
/// A'(x) = ∑ ( ∏ (x-j) )
///         i  j≠i
///       = (x-1)(x-2)...(x-(d-1)) +
///         (x-0)(x-2)...(x-(d-1)) +
///         ...
///         (x-0)(x-1)...(x-(d-2))
///
/// Lagrange polynomials in barycentric form
///              x-j         A(x)
/// L_i(x) = ∏  ----- = ---------------
///         j≠i  i-j     A'(i) * (x-i)
/// ```
pub struct PrecomputedWeights {
    /// The `A'(i)`, for i in domain
    ///
    /// ```text
    /// A'(i) =  ∏ (i-j)
    ///         j≠i
    /// ```
    a_prime: [ScalarField; VERKLE_NODE_WIDTH],
    /// The `1/A'(i)` , for i in domain
    a_prime_inv: [ScalarField; VERKLE_NODE_WIDTH],
    /// The `1/i` , for i in domain (except when i is zero, in which case value is zero)
    domain_inv: [ScalarField; VERKLE_NODE_WIDTH],
}

static INSTANCE: Lazy<PrecomputedWeights> = Lazy::new(PrecomputedWeights::new);

impl PrecomputedWeights {
    fn new() -> Self {
        let a_prime = array::from_fn(|i| {
            //  ∏ (i-j)
            // j≠i
            (0..VERKLE_NODE_WIDTH)
                .filter(|j| i != *j)
                .map(|j| ScalarField::from(i) - ScalarField::from(j))
                .product()
        });

        let a_prime_inv = a_prime.clone().batch_inverse();

        let domain_inv = array::from_fn(ScalarField::from).batch_inverse();

        Self {
            a_prime,
            a_prime_inv,
            domain_inv,
        }
    }

    /// Evaluates polynomial `A` at a given point `z`
    ///
    /// `A(z) = ∏ (z - i) = (z-0)(z-1)...(z-d)`
    pub fn evaluate_a(z: &ScalarField) -> ScalarField {
        (0..VERKLE_NODE_WIDTH)
            .map(|i| z - ScalarField::from(i))
            .product()
    }

    /// Returns `A'(i)` for i in domain
    pub fn a_prime(i: u8) -> &'static ScalarField {
        &INSTANCE.a_prime[i as usize]
    }

    /// Returns `1/A'(i)` for i in domain
    pub fn a_prime_inv(i: u8) -> &'static ScalarField {
        &INSTANCE.a_prime_inv[i as usize]
    }

    pub fn domain_inv(i: u8) -> &'static ScalarField {
        assert_ne!(i, 0);
        &INSTANCE.domain_inv[i as usize]
    }

    /// Evaluates Lagrange polynomials `L_i` at a given point `z`, using barycentric formula.
    ///
    /// ```text
    ///              z - j          A(z)
    /// L_i(z) = ∏  ------- = -----------------
    ///         j≠i  i - j     A'(i) * (z - i)
    /// ```
    pub fn evaluate_lagrange_polynomials(z: &ScalarField) -> [ScalarField; VERKLE_NODE_WIDTH] {
        // A(z) = (z-0)(z-1)(z-2)...(z-d)
        let a_z = Self::evaluate_a(z);

        // A'(i) * (z-i)
        let lagrange_evaluations: [ScalarField; VERKLE_NODE_WIDTH] =
            array::from_fn(|i| (z - ScalarField::from(i)) * Self::a_prime(i as u8));

        // A(z) / (A'(i) * (z-i))
        lagrange_evaluations.batch_inverse_and_mul(&a_z)
    }
}
