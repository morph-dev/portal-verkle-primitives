use itertools::{zip_eq, Itertools};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

use crate::{constants::VERKLE_NODE_WIDTH, BatchInversion, DotProduct, Point, ScalarField};

use super::{lagrange_basis::LagrangeBasis, transcript::Transcript, IpaProof};

pub struct ProverQuery {
    pub poly: LagrangeBasis,
    pub c: Point,
    pub z: usize,
}

pub struct VerifierQuery {
    pub c: Point,
    pub z: usize,
    pub y: ScalarField,
}

/// The multi-point proof based on IPA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(deny_unknown_fields)]
pub struct MultiPointProof {
    #[serde(alias = "ipaProof")]
    pub ipa_proof: IpaProof,

    /// The commitment to the polynomial g(x): `D=[g(x)]`
    #[serde(alias = "gX")]
    pub g_commitment: Point,
}

impl MultiPointProof {
    pub fn create(queries: Vec<ProverQuery>, transcript: &mut Transcript) -> Self {
        transcript.domain_sep("multiproof");

        // 1. Compute r
        // r = H(C_0, z_0, y_0,  C_1, z_1, y_1,  ...,  C_m, z_m, y_m)
        for query in &queries {
            transcript.append_point("C", &query.c);
            transcript.append_scalar("z", &ScalarField::from(query.z));
            transcript.append_scalar("y", query.poly.evaluate_in_domain(query.z))
        }
        let r = transcript.challenge_scalar("r");

        // 2. Compute g(x)
        // g(x) = ∑ r^i * (f_i(x) - y_i) / (x - z_i)

        // r^i
        let powers_of_r = ScalarField::powers_of(&r, queries.len());

        // Aggregate scaled polynomials by opening point `z`
        // HashMap<z, sum(r^i * f_i(x))> ; z = z_i
        let agg_polynomials = zip_eq(&powers_of_r, queries)
            .map(|(power_of_r, query)| (query.z, query.poly * power_of_r))
            .into_grouping_map()
            .sum();

        // g(x)
        let g: LagrangeBasis = agg_polynomials
            .iter()
            .map(|(z, poly)| poly.divide_on_domain(*z))
            .sum();

        // 3. Commit to g(x): D=[g(x)]
        let d = g.commit();
        transcript.append_point("D", &d);

        // 4. Compute t
        // t = H(r, D)
        let t = transcript.challenge_scalar("t");

        // NOTE: Check that t is not in the domain!
        assert!(t > ScalarField::from(VERKLE_NODE_WIDTH));

        // 5. Compute h(x)
        // h(x) = ∑ r^i * f_i(x) / (t - z_i)

        // 1 / (t - z_i)
        let inverse_denominators = agg_polynomials
            .keys()
            .map(|z| &t - ScalarField::from(*z))
            .collect::<Vec<_>>()
            .batch_inverse();

        // h(x)
        let h: LagrangeBasis = zip_eq(agg_polynomials.into_values(), inverse_denominators)
            .map(|(poly, inverse_denominator)| poly * inverse_denominator)
            .sum();

        // 6. Commit to h(x): E=[h(x)]
        let e = h.commit();
        transcript.append_point("E", &e);

        // 7. Compute p(x) and its commitment
        // p(x) = h(x) - g(x)
        // [p(x)] = [h(x)] - [g(x)] = E - D
        let p = h - g;
        let p_comm = e - &d;

        // 8. Create IPA proof for polynomial p(x) at point t
        let ipa = IpaProof::open_polynomial(Some(p_comm), p, t, transcript);

        Self {
            ipa_proof: ipa,
            g_commitment: d,
        }
    }

    pub fn verify(&self, queries: &[VerifierQuery], transcript: &mut Transcript) -> bool {
        transcript.domain_sep("multiproof");

        // Extract D (commitment to g(x): D = [g(x)])
        let d = &self.g_commitment;

        // 1. Compute r
        // r = H(C_0, z_0, y_0,  C_1, z_1, y_1,  ...,  C_m, z_m, y_m)
        for query in queries {
            transcript.append_point("C", &query.c);
            transcript.append_scalar("z", &ScalarField::from(query.z));
            transcript.append_scalar("y", &query.y)
        }
        let r = transcript.challenge_scalar("r");

        // 2. Compute t
        // t = H(r, D)
        transcript.append_point("D", d);
        let t = transcript.challenge_scalar("t");

        // 3. Compute coefficients coeff_i
        // coeff_i = r^i / (t - z_i)

        // r^i
        let powers_of_r = ScalarField::powers_of(&r, queries.len());

        // 1 / (t - z_i)
        let inverse_denominators = queries
            .iter()
            .map(|query| &t - ScalarField::from(query.z))
            .collect::<Vec<_>>()
            .batch_inverse();

        // coeff_i
        let coefficients = zip_eq(powers_of_r, inverse_denominators)
            .map(|(power_of_r, inverse_denominator)| power_of_r * inverse_denominator)
            .collect::<Vec<_>>();

        // 4. Compute E (commitment to h(x): E = [h(x)])
        // E = ∑ r^i * C_i / (t - z_i) = ∑ coeff_i * C_i
        let e = Point::multi_scalar_mul(queries.iter().map(|query| &query.c), &coefficients);
        transcript.append_point("E", &e);

        // 4. Compute p(t)
        // p(t) = ∑ r^i * y_i / (t - z_i) = ∑ coeff_i * y_i
        // let p_t = zip_eq(coefficients, queries)
        //     .map(|(coeff, query)| coeff * &query.y)
        //     .sum();
        let p_t = ScalarField::dot_product(coefficients, queries.iter().map(|query| &query.y));

        // 5. Compute commitment to p(x)
        // [p(x)] = [h(x)] - [g(x)] = E - D
        let p_comm = e - d;

        // 4. Verify IPA proof
        self.ipa_proof.verify_polynomial(p_comm, t, p_t, transcript)
    }
}
