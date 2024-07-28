use std::{ops::AddAssign, sync::OnceLock};

use crate::{Point, ScalarField, CRS};

#[derive(Clone)]
pub struct Commitment {
    commitment: Point,
    scalar: OnceLock<ScalarField>,
}

impl Commitment {
    pub fn new(commitment: Point) -> Self {
        Self {
            commitment,
            scalar: OnceLock::new(),
        }
    }

    pub fn as_point(&self) -> &Point {
        &self.commitment
    }

    pub fn to_point(&self) -> Point {
        self.as_point().clone()
    }

    pub fn as_scalar(&self) -> &ScalarField {
        self.scalar
            .get_or_init(|| self.commitment.map_to_scalar_field())
    }

    pub fn to_scalar(&self) -> ScalarField {
        self.as_scalar().clone()
    }

    /// Updates this commitment and returns by how much the commitment hash changed.
    ///
    /// @param diff By how much scalar changed.
    pub fn update_single(&mut self, index: u8, diff: &ScalarField) -> ScalarField {
        let old_scalar = self.to_scalar();
        *self += CRS::commit_single(index, diff);
        self.as_scalar() - old_scalar
    }

    /// Updates this commitment and returns by how much the commitment hash changed.
    ///
    /// @param diff By how much each inner scalar changed.
    pub fn update(&mut self, diff: &[(u8, ScalarField)]) -> ScalarField {
        let old_scalar = self.to_scalar();
        *self += CRS::commit_sparse(diff);
        self.as_scalar() - old_scalar
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
        self.scalar = OnceLock::new();
    }
}
