use std::collections::HashMap;

use alloy_primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use verkle_core::TrieValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub alloc: HashMap<Address, AccountAlloc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountAlloc {
    pub balance: U256,
    pub nonce: Option<U256>,
    pub code: Option<Bytes>,
    pub storage: Option<HashMap<U256, TrieValue>>,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::paths::{genesis_path, test_path};

    use super::*;

    #[test]
    fn parse_genesis() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open(test_path(genesis_path()))?);
        let genesis_config: GenesisConfig = serde_json::from_reader(reader)?;
        let alloc = genesis_config.alloc;
        assert_eq!(alloc.len(), 278);
        Ok(())
    }
}
