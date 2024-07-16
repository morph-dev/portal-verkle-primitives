use std::path::PathBuf;

pub const TESTNET_DATA_PATH: &str = "data/verkle-devnet-6/";

pub fn beacon_slot_path(slot: u64) -> PathBuf {
    PathBuf::from(TESTNET_DATA_PATH).join(format!("beacon/slot.{slot}.json"))
}

pub fn genesis_path() -> PathBuf {
    PathBuf::from(TESTNET_DATA_PATH).join("genesis.json")
}

#[cfg(test)]
pub fn test_path<P: AsRef<std::path::Path>>(path: P) -> PathBuf {
    PathBuf::from("..").join(path)
}
