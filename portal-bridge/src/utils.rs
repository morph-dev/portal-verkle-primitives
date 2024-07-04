use std::{fs::File, io::BufReader, path::Path};

use crate::{paths::genesis_path, types::genesis::GenesisConfig};

pub fn read_genesis() -> anyhow::Result<GenesisConfig> {
    read_genesis_from_file(genesis_path())
}

#[cfg(test)]
pub fn read_genesis_for_test() -> anyhow::Result<GenesisConfig> {
    use crate::paths::test_path;

    read_genesis_from_file(test_path(genesis_path()))
}

fn read_genesis_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<GenesisConfig> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}
