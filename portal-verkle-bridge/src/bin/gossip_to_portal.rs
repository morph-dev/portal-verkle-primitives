use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use alloy_primitives::B256;
use clap::Parser;
use ethportal_api::{
    types::content_key::verkle::LeafFragmentKey, VerkleContentKey, VerkleContentValue,
    VerkleNetworkApiClient,
};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use portal_verkle_bridge::{
    beacon_block_fetcher::BeaconBlockFetcher,
    evm::VerkleEvm,
    types::{genesis::GenesisConfig, state_write::StateWrites},
    utils::{dummy_multiproof, read_genesis},
    verkle_trie::PathToLeaf,
};
use portal_verkle_trie::nodes::portal::ssz::{
    nodes::{
        BranchBundleNodeWithProof, BranchFragmentNode, BranchFragmentNodeWithProof,
        LeafBundleNodeWithProof, LeafFragmentNode, LeafFragmentNodeWithProof,
    },
    TriePath, TrieProof,
};
use verkle_core::{constants::PORTAL_NETWORK_NODE_WIDTH, Point, Stem};

const LOCALHOST_BEACON_RPC_URL: &str = "http://localhost:9596/";
const LOCALHOST_PORTAL_RPC_URL: &str = "http://localhost:8545/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub slots: u64,
    #[arg(long, default_value_t = String::from(LOCALHOST_BEACON_RPC_URL))]
    pub beacon_rpc_url: String,
    #[arg(long, default_value_t = String::from(LOCALHOST_PORTAL_RPC_URL))]
    pub portal_rpc_url: String,
}

struct Gossiper {
    block_fetcher: BeaconBlockFetcher,
    portal_client: HttpClient,
    evm: VerkleEvm,
}

impl Gossiper {
    fn new(args: &Args) -> anyhow::Result<Self> {
        let block_fetcher =
            BeaconBlockFetcher::new(&args.beacon_rpc_url, /* save_locally = */ false);
        let portal_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(60))
            .build(&args.portal_rpc_url)?;
        let evm = VerkleEvm::new(&read_genesis()?)?;

        Ok(Self {
            block_fetcher,
            portal_client,
            evm,
        })
    }

    async fn gossip_genesis(&mut self) -> anyhow::Result<()> {
        let genesis_config = read_genesis()?;
        let state_writes = genesis_config.generate_state_diff().into();
        println!("Gossiping genesis...");
        self.gossip_state_writes(
            GenesisConfig::DEVNET6_BLOCK_HASH,
            state_writes,
            HashSet::new(),
        )
        .await?;
        Ok(())
    }

    async fn gossip_slot(&mut self, slot: u64) -> anyhow::Result<()> {
        let Ok(Some(beacon_block)) = self.block_fetcher.fetch_beacon_block(slot).await else {
            println!("Beacon block for slot {slot} not found!");
            return Ok(());
        };
        let execution_payload = &beacon_block.message.body.execution_payload;
        let process_block_result = self.evm.process_block(execution_payload)?;
        println!(
            "Gossiping slot {slot:04} (block - number={:04} hash={} root={})",
            execution_payload.block_number,
            execution_payload.block_hash,
            execution_payload.state_root
        );
        self.gossip_state_writes(
            execution_payload.block_hash,
            process_block_result.state_writes,
            process_block_result.new_branch_nodes,
        )
        .await?;
        Ok(())
    }

    async fn gossip_state_writes(
        &self,
        block_hash: B256,
        state_writes: StateWrites,
        new_branch_nodes: HashSet<TriePath>,
    ) -> anyhow::Result<()> {
        let mut content_to_gossip = HashMap::<VerkleContentKey, VerkleContentValue>::new();
        for stem_state_write in state_writes.iter() {
            let stem = &stem_state_write.stem;
            let path_to_leaf = self.evm.state_trie().traverse_to_leaf(stem)?;
            for (depth, branch) in path_to_leaf.branches.iter().enumerate() {
                let trie_path = TriePath::from(stem[..depth].to_vec());

                // Branch bundle
                let content_key = VerkleContentKey::Bundle(branch.commitment().clone());
                if new_branch_nodes.contains(&trie_path)
                    && content_to_gossip.contains_key(&content_key)
                {
                    // We already gossiped this bundle and all fragments
                    continue;
                }

                content_to_gossip.entry(content_key).or_insert_with(|| {
                    let trie_proof = Self::create_bundle_trie_proof(
                        stem,
                        depth,
                        &path_to_leaf,
                        branch.commitment(),
                    );
                    VerkleContentValue::BranchBundleWithProof(BranchBundleNodeWithProof {
                        node: branch.extract_bundle_node(),
                        block_hash,
                        path: trie_path.clone(),
                        proof: trie_proof,
                    })
                });

                let fragment_indices = if new_branch_nodes.contains(&trie_path) {
                    0..PORTAL_NETWORK_NODE_WIDTH
                } else {
                    let fragment_index = stem[depth] as usize / PORTAL_NETWORK_NODE_WIDTH;
                    fragment_index..fragment_index + 1
                };
                // Branch fragment
                for fragment_index in fragment_indices {
                    let (fragment_commitment, fragment) =
                        branch.extract_fragment_node(fragment_index);
                    if fragment_commitment.is_zero() {
                        continue;
                    }
                    let content_key = VerkleContentKey::BranchFragment(fragment_commitment);
                    content_to_gossip.entry(content_key).or_insert_with(|| {
                        let trie_proof = Self::create_branch_fragment_trie_proof(
                            stem,
                            depth,
                            &path_to_leaf,
                            branch.commitment(),
                            &fragment,
                        );
                        VerkleContentValue::BranchFragmentWithProof(BranchFragmentNodeWithProof {
                            node: fragment,
                            block_hash,
                            path: trie_path.clone(),
                            proof: trie_proof,
                        })
                    });
                }
            }

            // Leaf bundle
            let bundle_commitment = path_to_leaf.leaf.commitment();
            let content_key = VerkleContentKey::Bundle(bundle_commitment.clone());
            content_to_gossip.entry(content_key).or_insert_with(|| {
                let trie_proof = Self::create_bundle_trie_proof(
                    stem,
                    path_to_leaf.branches.len(),
                    &path_to_leaf,
                    bundle_commitment,
                );
                VerkleContentValue::LeafBundleWithProof(LeafBundleNodeWithProof {
                    node: path_to_leaf.leaf.extract_bundle_node(),
                    block_hash,
                    proof: trie_proof,
                })
            });

            // Leaf Fragments
            let mut modified_fragments = stem_state_write
                .suffix_writes
                .iter()
                .map(|suffix_write| suffix_write.suffix as usize / PORTAL_NETWORK_NODE_WIDTH)
                .collect::<Vec<_>>();
            modified_fragments.sort();
            modified_fragments.dedup();
            for fragment_index in modified_fragments {
                let (fragment_commitment, fragment_node) =
                    path_to_leaf.leaf.extract_fragment_node(fragment_index);
                let content_key = VerkleContentKey::LeafFragment(LeafFragmentKey {
                    stem: *stem,
                    commitment: fragment_commitment,
                });

                content_to_gossip.entry(content_key).or_insert_with(|| {
                    let trie_proof = Self::create_leaf_fragment_trie_proof(
                        stem,
                        path_to_leaf.branches.len(),
                        &path_to_leaf,
                        bundle_commitment,
                        &fragment_node,
                    );
                    VerkleContentValue::LeafFragmentWithProof(LeafFragmentNodeWithProof {
                        node: fragment_node,
                        block_hash,
                        proof: trie_proof,
                    })
                });
            }
        }

        for (key, value) in content_to_gossip {
            self.portal_client.gossip(key, value).await?;
        }
        Ok(())
    }

    fn create_bundle_trie_proof(
        _stem: &Stem,
        depth: usize,
        path_to_leaf: &PathToLeaf,
        _node_commitment: &Point,
    ) -> TrieProof {
        TrieProof {
            commitments_by_path: path_to_leaf.branches[..depth]
                .iter()
                .map(|branch| branch.commitment().clone())
                .collect::<Vec<_>>()
                .into(),
            multi_point_proof: dummy_multiproof(),
        }
    }

    fn create_branch_fragment_trie_proof(
        _stem: &Stem,
        depth: usize,
        path_to_leaf: &PathToLeaf,
        _bundle_node_commitment: &Point,
        _node: &BranchFragmentNode,
    ) -> TrieProof {
        TrieProof {
            commitments_by_path: path_to_leaf.branches[..depth]
                .iter()
                .map(|branch| branch.commitment().clone())
                .collect::<Vec<_>>()
                .into(),
            multi_point_proof: dummy_multiproof(),
        }
    }

    fn create_leaf_fragment_trie_proof(
        _stem: &Stem,
        depth: usize,
        path_to_leaf: &PathToLeaf,
        _bundle_node_commitment: &Point,
        _node: &LeafFragmentNode,
    ) -> TrieProof {
        TrieProof {
            commitments_by_path: path_to_leaf.branches[..depth]
                .iter()
                .map(|branch| branch.commitment().clone())
                .collect::<Vec<_>>()
                .into(),
            multi_point_proof: dummy_multiproof(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Initializing...");
    let mut gossiper = Gossiper::new(&args)?;

    println!("Starting gossiping");
    gossiper.gossip_genesis().await?;
    for slot in 1..=args.slots {
        gossiper.gossip_slot(slot).await?;
    }

    Ok(())
}
