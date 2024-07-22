use banderwagon::CanonicalSerialize;
use sha2::{Digest, Sha256};

use crate::{Point, ScalarField};

pub struct Transcript {
    hasher: Sha256,
}

impl Transcript {
    pub fn new(label: &str) -> Self {
        Self {
            hasher: Sha256::new_with_prefix(label),
        }
    }

    pub fn domain_sep(&mut self, label: &str) {
        self.hasher.update(label);
    }

    pub fn append_point(&mut self, label: &str, point: &Point) {
        self.hasher.update(label);
        point
            .inner()
            .serialize_compressed(&mut self.hasher)
            .expect("Point should serialize to hasher");
    }

    pub fn append_scalar(&mut self, label: &str, scalar: &ScalarField) {
        self.hasher.update(label);
        scalar
            .inner()
            .serialize_compressed(&mut self.hasher)
            .expect("Scalar should serialize to hasher");
    }

    pub fn challenge_scalar(&mut self, label: &str) -> ScalarField {
        self.domain_sep(label);

        let scalar = ScalarField::from_le_bytes_mod_order(&self.hasher.finalize_reset());
        self.append_scalar(label, &scalar);
        scalar
    }
}

#[cfg(test)]
mod tests {
    // Based on https://github.com/crate-crypto/verkle-trie-ref/blob/2332ab82a77c17024861eb47fd140301c27de980/ipa/transcript_test.py

    use alloy_primitives::b256;

    use super::*;

    #[test]
    fn test_vector_0() {
        let mut transcript = Transcript::new("simple_protocol");
        let first_challenge = transcript.challenge_scalar("simple_challenge");
        let second_challenge = transcript.challenge_scalar("simple_challenge");
        // We can never even accidentally, generate the same challenge
        assert_ne!(first_challenge, second_challenge)
    }

    #[test]
    fn test_vector_1() {
        let mut transcript = Transcript::new("simple_protocol");
        let challenge = transcript.challenge_scalar("simple_challenge");

        let expected = b256!("c2aa02607cbdf5595f00ee0dd94a2bbff0bed6a2bf8452ada9011eadb538d003");

        assert_eq!(expected, challenge.to_be_bytes())
    }

    #[test]
    fn test_vector_2() {
        let mut transcript = Transcript::new("simple_protocol");
        let five = ScalarField::from(5u64);

        transcript.append_scalar("five", &five);
        transcript.append_scalar("five again", &five);

        let challenge = transcript.challenge_scalar("simple_challenge");

        let expected = b256!("498732b694a8ae1622d4a9347535be589e4aee6999ffc0181d13fe9e4d037b0b");

        assert_eq!(expected, challenge.to_be_bytes())
    }

    #[test]
    fn test_vector_3() {
        let mut transcript = Transcript::new("simple_protocol");
        let one = ScalarField::from(1u64);
        let minus_one = -one.clone();

        transcript.append_scalar("-1", &minus_one);
        transcript.domain_sep("separate me");
        transcript.append_scalar("-1 again", &minus_one);
        transcript.domain_sep("separate me again");
        transcript.append_scalar("now 1", &one);

        let challenge = transcript.challenge_scalar("simple_challenge");

        let expected = b256!("14f59938e9e9b1389e74311a464f45d3d88d8ac96adf1c1129ac466de088d618");

        assert_eq!(expected, challenge.to_be_bytes())
    }

    #[test]
    fn test_vector_4() {
        let mut transcript = Transcript::new("simple_protocol");
        let generator = Point::prime_subgroup_generator();

        transcript.append_point("generator", &generator);

        let challenge = transcript.challenge_scalar("simple_challenge");

        let expected = b256!("8c2dafe7c0aabfa9ed542bb2cbf0568399ae794fc44fdfd7dff6cc0e6144921c");

        assert_eq!(expected, challenge.to_be_bytes())
    }
}
