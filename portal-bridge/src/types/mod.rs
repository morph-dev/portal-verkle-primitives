use serde::{Deserialize, Serialize};

use self::beacon::SignedBeaconBlock;

pub mod beacon;
pub mod genesis;
pub mod state_write;
pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonResponseMessage {
    Success(SuccessMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuccessMessage {
    pub data: SignedBeaconBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    #[serde(alias = "statusCode")]
    pub code: u32,
    pub error: Option<String>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{read_dir, File},
        io::BufReader,
    };

    use crate::paths::{test_path, TESTNET_DATA_PATH};

    use super::*;

    #[test]
    fn parse_block_15() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("testdata/beacon.block.15.test.json")?);
        let response: JsonResponseMessage = serde_json::from_reader(reader)?;
        assert!(matches!(response, JsonResponseMessage::Success(_)));
        Ok(())
    }

    #[test]
    fn parse_block_453() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("testdata/beacon.block.453.test.json")?);
        let response: JsonResponseMessage = serde_json::from_reader(reader)?;
        assert!(matches!(response, JsonResponseMessage::Error(_)));
        Ok(())
    }

    #[test]
    fn parse_block_32100() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("testdata/beacon.block.32100.test.json")?);
        let response: JsonResponseMessage = serde_json::from_reader(reader)?;
        assert!(matches!(response, JsonResponseMessage::Success(_)));
        Ok(())
    }

    #[test]
    fn parse_all_beacon_slots() -> anyhow::Result<()> {
        let beacon_dir = test_path(TESTNET_DATA_PATH).join("beacon");
        for file in read_dir(beacon_dir)? {
            let reader = BufReader::new(File::open(file?.path())?);
            let response: JsonResponseMessage = serde_json::from_reader(reader)?;
            assert!(matches!(response, JsonResponseMessage::Success(_)));
        }
        Ok(())
    }
}
