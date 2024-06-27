use std::io::Write;

use alloy_primitives::hex::{encode, encode_prefixed};
use verkle_core::constants::VERKLE_NODE_WIDTH;

use super::{
    nodes::{branch::BranchNode, leaf::LeafNode, Node},
    VerkleTrie,
};

pub trait TriePrinter {
    fn print_state<W: Write>(&self, writer: &mut W) -> std::io::Result<()>;

    fn print_trie<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.print_trie_with_identation(writer, 0)
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> anyhow::Result<()>;
}

impl TriePrinter for VerkleTrie {
    fn print_state<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.root_node().print_state(writer)?;
        writer.flush()
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> anyhow::Result<()> {
        writeln!(writer, "{:identation$}root - {}", "", self.root())?;
        self.root_node()
            .print_trie_with_identation(writer, identation + 2)
    }
}

impl TriePrinter for Node {
    fn print_state<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
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
    ) -> anyhow::Result<()> {
        match self {
            Node::Empty => Ok(()),
            Node::Branch(branch_node) => branch_node.print_trie_with_identation(writer, identation),
            Node::Leaf(leaf_node) => leaf_node.print_trie_with_identation(writer, identation),
        }
    }
}

impl TriePrinter for BranchNode {
    fn print_state<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for index in 0..VERKLE_NODE_WIDTH {
            self.get_child(index).print_state(writer)?;
        }
        Ok(())
    }

    fn print_trie_with_identation<W: Write>(
        &self,
        writer: &mut W,
        identation: usize,
    ) -> anyhow::Result<()> {
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
    fn print_state<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
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
    ) -> anyhow::Result<()> {
        writeln!(writer, "{:identation$}stem - {}", "", self.stem())?;

        writeln!(writer, "{:identation$}C1 - {:?}", "", self.c1.commitment())?;
        for index in 0..(VERKLE_NODE_WIDTH / 2) {
            if let Some(value) = self.get(index) {
                writeln!(
                    writer,
                    "{:identation$}{index:02x} - {}",
                    "",
                    encode_prefixed(value.as_slice()),
                    identation = identation + 2,
                )?;
            }
        }

        writeln!(writer, "{:identation$}C2 - {:?}", "", self.c2.commitment())?;
        for index in (VERKLE_NODE_WIDTH / 2)..VERKLE_NODE_WIDTH {
            if let Some(value) = self.get(index) {
                writeln!(
                    writer,
                    "{:identation$}{index:02x} - {}",
                    "",
                    encode_prefixed(value.as_slice()),
                    identation = identation + 2,
                )?;
            }
        }

        Ok(())
    }
}
