use std::collections::{BTreeMap, HashMap};

use alloy_primitives::{b256, keccak256, Address, Bytes, B256, U256};
use serde::{Deserialize, Serialize};

use crate::{Stem, TrieKey, TrieValue};

use super::{storage::AccountStorageLayout, StateWrites, StemStateWrite};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountAlloc {
    pub balance: U256,
    pub nonce: Option<U256>,
    pub code: Option<Bytes>,
    pub storage: Option<HashMap<U256, TrieValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub alloc: HashMap<Address, AccountAlloc>,
}

impl GenesisConfig {
    pub const DEVNET6_BLOCK_HASH: B256 =
        b256!("3fe165c03e7a77d1e3759362ebeeb16fd964cb411ce11fbe35c7032fab5b9a8a");
    pub const DEVNET6_STATE_ROOT: B256 =
        b256!("1fbf85345a3cbba9a6d44f991b721e55620a22397c2a93ee8d5011136ac300ee");

    pub fn into_state_writes(self) -> StateWrites {
        let mut state_writes = BTreeMap::<Stem, StemStateWrite>::new();
        let mut insert_state_write = |key: TrieKey, value: TrieValue| {
            let stem = key.stem();
            state_writes
                .entry(stem)
                .or_insert_with(|| StemStateWrite {
                    stem,
                    writes: HashMap::new(),
                })
                .writes
                .insert(key.suffix(), value);
        };

        for (address, account_alloc) in self.alloc {
            let storage_layout = AccountStorageLayout::new(address);
            insert_state_write(storage_layout.version_key(), U256::ZERO.into());
            insert_state_write(storage_layout.balance_key(), account_alloc.balance.into());
            insert_state_write(
                storage_layout.nonce_key(),
                account_alloc.nonce.unwrap_or(U256::ZERO).into(),
            );

            match &account_alloc.code {
                None => insert_state_write(storage_layout.code_hash_key(), keccak256([]).into()),
                Some(code) => {
                    insert_state_write(storage_layout.code_hash_key(), keccak256(code).into());
                    insert_state_write(
                        storage_layout.code_size_key(),
                        U256::from(code.len()).into(),
                    );
                    for (key, value) in storage_layout.chunkify_code(code) {
                        insert_state_write(key, value);
                    }
                }
            }

            if let Some(storage) = account_alloc.storage {
                for (storage_key, value) in storage {
                    insert_state_write(storage_layout.storage_slot_key(storage_key), value);
                }
            }
        }

        StateWrites::new(state_writes.into_values().collect())
    }
}

impl From<GenesisConfig> for StateWrites {
    fn from(genesis_config: GenesisConfig) -> Self {
        genesis_config.into_state_writes()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use super::*;

    #[test]
    fn parse_genesis() -> anyhow::Result<()> {
        let reader = BufReader::new(File::open("../testdata/genesis.json")?);
        let genesis_config: GenesisConfig = serde_json::from_reader(reader)?;
        let alloc = genesis_config.alloc;
        assert_eq!(alloc.len(), 278);
        Ok(())
    }
}
