use anyhow::bail;
use clap::Parser;
use portal_bridge::{
    beacon_block_fetcher::BeaconBlockFetcher, state_trie_fetcher::StateTrieFetcher,
    types::genesis::GenesisConfig,
};

const LOCALHOST_BEACON_RPC_URL: &str = "http://localhost:9596/";
const LOCALHOST_PORTAL_RPC_URL: &str = "http://localhost:8545/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, num_args=1..)]
    pub slots: Vec<u64>,
    #[arg(long, default_value_t = String::from(LOCALHOST_BEACON_RPC_URL))]
    pub beacon_rpc_url: String,
    #[arg(long, default_value_t = String::from(LOCALHOST_PORTAL_RPC_URL))]
    pub portal_rpc_url: String,
}

struct StateVerifier {
    block_fetcher: BeaconBlockFetcher,
    state_trie_fetcher: StateTrieFetcher,
}

impl StateVerifier {
    fn new(args: &Args) -> anyhow::Result<Self> {
        println!("Initializing...");
        let block_fetcher =
            BeaconBlockFetcher::new(&args.beacon_rpc_url, /* save_locally = */ false);
        let state_trie_fetcher = StateTrieFetcher::new(&args.portal_rpc_url)?;
        Ok(Self {
            block_fetcher,
            state_trie_fetcher,
        })
    }

    async fn verify_state(&self, slot: u64) -> anyhow::Result<()> {
        let root = if slot == 0 {
            GenesisConfig::DEVNET6_STATE_ROOT
        } else {
            let Some(beacon_block) = self.block_fetcher.fetch_beacon_block(slot).await? else {
                bail!("Beacon block for slot {slot} not found!")
            };
            beacon_block.message.body.execution_payload.state_root
        };
        println!("Veryfing slot {slot} with state root: {root}");
        match self.state_trie_fetcher.fetch_state_trie(root).await {
            Ok(synced_state_trie) => {
                if synced_state_trie.root() == root {
                    println!("SUCCESS");
                } else {
                    println!(
                        "ERROR: State trie fetched but root is different! Expected {root} but received {}",
                        synced_state_trie.root()
                    )
                }
            }
            Err(err) => {
                println!("ERROR: Error while fetching state trie: {err}")
            }
        };
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let verifier = StateVerifier::new(&args)?;
    for slot in args.slots {
        verifier.verify_state(slot).await?;
    }
    Ok(())
}
