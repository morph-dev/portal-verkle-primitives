use clap::Parser;
use portal_bridge::beacon_block_fetcher::BeaconBlockFetcher;

const LOCALHOST_RPC_URL: &str = "http://localhost:9596/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub slots: u64,
    #[arg(long, default_value_t = String::from(LOCALHOST_RPC_URL))]
    pub rpc_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let block_fetcher = BeaconBlockFetcher::new(&args.rpc_url, /* save_locally= */ true);

    for slot in 0..=args.slots {
        match block_fetcher.fetch_beacon_block(slot).await {
            Ok(Some(_)) => continue,
            Ok(None) => {
                eprintln!("Beacon slot {slot} doesn't exist")
            }
            Err(err) => {
                eprintln!("Error fetching beacon slot {slot}: {err}")
            }
        }
    }

    Ok(())
}
