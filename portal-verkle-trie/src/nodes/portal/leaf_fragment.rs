use verkle_core::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    msm::{DefaultMsm, MultiScalarMultiplicator},
    Point, TrieValue, TrieValueSplit,
};

pub struct LeafFragmentNode {
    parent_index: usize,
    commitment: Point,
    children: [Option<TrieValue>; PORTAL_NETWORK_NODE_WIDTH],
}

impl LeafFragmentNode {
    pub fn new(parent_index: usize) -> Self {
        Self::new_with_children(parent_index, <_>::default())
    }

    pub fn new_with_children(
        parent_index: usize,
        children: [Option<TrieValue>; PORTAL_NETWORK_NODE_WIDTH],
    ) -> Self {
        if parent_index >= PORTAL_NETWORK_NODE_WIDTH {
            panic!("Invalid parent index: {parent_index}")
        }

        let commitment = DefaultMsm.commit_sparse(
            children
                .iter()
                .enumerate()
                .filter_map(|(child_index, child)| child.as_ref().map(|child| (child_index, child)))
                .flat_map(|(child_index, child)| {
                    let (low_index, high_index) = Self::bases_indices(parent_index, child_index);
                    let (low_value, high_value) = child.split();
                    [(low_index, low_value), (high_index, high_value)]
                })
                .collect::<Vec<_>>()
                .as_slice(),
        );

        Self {
            parent_index,
            commitment,
            children,
        }
    }

    pub fn commitment(&self) -> &Point {
        &self.commitment
    }

    pub fn set(&mut self, child_index: usize, child: TrieValue) {
        let (new_low_value, new_high_value) = child.split();
        let old_value = self.children[child_index].replace(child);
        let (old_low_value, old_high_value) = old_value.split();

        let (low_index, high_index) = Self::bases_indices(self.parent_index, child_index);

        self.commitment += DefaultMsm.commit_sparse(&[
            (low_index, new_low_value - old_low_value),
            (high_index, new_high_value - old_high_value),
        ]);
    }

    pub fn get(&self, index: usize) -> Option<&TrieValue> {
        self.children[index].as_ref()
    }

    /// Returns the bases indices that correspond to the child index.
    fn bases_indices(parent_index: usize, child_index: usize) -> (usize, usize) {
        let starting_index =
            parent_index % (PORTAL_NETWORK_NODE_WIDTH / 2) * 2 * PORTAL_NETWORK_NODE_WIDTH;
        let low_index = starting_index + 2 * child_index;
        let high_index = low_index + 1;
        (low_index, high_index)
    }
}
