use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use anyhow::bail;
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::{
    paths::beacon_slot_path,
    types::{beacon::SignedBeaconBlock, JsonResponseMessage},
};

const BEACON_BLOCK_URL_PATH: &str = "eth/v2/beacon/blocks/";

pub struct BeaconBlockFetcher {
    rpc_url: String,
    save_locally: bool,
    client: Client,
}

impl BeaconBlockFetcher {
    pub fn new(rpc_url: &str, save_locally: bool) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            save_locally,
            client: Client::new(),
        }
    }

    pub async fn fetch_beacon_block(&self, slot: u64) -> anyhow::Result<Option<SignedBeaconBlock>> {
        let path = beacon_slot_path(slot);
        if path.exists() {
            let reader = BufReader::new(File::open(path)?);
            let message: JsonResponseMessage = serde_json::from_reader(reader)?;
            match message {
                JsonResponseMessage::Success(success_message) => Ok(Some(success_message.data)),
                JsonResponseMessage::Error(error_message) => {
                    bail!("Error reading beacon slot file {slot}: {:?}", error_message)
                }
            }
        } else {
            let url = Url::parse(&self.rpc_url)?
                .join(BEACON_BLOCK_URL_PATH)?
                .join(&slot.to_string())?;
            let response = self.client.get(url).send().await?;
            let response: serde_json::Value = response.json().await?;
            let message = JsonResponseMessage::deserialize(&response)?;
            match message {
                JsonResponseMessage::Success(success_message) => {
                    if self.save_locally {
                        let writer = BufWriter::new(File::create(path)?);
                        serde_json::to_writer_pretty(writer, &response)?;
                    }
                    Ok(Some(success_message.data))
                }
                JsonResponseMessage::Error(error_message) => {
                    if error_message.code == 404 {
                        Ok(None)
                    } else {
                        bail!("Error fetching beacon slot {slot}: {:?}", error_message)
                    }
                }
            }
        }
    }
}
