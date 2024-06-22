use std::path::PathBuf;

pub mod types;

const DATA_PATH: &str = "data/verkle-devnet-6/";

pub fn beacon_slot_path(slot: u64) -> PathBuf {
    let mut path = PathBuf::new();

    path.push(DATA_PATH);
    path.push(format!("beacon/slot.{slot}.json"));
    path
}
