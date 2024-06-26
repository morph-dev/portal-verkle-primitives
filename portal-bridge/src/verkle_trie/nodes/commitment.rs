use std::ops::{Add, AddAssign};

use ark_ff::Zero;
use banderwagon::{Element, Fr};

pub struct Commitment {
    commitment: Element,
    commitment_hash: Option<Fr>,
}

impl Commitment {
    pub fn new(commitment: Element) -> Self {
        Self {
            commitment,
            commitment_hash: None,
        }
    }

    pub fn commitment(&self) -> &Element {
        &self.commitment
    }

    pub fn commitment_hash(&mut self) -> &Fr {
        self.commitment_hash
            .get_or_insert_with(|| self.commitment.map_to_scalar_field())
    }
}

impl Zero for Commitment {
    fn zero() -> Self {
        Self::new(Element::zero())
    }

    fn is_zero(&self) -> bool {
        self.commitment.is_zero()
    }
}

impl Add for Commitment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.commitment + rhs.commitment)
    }
}

impl AddAssign<Element> for Commitment {
    fn add_assign(&mut self, rhs: Element) {
        self.commitment += rhs;
        self.commitment_hash = None;
    }
}
