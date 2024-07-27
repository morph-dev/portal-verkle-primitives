use std::{array, mem};

use crate::{
    constants::VERKLE_NODE_WIDTH,
    ssz::TriePath,
    verkle::{NewBranchNode, StemStateWrite},
    Point, ScalarField, Stem, TrieKey, TrieValue,
};

use super::{commitment::Commitment, leaf::LeafNode, Node};

pub struct BranchNode {
    depth: usize,
    commitment: Commitment,
    children: [Node; VERKLE_NODE_WIDTH],
}

impl BranchNode {
    pub fn new(depth: usize) -> Self {
        if depth >= Stem::len_bytes() {
            panic!("Invalid branch depth!")
        }
        Self {
            depth,
            commitment: Commitment::zero(),
            children: array::from_fn(|_| Node::Empty),
        }
    }

    pub fn commitment(&self) -> &Point {
        self.commitment.commitment()
    }

    pub fn commitment_hash(&mut self) -> ScalarField {
        self.commitment.commitment_hash()
    }

    pub fn get(&self, key: &TrieKey) -> Option<&TrieValue> {
        let index = key[self.depth] as usize;
        match &self.children[index] {
            Node::Empty => None,
            Node::Branch(branch_node) => branch_node.get(key),
            Node::Leaf(leaf_node) => {
                if key.starts_with_stem(leaf_node.stem()) {
                    leaf_node.get(key.suffix())
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn get_child(&self, index: u8) -> &Node {
        &self.children[index as usize]
    }

    fn set_child(&mut self, index: u8, mut child: Node) {
        self.commitment.update_single(
            index,
            child.commitment_hash() - self.children[index as usize].commitment_hash(),
        );
        self.children[index as usize] = child;
    }

    /// Returns by how much the commitmant hash has changed and the path to the new branch node if
    /// one was created.
    pub fn update(&mut self, state_write: &StemStateWrite) -> (ScalarField, NewBranchNode) {
        let index = state_write.stem[self.depth];
        let child = &mut self.children[index as usize];
        match child {
            Node::Empty => {
                let mut leaf_node = Box::new(LeafNode::new(state_write.stem));
                leaf_node.update(&state_write.writes);
                *child = Node::Leaf(leaf_node);
                (
                    self.commitment
                        .update_single(index, child.commitment_hash()),
                    None,
                )
            }
            Node::Branch(branch_node) => {
                let (commitment_hash_diff, new_branch_node) = branch_node.update(state_write);
                (
                    self.commitment.update_single(index, commitment_hash_diff),
                    new_branch_node,
                )
            }
            Node::Leaf(leaf_node) => {
                if leaf_node.stem() == &state_write.stem {
                    let commitment_hash_diff = leaf_node.update(&state_write.writes);
                    (
                        self.commitment.update_single(index, commitment_hash_diff),
                        None,
                    )
                } else {
                    let old_commitment_hash = leaf_node.commitment_hash();

                    let old_child_index_in_new_branch = leaf_node.stem()[self.depth + 1];
                    let old_child = mem::replace(child, Node::Empty);

                    let mut branch_node = Box::new(Self::new(self.depth + 1));
                    branch_node.set_child(old_child_index_in_new_branch, old_child);
                    branch_node.update(state_write);

                    let new_branch_node = Some(TriePath::from(
                        state_write.stem[..branch_node.depth].to_vec(),
                    ));
                    *child = Node::Branch(branch_node);
                    (
                        self.commitment
                            .update_single(index, child.commitment_hash() - old_commitment_hash),
                        new_branch_node,
                    )
                }
            }
        }
    }
}
