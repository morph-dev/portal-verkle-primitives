use std::{fs::File, io::BufWriter};

use clap::Parser;
use portal_bridge::paths::beacon_slot_path;
use reqwest::{Client, Method, Url};

const LOCALHOST_RPC_URL: &str = "http://localhost:9596/";
const BEACON_BLOCK_URL_PATH: &str = "eth/v2/beacon/blocks/";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub slots: u64,
    #[arg(long, default_value_t = String::from(LOCALHOST_RPC_URL))]
    pub rpc_url: String,
    #[arg(long, default_value_t = false)]
    pub override_existing: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = Client::new();

    for slot in 0..=args.slots {
        let path = beacon_slot_path(slot);
        if path.exists() && !args.override_existing {
            println!("Skipping - {slot:6}");
            continue;
        }
        println!("Fetching - {slot:6}");
        let url = Url::parse(&args.rpc_url)?
            .join(BEACON_BLOCK_URL_PATH)?
            .join(&slot.to_string())?;
        let response = client.request(Method::GET, url).send().await?;
        let response: serde_json::Value = response.json().await?;

        let response_as_map = response.as_object().expect("Response should be an object");
        if response_as_map.contains_key("error") {
            eprintln!("Error fetching slot: {slot}\n{response}");
            continue;
        }

        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer_pretty(writer, &response)?;
    }

    Ok(())
}
