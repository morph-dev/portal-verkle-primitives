use clap::Parser;
use portal_bridge::{
    beacon_block_fetcher::BeaconBlockFetcher, evm::VerkleEvm, types::state_write::StateWrites,
    utils::read_genesis,
};

const LOCALHOST_RPC_URL: &str = "http://localhost:9596/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub slots: u64,
    #[arg(long, default_value_t = String::from(LOCALHOST_RPC_URL))]
    pub rpc_url: String,
}

async fn gossip_state_writes(_state_writes: StateWrites) -> anyhow::Result<()> {
    // todo!("gossip state")
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let block_fetcher = BeaconBlockFetcher::new(&args.rpc_url, /* save_locally = */ false);
    let mut evm = VerkleEvm::new();

    let state_writes = evm.initialize_genesis(&read_genesis()?)?;
    println!("Gossiping genesis...");
    gossip_state_writes(state_writes).await?;

    for slot in 1..=args.slots {
        let Some(beacon_block) = block_fetcher.fetch_beacon_block(slot).await? else {
            println!("Beacon block for slot {slot} not found!");
            continue;
        };
        let execution_payload = &beacon_block.message.body.execution_payload;
        let state_writes = evm.process_block(execution_payload)?;
        println!(
            "Gossiping slot {slot:04} (block - number={:04} hash={})",
            execution_payload.block_number, execution_payload.block_hash
        );
        gossip_state_writes(state_writes).await?;
    }

    Ok(())
}
