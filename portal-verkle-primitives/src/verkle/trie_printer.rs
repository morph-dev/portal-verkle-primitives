use std::io::{self, Write};

use alloy_primitives::hex::{encode, encode_prefixed};

use super::{
    nodes::{branch::BranchNode, leaf::LeafNode, Node},
    VerkleTrie,
};
use crate::constants::VERKLE_NODE_WIDTH;

pub trait TriePrinter {
    /// Prints all Trie key-value pairs.
    fn print_state<W: Write>(&self, writer: &mut W) -> io::Result<()>;

    /// Prints entire trie structure, with all intermediate commitments.
    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> io::Result<()>;
}

impl TriePrinter for VerkleTrie {
    fn print_state<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.root_node().print_state(writer)?;
        writer.flush()
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> io::Result<()> {
        writeln!(writer, "{:identation$}root - {}", "", self.root())?;
        self.root_node()
            .print_trie_with_identation(writer, identation + 2)
    }
}

impl TriePrinter for Node {
    fn print_state<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Node::Empty => Ok(()),
            Node::Branch(branch_node) => branch_node.print_state(writer),
            Node::Leaf(leaf_node) => leaf_node.print_state(writer),
        }
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> io::Result<()> {
        match self {
            Node::Empty => Ok(()),
            Node::Branch(branch_node) => branch_node.print_trie_with_identation(writer, identation),
            Node::Leaf(leaf_node) => leaf_node.print_trie_with_identation(writer, identation),
        }
    }
}

impl TriePrinter for BranchNode {
    fn print_state<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for index in 0..VERKLE_NODE_WIDTH {
            self.get_child(index).print_state(writer)?;
        }
        Ok(())
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> io::Result<()> {
        for index in 0..VERKLE_NODE_WIDTH {
            let child = self.get_child(index);
            if child.is_empty() {
                continue;
            }
            writeln!(
                writer,
                "{:identation$}{index:02x} - {:?}",
                "",
                child.commitment()
            )?;
            child.print_trie_with_identation(writer, identation + 2)?;
        }
        Ok(())
    }
}

impl TriePrinter for LeafNode {
    fn print_state<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for index in 0..VERKLE_NODE_WIDTH {
            if let Some(value) = self.get(index) {
                writeln!(
                    writer,
                    "{}{index:02x}: {}",
                    encode(self.stem()),
                    encode(value.as_slice()),
                )?;
            }
        }
        Ok(())
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> io::Result<()> {
        writeln!(writer, "{:identation$}stem - {}", "", self.stem())?;

        writeln!(writer, "{:identation$}C1 - {:?}", "", self.c1())?;
        for index in 0..(VERKLE_NODE_WIDTH / 2) {
            if let Some(value) = self.get(index) {
                writeln!(
                    writer,
                    "  {:identation$}{index:02x} - {}",
                    "",
                    encode_prefixed(value.as_slice()),
                    identation = identation + 2,
                )?;
            }
        }

        writeln!(writer, "{:identation$}C2 - {:?}", "", self.c2())?;
        for index in (VERKLE_NODE_WIDTH / 2)..VERKLE_NODE_WIDTH {
            if let Some(value) = self.get(index) {
                writeln!(
                    writer,
                    "  {:identation$}{index:02x} - {}",
                    "",
                    encode_prefixed(value.as_slice()),
                    identation = identation + 2,
                )?;
            }
        }

        Ok(())
    }
}
