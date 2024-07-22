use std::{array, iter};

use itertools::{chain, zip_eq};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{typenum, FixedVector};

use crate::{
    constants::{VERKLE_NODE_WIDTH, VERKLE_NODE_WIDTH_BITS},
    ec::CRS,
    proof::precomputed_weights::PrecomputedWeights,
    BatchInversion, DotProduct, Point, ScalarField,
};

use super::{lagrange_basis::LagrangeBasis, transcript::Transcript};

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

impl IpaProof {
    /// Opens given polynomial with commitment `C` at `x`.
    ///
    /// The `x` is outside of the domain.
    pub fn open_polynomial(
        c: Option<Point>,
        polynomial: LagrangeBasis,
        x: ScalarField,
        transcript: &mut Transcript,
    ) -> Self {
        // 0. Prepare variables
        let c = c.unwrap_or_else(|| polynomial.commit());
        let mut a_orig = polynomial.evaluations().clone();
        let mut b_orig = PrecomputedWeights::evaluate_lagrange_polynomials(&x);
        let mut g_orig = CRS::bases().clone();

        let y = zip_eq(&a_orig, &b_orig).map(|(a, b)| a * b).sum();

        let mut a = a_orig.as_mut_slice();
        let mut b = b_orig.as_mut_slice();
        let mut g = g_orig.as_mut_slice();

        let mut cls = Vec::with_capacity(VERKLE_NODE_WIDTH_BITS);
        let mut crs = Vec::with_capacity(VERKLE_NODE_WIDTH_BITS);

        // 1. Compute w and Q
        transcript.domain_sep("ipa");
        transcript.append_point("C", &c);
        transcript.append_scalar("input point", &x);
        transcript.append_scalar("output point", &y);
        let w = transcript.challenge_scalar("w");

        // Q = CRS.Q * w
        let q = CRS::generator().mul(&w);

        // 2. Run reduction
        let mut n = VERKLE_NODE_WIDTH;
        while n > 1 {
            let n_half = n / 2;

            // 2.1 Split a, b, G
            let (a_l, a_r) = a.split_at_mut(n_half);
            let (b_l, b_r) = b.split_at_mut(n_half);
            let (g_l, g_r) = g.split_at_mut(n_half);

            // 2.2 Compute z_l and z_r

            // z_l = sum(a_r_i * b_l_i)
            let z_l = ScalarField::dot_product(a_r.iter(), b_l.iter());
            // z_r = sum(a_l_i * b_r_i)
            let z_r = ScalarField::dot_product(a_l.iter(), b_r.iter());

            // 2.3 Compute, commit to, and save C_l and C_r

            // C_l = sum(a_r_i * G_l_i) + z_l * Q
            let c_l = Point::multi_scalar_mul(
                g_l.iter().chain(iter::once(&q)),
                a_r.iter().chain(iter::once(&z_l)),
            );
            transcript.append_point("L", &c_l);
            cls.push(c_l);

            // C_r = sum(a_l_i * G_r_i) + z_r * Q
            let c_r = Point::multi_scalar_mul(
                g_r.iter().chain(iter::once(&q)),
                a_l.iter().chain(iter::once(&z_r)),
            );
            transcript.append_point("R", &c_r);
            crs.push(c_r);

            // 2.4 Compute challenge x
            let x = transcript.challenge_scalar("x");
            let x_inv = x.inverse().expect("x shouldn't be zero");

            // 2.5 Reduce a, b, G
            for i in 0..n_half {
                a_l[i] += &a_r[i] * &x;
                b_l[i] += &b_r[i] * &x_inv;
                g_l[i] += g_r[i].mul(&x_inv);
            }
            a = a_l;
            b = b_l;
            g = g_l;

            // 2.6 Update n
            n = n_half;
        }

        Self {
            cl: FixedVector::new(cls).expect("The cl should have expected length"),
            cr: FixedVector::new(crs).expect("The cr should have expected length"),
            final_evaluation: a[0].clone(),
        }
    }

    /// Verify that polynomial with commitment `C` evaluates to `y` at `x`.
    pub fn verify_polynomial(
        &self,
        c: Point,
        x: ScalarField,
        y: ScalarField,
        transcript: &mut Transcript,
    ) -> bool {
        // 0. Prepare variables
        let a_prime = &self.final_evaluation;
        let q = CRS::generator();
        let b = PrecomputedWeights::evaluate_lagrange_polynomials(&x);

        // 1. Compute w
        // NOTE: we should multiply Q with it, but we will do it later
        transcript.domain_sep("ipa");
        transcript.append_point("C", &c);
        transcript.append_scalar("input point", &x);
        transcript.append_scalar("output point", &y);
        let w = transcript.challenge_scalar("w");

        // 2. Compute x_i and 1/x_i
        let x = zip_eq(&self.cl, &self.cr)
            .map(|(c_l, c_r)| {
                transcript.append_point("L", c_l);
                transcript.append_point("R", c_r);
                transcript.challenge_scalar("x")
            })
            .collect::<Vec<_>>();
        let x_inv = x.clone().batch_inverse();

        // 3. Compute scalars that we will use to multiply G_i: g_coeff
        // They are calculated by aggregating 1/x_i (x_agg_i) in a smart way.
        // We also multiply them with -a' to avoid that step in the future, as it is needed for
        // multiplication with both G and b.
        let mut g_coeff: [ScalarField; VERKLE_NODE_WIDTH] = array::from_fn(|_| -a_prime);
        for (i, g_coeff) in g_coeff.iter_mut().enumerate() {
            for bit_index in 0..VERKLE_NODE_WIDTH_BITS {
                if i & (1 << bit_index) != 0 {
                    let x_index = VERKLE_NODE_WIDTH_BITS - 1 - bit_index;
                    *g_coeff *= &x_inv[x_index];
                }
            }
        }

        // 4. Compute coefficient next to Q
        // q_coeff = w * (y + sum(b_i * (-a' * x_agg_i))) = w * (y + sum(b_i * g_coeff_i)))
        let q_coeff = w * (y + ScalarField::dot_product(b, &g_coeff));

        // 5. Verify that equation holds:
        // C + Q * w * y + sum(Cl_i * x_i) + sum(Cr_i / x_i) = G' * a' + Q * w * (a'b')
        // Where G' = sum(G_i * x_agg_i) and b' = sum(b_i * x_agg_i)
        // Simplified, this is equal to:
        // C + Q * q_coeff + sum(Cl_i * x_i) + sum(Cr_i / x_i) + sum(G_i * g_coeff_i) = 0
        let c_coeff = ScalarField::one();
        let result = Point::multi_scalar_mul(
            chain!(iter::once(&c), iter::once(q), &self.cl, &self.cr),
            chain!(iter::once(&c_coeff), iter::once(&q_coeff), &x, &x_inv),
        ) + CRS::commit(&g_coeff);
        result.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_polynomial() {
        let x = ScalarField::from(1234u64);
        let y = ScalarField::from(42u64);
        let poly = LagrangeBasis::new_const(&y);
        let c = poly.commit();
        let mut transcript = Transcript::new("test");
        let proof = IpaProof::open_polynomial(Some(c.clone()), poly, x.clone(), &mut transcript);

        let mut transcript = Transcript::new("test");
        assert!(proof.verify_polynomial(c, x, y, &mut transcript))
    }

    #[test]
    fn simple_polynomial() {
        // poly = (x+1)(x+10)(x-100)
        let eval = |x: &ScalarField| {
            (x + ScalarField::from(1u64))
                * (x + ScalarField::from(10u64))
                * (x - ScalarField::from(100u64))
        };
        let x = ScalarField::from(1234u64);
        let y = eval(&x);
        let poly = LagrangeBasis::new(array::from_fn(|i| eval(&ScalarField::from(i))));
        let c = poly.commit();
        let mut transcript = Transcript::new("test");
        let proof = IpaProof::open_polynomial(Some(c.clone()), poly, x.clone(), &mut transcript);

        let mut transcript = Transcript::new("test");
        assert!(proof.verify_polynomial(c, x, y, &mut transcript))
    }
}
