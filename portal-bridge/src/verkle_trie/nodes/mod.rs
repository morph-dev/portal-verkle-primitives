use ark_ff::Zero;
use banderwagon::{Element, Fr};
use branch::BranchNode;
use leaf::LeafNode;

pub mod branch;
pub mod commitment;
pub mod leaf;

pub enum Node {
    Empty,
    Branch(Box<BranchNode>),
    Leaf(Box<LeafNode>),
}

impl Node {
    pub fn commitment(&self) -> Element {
        match self {
            Node::Empty => Element::zero(),
            Node::Branch(branch_node) => *branch_node.commitment(),
            Node::Leaf(leaf_node) => *leaf_node.commitment(),
        }
    }

    pub fn commitment_hash(&mut self) -> Fr {
        match self {
            Node::Empty => Fr::zero(),
            Node::Branch(branch_node) => *branch_node.commitment_hash(),
            Node::Leaf(leaf_node) => *leaf_node.commitment_hash(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }
}
