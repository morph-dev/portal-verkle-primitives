use std::borrow::Cow;

use itertools::{zip_eq, Itertools};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

use crate::{constants::VERKLE_NODE_WIDTH, BatchInversion, DotProduct, Point, ScalarField};

use super::{
    lagrange_basis::LagrangeBasis, transcript::Transcript, IpaProof, ProverMultiQuery,
    VerifierMultiQuery,
};

/// The multi-point proof based on IPA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(deny_unknown_fields)]
pub struct MultiProof {
    #[serde(alias = "ipaProof")]
    pub ipa_proof: IpaProof,

    /// The commitment to the polynomial g(x): `D=[g(x)]`
    #[serde(alias = "gX")]
    pub g_commitment: Point,
}

impl MultiProof {
    pub fn create_portal_network_proof(multiquery: ProverMultiQuery) -> Self {
        Self::create(
            multiquery,
            &mut Transcript::new(Transcript::PORTAL_NETWORK_LABEL),
        )
    }

    pub fn create(multiquery: ProverMultiQuery, transcript: &mut Transcript) -> Self {
        transcript.domain_sep("multiproof");

        // 1. Compute r
        // r = H(C_0, z_0, y_0,  C_1, z_1, y_1,  ...,  C_m, z_m, y_m)
        for query in multiquery.iter() {
            let commitment: Cow<Point> = match query.commitment.as_ref() {
                Some(commitment) => Cow::Borrowed(commitment),
                None => Cow::Owned(query.poly.commit()),
            };
            transcript.append_point("C", &commitment);
            transcript.append_scalar("z", &ScalarField::from(query.z));
            transcript.append_scalar("y", query.poly.evaluate_in_domain(query.z))
        }
        let r = transcript.challenge_scalar("r");

        // 2. Compute g(x)
        // g(x) = ∑ r^i * (f_i(x) - y_i) / (x - z_i)

        // r^i
        let powers_of_r = ScalarField::powers_of(&r, multiquery.len());

        // Aggregate scaled polynomials by opening point `z`
        // HashMap<z, sum(r^i * f_i(x))> ; z = z_i
        let agg_polynomials = zip_eq(&powers_of_r, multiquery)
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
        assert!(t >= ScalarField::from(VERKLE_NODE_WIDTH));

        // 5. Compute h(x)
        // h(x) = ∑ r^i * f_i(x) / (t - z_i)

        // 1 / (t - z_i)
        let inverse_denominators = agg_polynomials
            .keys()
            .map(|z| &t - ScalarField::from(*z))
            .collect_vec()
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

    pub fn verify_portal_network_proof(&self, multiquery: VerifierMultiQuery) -> bool {
        self.verify(
            multiquery,
            &mut Transcript::new(Transcript::PORTAL_NETWORK_LABEL),
        )
    }

    pub fn verify(&self, multiquery: VerifierMultiQuery, transcript: &mut Transcript) -> bool {
        transcript.domain_sep("multiproof");

        // Extract D (commitment to g(x): D = [g(x)])
        let d = &self.g_commitment;

        // 1. Compute r
        // r = H(C_0, z_0, y_0,  C_1, z_1, y_1,  ...,  C_m, z_m, y_m)
        for query in multiquery.iter() {
            transcript.append_point("C", &query.commitment);
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
        let powers_of_r = ScalarField::powers_of(&r, multiquery.len());

        // 1 / (t - z_i)
        let inverse_denominators = multiquery
            .iter()
            .map(|query| &t - ScalarField::from(query.z))
            .collect_vec()
            .batch_inverse();

        // coeff_i
        let coefficients = zip_eq(powers_of_r, inverse_denominators)
            .map(|(power_of_r, inverse_denominator)| power_of_r * inverse_denominator)
            .collect_vec();

        // 4. Compute E (commitment to h(x): E = [h(x)])
        // E = ∑ r^i * C_i / (t - z_i) = ∑ coeff_i * C_i
        let e = Point::multi_scalar_mul(
            multiquery.iter().map(|query| &query.commitment),
            &coefficients,
        );
        transcript.append_point("E", &e);

        // 4. Compute p(t)
        // p(t) = ∑ r^i * y_i / (t - z_i) = ∑ coeff_i * y_i
        let p_t = ScalarField::dot_product(coefficients, multiquery.iter().map(|query| &query.y));

        // 5. Compute commitment to p(x)
        // [p(x)] = [h(x)] - [g(x)] = E - D
        let p_comm = e - d;

        // 4. Verify IPA proof
        self.ipa_proof.verify_polynomial(p_comm, t, p_t, transcript)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{hex::FromHex, B256};

    use crate::{
        constants::{LEAF_MARKER_INDEX, LEAF_STEM_INDEX},
        proof::{prover_query::ProverQuery, VerifierQuery},
        utils::array_long,
        Stem,
    };

    use super::*;

    type PolynomialFn = Box<dyn Fn(u8) -> ScalarField>;

    #[test]
    fn simple_multi_opening() {
        // p(x) = 42
        let f_const = |_: u8| ScalarField::from(42u64);
        // p(x) = (x + 1)(x + 10)(x - 100)
        let f_poly = |x: u8| {
            let x = ScalarField::from(x);
            (&x + ScalarField::from(1u64))
                * (&x + ScalarField::from(10u64))
                * (&x - ScalarField::from(100u64))
        };
        // p(x) = (x + 2) / (x + 1) (on domain)
        let f_non_poly = |x: u8| {
            let x = ScalarField::from(x);
            (&x + ScalarField::from(2u64)) * (&x + ScalarField::from(1u64)).inverse().unwrap()
        };

        let openings: Vec<(PolynomialFn, u8)> = vec![
            (Box::new(f_const), 13),
            (Box::new(f_poly), 42),
            (Box::new(f_non_poly), 183),
            (Box::new(f_non_poly), 42), // same z
        ];

        let mut transcript = Transcript::new("test");
        let queries = openings
            .iter()
            .map(|(poly_f, z)| {
                let poly = LagrangeBasis::new(array_long(poly_f));
                ProverQuery {
                    poly,
                    commitment: None,
                    z: *z,
                }
            })
            .collect();
        let proof = MultiProof::create(queries, &mut transcript);

        let mut transcript = Transcript::new("test");
        let queries = openings
            .iter()
            .map(|(poly_f, z)| VerifierQuery {
                commitment: LagrangeBasis::new(array_long(poly_f)).commit(),
                z: *z,
                y: poly_f(*z),
            })
            .collect();
        assert!(proof.verify(queries, &mut transcript));
    }

    /// Proof from the kaustinen devnet-6, block 1
    #[test]
    fn devnet_6_block_1() -> anyhow::Result<()> {
        let ipa_proof = r#"{
            "cl": [
                "0x6e3104792843f10a7236b6648f0d32d9ede087d55a3c8705038ed5358e073904",
                "0x5eceadbf8532a2c626366356e9dd870a9e83bc0393ea088a32a8dc7452291cc3",
                "0x13bf721c1aa1cdf84ea69161114bbe0eda90f51173ac00dabdfc4e6aeed301a4",
                "0x21a7dfe85100ed2e0c6b1f07ff07ad9660711d25ff7ffdd7a7a02b3363089df9",
                "0x0abe44837167a603c0ab167071c7d01fab67d0b81b0f8c2c195445f6a0063301",
                "0x3f4e616c904c91bc4211dfb14f33528f498fe76c1acc85491ef253afd4ce2cbb",
                "0x08631ce02176c4dcf83dbd04c68adf17d70090f2a2d367085b35a733baadf991",
                "0x44b4f57ed5ea4fdb1cd97901aab184b9a3d743bc3732e3e9c3b1e29370eb24d8"
            ],
            "cr": [
                "0x6206979e2aa815bff1c76c521d025229e5867ba2d67422bd54d993a471aebbf8",
                "0x11bbacadfad4a06e85a7509ac601693646d1dc6416fb06c14f0745276579d548",
                "0x6b34ee89647bf02642546d0c2438fc34f350dae628ba033efbce2d9efe5c714b",
                "0x2eaf5b365d0071a04b0bf880df7cc4fefb88bb7e42693fedff0b94f4a4faab18",
                "0x3506e34c2e7686856cd0f4485f51f965aa926f235ee2b4c632b2239ffd0c9ff1",
                "0x18fbb94e59a02d289cf2d886f365cce91b89416d43e0a44d548cc6fd790b8d06",
                "0x31e28a78851612861fede858f62f7bbbfa12d02a302fea0272e03e668f0cb5a9",
                "0x48585ea2484b2b902f38297e14491311d8c74002f3eaeca9ac43bf03cfc090b5"
            ],
            "final_evaluation": "0x08a3079093df751fc850f3805e10cc898314688111df75aa20511c32198a93e3"
        }"#;
        let proof = MultiProof {
            ipa_proof: serde_json::from_str::<IpaProof>(ipa_proof)?,
            g_commitment: Point::from(&B256::from_hex(
                "0x5af46ab3e8676b9d4de8ae0be9670c45e9afd43cc11524c7946728268652028a",
            )?),
        };

        let root = Point::from(&B256::from_hex(
            "0x1fbf85345a3cbba9a6d44f991b721e55620a22397c2a93ee8d5011136ac300ee",
        )?);
        let other_leaf = Point::from(&B256::from_hex(
            "0x26715ff22c071fdd9d9c2c6b5f5bf9bb0d83a2087e1366deabcecdf2a1d3f82e",
        )?);
        let other_stem =
            Stem::from_hex("0x5bdf12f5e17d2911dac2d2b0fc9e64a3ddc1d1ea4fc2568fe7e741ff2daa18")?;

        let mut queries = VerifierMultiQuery::new();
        queries.extend([
            VerifierQuery {
                commitment: root,
                z: 0x5b,
                y: other_leaf.map_to_scalar_field(),
            },
            VerifierQuery {
                commitment: other_leaf.clone(),
                z: LEAF_MARKER_INDEX,
                y: ScalarField::one(),
            },
            VerifierQuery {
                commitment: other_leaf,
                z: LEAF_STEM_INDEX,
                y: ScalarField::from(&other_stem),
            },
        ]);
        let mut transcript = Transcript::new("vt");
        assert!(
            proof.verify(queries, &mut transcript),
            "Multiproof should verify"
        );
        Ok(())
    }
}
