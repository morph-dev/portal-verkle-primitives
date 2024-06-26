use std::collections::HashMap;

use alloy_primitives::{address, keccak256, U256, U8};
use verkle_core::{storage::AccountStorageLayout, Stem, TrieKey, TrieValue};

use super::error::EvmError;
use crate::{
    types::{
        beacon::ExecutionPayload,
        genesis::GenesisConfig,
        witness::{StemStateDiff, SuffixStateDiff},
    },
    verkle_trie::VerkleTrie,
};

#[derive(Default)]
pub struct VerkleEvm {
    next_block: u64,
    state: VerkleTrie,
}

impl VerkleEvm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_block(&self) -> u64 {
        self.next_block
    }

    pub fn initialize_genesis(&mut self, genesis_config: &GenesisConfig) -> Result<(), EvmError> {
        if self.next_block != 0 {
            return Err(EvmError::UnexpectedBlock {
                expected: self.next_block,
                actual: 0,
            });
        }
        let mut state_diffs =
            HashMap::<Stem, StemStateDiff>::with_capacity(genesis_config.alloc.len());
        let mut insert_state_diff = |key: TrieKey, value: TrieValue| {
            let stem = key.stem();
            let state_diff = state_diffs.entry(stem).or_insert_with(|| StemStateDiff {
                stem,
                suffix_diffs: Vec::new(),
            });
            state_diff.suffix_diffs.push(SuffixStateDiff {
                suffix: U8::from(key.suffix()),
                current_value: None,
                new_value: Some(value),
            })
        };

        for (address, account_alloc) in &genesis_config.alloc {
            let storage_layout = AccountStorageLayout::new(*address);
            insert_state_diff(storage_layout.version_key(), U256::ZERO.into());
            insert_state_diff(storage_layout.balance_key(), account_alloc.balance.into());
            insert_state_diff(
                storage_layout.nonce_key(),
                account_alloc.nonce.unwrap_or(U256::ZERO).into(),
            );

            match &account_alloc.code {
                None => insert_state_diff(storage_layout.code_hash_key(), keccak256([]).into()),
                Some(code) => {
                    insert_state_diff(storage_layout.code_hash_key(), keccak256(code).into());
                    insert_state_diff(
                        storage_layout.code_size_key(),
                        U256::from(code.len()).into(),
                    );
                    for (key, value) in storage_layout.chunkify_code(code) {
                        insert_state_diff(key, value);
                    }
                }
            }

            if let Some(storage) = &account_alloc.storage {
                for (storage_key, value) in storage {
                    insert_state_diff(storage_layout.storage_slot_key(*storage_key), *value);
                }
            }
        }

        let state_diffs = state_diffs.into_values().collect::<Vec<_>>();
        self.state
            .update(&state_diffs)
            .map_err(EvmError::TrieError)?;
        self.next_block += 1;

        Ok(())
    }

    pub fn process_block(&mut self, execution_payload: &ExecutionPayload) -> Result<(), EvmError> {
        if self.next_block != execution_payload.block_number.to::<u64>() {
            return Err(EvmError::UnexpectedBlock {
                expected: self.next_block,
                actual: execution_payload.block_number.to(),
            });
        }

        if self.next_block == 1 {
            // Eip-2935: Initialize account: "0xfffffffffffffffffffffffffffffffffffffffe"
            // NOTE: This is not included into execution_witness (probably a bug).
            let storage_layout =
                AccountStorageLayout::new(address!("fffffffffffffffffffffffffffffffffffffffe"));
            self.state
                .insert(&storage_layout.version_key(), TrieValue::ZERO);
            self.state
                .insert(&storage_layout.balance_key(), TrieValue::ZERO);
            self.state
                .insert(&storage_layout.nonce_key(), TrieValue::ZERO);
            self.state
                .insert(&storage_layout.code_hash_key(), keccak256([]).into());
        }

        self.state
            .update(&execution_payload.execution_witness.state_diff)
            .map_err(EvmError::TrieError)?;
        self.next_block += 1;

        if self.state.root() != execution_payload.state_root {
            return Err(EvmError::WrongStateRoot {
                expected: execution_payload.state_root,
                actual: self.state.root(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use alloy_primitives::{b256, B256};
    use anyhow::Result;

    use crate::{
        paths::{beacon_slot_path, genesis_path, test_path},
        types::JsonResponseMessage,
    };

    use super::*;

    #[test]
    fn genesis() -> Result<()> {
        const STATE_ROOT: B256 =
            b256!("1fbf85345a3cbba9a6d44f991b721e55620a22397c2a93ee8d5011136ac300ee");

        let mut evm = VerkleEvm::new();

        let reader = BufReader::new(File::open(test_path(genesis_path()))?);
        let genesis_config = serde_json::from_reader(reader)?;
        evm.initialize_genesis(&genesis_config)?;

        assert_eq!(evm.state.root(), STATE_ROOT);
        Ok(())
    }

    #[test]
    fn process_block_1() -> Result<()> {
        let mut evm = VerkleEvm::new();

        let reader = BufReader::new(File::open(test_path(genesis_path()))?);
        let genesis_config = serde_json::from_reader(reader)?;
        evm.initialize_genesis(&genesis_config)?;

        let reader = BufReader::new(File::open(test_path(beacon_slot_path(1)))?);
        let response: JsonResponseMessage = serde_json::from_reader(reader)?;
        let execution_payload = response.data.message.body.execution_payload;
        evm.process_block(&execution_payload)?;
        assert_eq!(evm.state.root(), execution_payload.state_root);
        Ok(())
    }

    #[test]
    fn process_block_1000() -> Result<()> {
        let mut evm = VerkleEvm::new();

        let reader = BufReader::new(File::open(test_path(genesis_path()))?);
        let genesis_config = serde_json::from_reader(reader)?;
        evm.initialize_genesis(&genesis_config)?;

        for block in 1..=1000 {
            let path = test_path(beacon_slot_path(block));
            if !path.exists() {
                continue;
            }
            let reader = BufReader::new(File::open(path)?);
            let response: JsonResponseMessage = serde_json::from_reader(reader)?;
            let execution_payload = response.data.message.body.execution_payload;
            evm.process_block(&execution_payload)?;
        }
        Ok(())
    }
}
