use branch::BranchNode;
use commitment::Commitment;
use leaf::LeafNode;
use once_cell::sync::Lazy;

pub mod branch;
pub mod commitment;
pub mod leaf;
pub mod portal_branch_node_builder;

pub enum Node {
    Empty,
    Branch(Box<BranchNode>),
    Leaf(Box<LeafNode>),
}
pub static ZERO: Lazy<Commitment> = Lazy::new(Commitment::zero);

impl Node {
    pub fn commitment(&self) -> &Commitment {
        match self {
            Node::Empty => &ZERO,
            Node::Branch(branch_node) => branch_node.commitment(),
            Node::Leaf(leaf_node) => leaf_node.commitment(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }
}
