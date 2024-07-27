use derive_more::{Deref, DerefMut, IntoIterator};

use crate::{Point, ScalarField};

pub struct VerifierQuery {
    /// The commitment of the polynomial
    pub commitment: Point,
    /// The index of the opening (from domain)
    pub z: u8,
    /// The value at the opening `y=P(z)`
    pub y: ScalarField,
}

#[derive(Default, Deref, DerefMut, IntoIterator)]
pub struct VerifierMultiQuery(Vec<VerifierQuery>);

impl VerifierMultiQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trie_path_proof(
        &mut self,
        trie_path_iter: impl IntoIterator<Item = (Point, u8)>,
        last_child_commitment: &Point,
    ) {
        let mut trie_path_iter = trie_path_iter.into_iter().peekable();
        while let Some((commitment, child_index)) = trie_path_iter.next() {
            let child_commitment = trie_path_iter
                .peek()
                .map(|(commitment, _)| commitment)
                .unwrap_or(last_child_commitment);
            self.push(VerifierQuery {
                commitment,
                z: child_index,
                y: child_commitment.map_to_scalar_field(),
            })
        }
    }

    pub fn add_for_commitment(
        &mut self,
        commitment: &Point,
        children: impl IntoIterator<Item = (u8, ScalarField)>,
    ) {
        self.extend(
            children
                .into_iter()
                .map(|(child_index, child_value)| VerifierQuery {
                    commitment: commitment.clone(),
                    z: child_index,
                    y: child_value,
                }),
        );
    }
}

impl FromIterator<VerifierQuery> for VerifierMultiQuery {
    fn from_iter<T: IntoIterator<Item = VerifierQuery>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
