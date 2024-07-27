use derive_more::{Deref, DerefMut, IntoIterator};

use crate::Point;

use super::lagrange_basis::LagrangeBasis;

pub struct ProverQuery {
    /// The polynomial we are commiting to.
    pub poly: LagrangeBasis,
    /// The commitment of the polynomial (can be calculated from the polynomial as well)
    pub commitment: Option<Point>,
    /// The index of the opening (from domain)
    pub z: u8,
}

#[derive(Default, Deref, DerefMut, IntoIterator)]
pub struct ProverMultiQuery(Vec<ProverQuery>);

impl ProverMultiQuery {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FromIterator<ProverQuery> for ProverMultiQuery {
    fn from_iter<T: IntoIterator<Item = ProverQuery>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
