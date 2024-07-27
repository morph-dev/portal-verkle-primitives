use std::ops::AddAssign;

use crate::{Point, ScalarField, CRS};

pub struct Commitment {
    commitment: Point,
    commitment_hash: Option<ScalarField>,
}

impl Commitment {
    pub fn new(commitment: Point) -> Self {
        Self {
            commitment,
            commitment_hash: None,
        }
    }

    pub fn commitment(&self) -> &Point {
        &self.commitment
    }

    pub fn commitment_hash(&mut self) -> ScalarField {
        self.commitment_hash
            .get_or_insert_with(|| self.commitment.map_to_scalar_field())
            .clone()
    }

    /// Updates this commitment and returns by how much the commitment hash changed.
    ///
    /// @param diff By how much scalar changed.
    pub fn update_single(&mut self, index: u8, diff: ScalarField) -> ScalarField {
        let old_commitment_hash = self.commitment_hash();
        *self += CRS::commit_single(index, diff);
        self.commitment_hash() - old_commitment_hash
    }

    /// Updates this commitment and returns by how much the commitment hash changed.
    ///
    /// @param diff By how much each inner scalar changed.
    pub fn update(&mut self, diff: &[(u8, ScalarField)]) -> ScalarField {
        let old_commitment_hash = self.commitment_hash();
        *self += CRS::commit_sparse(diff);
        self.commitment_hash() - old_commitment_hash
    }

    pub fn zero() -> Self {
        Self::new(Point::zero())
    }

    pub fn is_zero(&self) -> bool {
        self.commitment.is_zero()
    }
}

impl AddAssign<Point> for Commitment {
    fn add_assign(&mut self, rhs: Point) {
        self.commitment += rhs;
        self.commitment_hash = None;
    }
}
