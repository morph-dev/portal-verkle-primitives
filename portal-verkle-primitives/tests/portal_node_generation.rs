use std::{collections::HashSet, fs::File, io::BufReader};

use alloy_primitives::B256;
use portal_verkle_primitives::{
    constants::PORTAL_NETWORK_NODE_WIDTH,
    verkle::{
        genesis_config::GenesisConfig,
        nodes::{
            branch::BranchNode, portal_branch_node_builder::PortalBranchNodeBuilder,
            portal_leaf_node_builder::PortalLeafNodeBuilder,
        },
        PathToLeaf, VerkleTrie,
    },
};

fn read_genesis() -> GenesisConfig {
    let reader = BufReader::new(File::open("../testdata/genesis.json").unwrap());
    serde_json::from_reader(reader).unwrap()
}

#[test]
fn leaves() {
    let state_writes = read_genesis().into_state_writes();

    let mut trie = VerkleTrie::new();
    trie.update(&state_writes);

    for state_write in state_writes.iter() {
        let stem = state_write.stem;
        println!("Starting stem: {stem}");

        let path_to_leaf = trie.traverse_to_leaf(&stem).unwrap();
        check_leaf(&path_to_leaf);
    }
}

fn check_leaf(path_to_leaf: &PathToLeaf) {
    let leaf_node_builder = PortalLeafNodeBuilder::new(path_to_leaf);

    println!(
        "Leaf bundle {:?}",
        path_to_leaf.leaf.commitment().as_point()
    );
    let bundle_node = leaf_node_builder.bundle_node_with_proof(GenesisConfig::DEVNET6_BLOCK_HASH);
    let verification_result = bundle_node.verify(
        path_to_leaf.leaf.commitment().as_point(),
        &GenesisConfig::DEVNET6_STATE_ROOT,
    );
    if verification_result.is_err() {
        println!("{verification_result:?}");
    }
    assert!(matches!(verification_result, Ok(())));

    for fragment_index in 0..PORTAL_NETWORK_NODE_WIDTH as u8 {
        let commitment = leaf_node_builder.fragment_commitment(fragment_index);
        if commitment.is_zero() {
            continue;
        }

        println!("   fragment {:?}", commitment);
        let fragment_node = leaf_node_builder
            .fragment_node_with_proof(fragment_index, GenesisConfig::DEVNET6_BLOCK_HASH);
        let verification_result = fragment_node.verify(
            commitment,
            &GenesisConfig::DEVNET6_STATE_ROOT,
            path_to_leaf.leaf.stem(),
        );
        assert!(matches!(verification_result, Ok(())))
    }
}

#[test]
fn branches() {
    let state_writes = read_genesis().into_state_writes();

    let mut trie = VerkleTrie::new();
    trie.update(&state_writes);

    let mut checked_branches = HashSet::new();

    for state_write in state_writes.iter() {
        let stem = state_write.stem;
        println!("Starting stem: {stem}");

        let path_to_leaf = trie.traverse_to_leaf(&stem).unwrap();

        for (i, (branch_node, _)) in path_to_leaf.trie_path.iter().enumerate() {
            let commitment = B256::from(branch_node.commitment().as_point());
            if checked_branches.contains(&commitment) {
                println!("Skipping branch: {commitment:?}");
                continue;
            }
            check_branch(branch_node, &path_to_leaf.trie_path[..i]);
            checked_branches.insert(commitment);
        }
        println!("Finished stem\n");
    }
}

fn check_branch(branch_node: &BranchNode, trie_path: &[(&BranchNode, u8)]) {
    let branch_node_builder = PortalBranchNodeBuilder::new(branch_node, trie_path).unwrap();

    println!("Branch bundle {:?}", branch_node.commitment().as_point());
    let bundle_node = branch_node_builder.bundle_node_with_proof(GenesisConfig::DEVNET6_BLOCK_HASH);
    let verification_result = bundle_node.verify(
        branch_node.commitment().as_point(),
        &GenesisConfig::DEVNET6_STATE_ROOT,
    );
    assert!(matches!(verification_result, Ok(())));

    for fragment_index in 0..PORTAL_NETWORK_NODE_WIDTH as u8 {
        let commitment = branch_node_builder.fragment_commitment(fragment_index);
        if commitment.is_zero() {
            continue;
        }

        println!("     fragment {fragment_index} {:?}", commitment);
        let fragment_node = branch_node_builder
            .fragment_node_with_proof(fragment_index, GenesisConfig::DEVNET6_BLOCK_HASH);
        let verification_result =
            fragment_node.verify(commitment, &GenesisConfig::DEVNET6_STATE_ROOT);
        assert!(matches!(verification_result, Ok(())))
    }
}
