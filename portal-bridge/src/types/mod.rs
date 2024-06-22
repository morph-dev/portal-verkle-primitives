use serde::{Deserialize, Serialize};

use self::beacon::SignedBeaconBlock;

pub mod beacon;
pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonResponseMessage {
    data: SignedBeaconBlock,
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{read_dir, File},
        io::BufReader,
    };

    use super::*;

    #[test]
    fn parse_block_15() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("testdata/beacon.block.15.test.json")?);
        let _: JsonResponseMessage = serde_json::from_reader(reader)?;
        Ok(())
    }

    #[test]
    fn parse_block_32100() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("testdata/beacon.block.32100.test.json")?);
        let _: JsonResponseMessage = serde_json::from_reader(reader)?;
        Ok(())
    }

    #[test]
    fn parse_all_beacon_slots() -> anyhow::Result<()> {
        for file in read_dir("../data/verkle-devnet-6/beacon")? {
            let reader = BufReader::new(File::open(dbg!(file?.path()))?);
            let _: JsonResponseMessage = serde_json::from_reader(reader)?;
        }
        Ok(())
    }
}
