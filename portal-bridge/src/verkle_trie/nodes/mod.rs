use branch::BranchNode;
use leaf::LeafNode;
use verkle_core::{Point, ScalarField};

pub mod branch;
pub mod commitment;
pub mod leaf;

pub enum Node {
    Empty,
    Branch(Box<BranchNode>),
    Leaf(Box<LeafNode>),
}

impl Node {
    pub fn commitment(&self) -> Point {
        match self {
            Node::Empty => Point::zero(),
            Node::Branch(branch_node) => branch_node.commitment().clone(),
            Node::Leaf(leaf_node) => leaf_node.commitment().clone(),
        }
    }

    pub fn commitment_hash(&mut self) -> ScalarField {
        match self {
            Node::Empty => ScalarField::zero(),
            Node::Branch(branch_node) => branch_node.commitment_hash(),
            Node::Leaf(leaf_node) => leaf_node.commitment_hash(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }
}
