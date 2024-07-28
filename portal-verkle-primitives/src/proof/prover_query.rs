use derive_more::{Deref, DerefMut, IntoIterator};

use crate::Point;

use super::lagrange_basis::LagrangeBasis;

#[derive(Clone)]
pub struct ProverQuery {
    /// The polynomial we are commiting to.
    pub poly: LagrangeBasis,
    /// The commitment of the polynomial (can be calculated from the polynomial as well)
    pub commitment: Option<Point>,
    /// The index of the opening (from domain)
    pub z: u8,
}

#[derive(Default, Clone, Deref, DerefMut, IntoIterator)]
pub struct ProverMultiQuery(Vec<ProverQuery>);

impl ProverMultiQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trie_path_proof(
        &mut self,
        trie_path_iter: impl IntoIterator<Item = (Point, LagrangeBasis, u8)>,
    ) {
        for (commitment, poly, child_index) in trie_path_iter {
            self.push(ProverQuery {
                poly,
                commitment: Some(commitment),
                z: child_index,
            })
        }
    }

    pub fn add_vector(
        &mut self,
        commitment: Point,
        poly: LagrangeBasis,
        openings: impl IntoIterator<Item = u8>,
    ) {
        self.extend(openings.into_iter().map(|child_index| ProverQuery {
            poly: poly.clone(),
            commitment: Some(commitment.clone()),
            z: child_index,
        }));
    }
}

impl FromIterator<ProverQuery> for ProverMultiQuery {
    fn from_iter<T: IntoIterator<Item = ProverQuery>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
