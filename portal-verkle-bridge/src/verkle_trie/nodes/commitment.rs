use std::ops::{Add, AddAssign};

use verkle_core::{Point, ScalarField};

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

    pub fn zero() -> Self {
        Self::new(Point::zero())
    }

    pub fn is_zero(&self) -> bool {
        self.commitment.is_zero()
    }
}

impl Add for Commitment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.commitment + rhs.commitment)
    }
}

impl AddAssign<Point> for Commitment {
    fn add_assign(&mut self, rhs: Point) {
        self.commitment += rhs;
        self.commitment_hash = None;
    }
}
