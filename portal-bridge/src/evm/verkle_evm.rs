use alloy_primitives::{address, keccak256, U8};
use verkle_core::{
    constants::{BALANCE_LEAF_KEY, CODE_KECCAK_LEAF_KEY, NONCE_LEAF_KEY, VERSION_LEAF_KEY},
    storage::AccountStorageLayout,
    TrieValue,
};

use super::error::EvmError;
use crate::{
    types::{
        beacon::ExecutionPayload,
        genesis::GenesisConfig,
        state_write::StateWrites,
        witness::{StateDiff, SuffixStateDiff},
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

    pub fn initialize_genesis(
        &mut self,
        genesis_config: &GenesisConfig,
    ) -> Result<StateWrites, EvmError> {
        if self.next_block != 0 {
            return Err(EvmError::UnexpectedBlock {
                expected: self.next_block,
                actual: 0,
            });
        }
        let state_writes = genesis_config.generate_state_diff().into();
        self.state
            .update(&state_writes)
            .map_err(EvmError::TrieError)?;
        self.next_block += 1;

        Ok(state_writes)
    }

    pub fn process_block(
        &mut self,
        execution_payload: &ExecutionPayload,
    ) -> Result<StateWrites, EvmError> {
        if self.next_block != execution_payload.block_number.to::<u64>() {
            return Err(EvmError::UnexpectedBlock {
                expected: self.next_block,
                actual: execution_payload.block_number.to(),
            });
        }

        let mut state_diff = execution_payload.execution_witness.state_diff.clone();

        if self.next_block == 1 {
            update_state_diff_for_eip2935(&mut state_diff);
        }

        let state_writes = StateWrites::from(state_diff);

        self.state
            .update(&state_writes)
            .map_err(EvmError::TrieError)?;
        self.next_block += 1;

        if self.state.root() != execution_payload.state_root {
            return Err(EvmError::WrongStateRoot {
                expected: execution_payload.state_root,
                actual: self.state.root(),
            });
        }
        Ok(state_writes)
    }
}

/// Eip-2935: Initialize account: "0xfffffffffffffffffffffffffffffffffffffffe"
/// NOTE: This is not included into execution_witness (probably a bug).
fn update_state_diff_for_eip2935(state_diff: &mut StateDiff) {
    let storage_layout =
        AccountStorageLayout::new(address!("fffffffffffffffffffffffffffffffffffffffe"));
    let suffix_diffs = [
        (VERSION_LEAF_KEY, TrieValue::ZERO),
        (BALANCE_LEAF_KEY, TrieValue::ZERO),
        (NONCE_LEAF_KEY, TrieValue::ZERO),
        (CODE_KECCAK_LEAF_KEY, TrieValue::from(keccak256([]))),
    ];

    let stem_state_diff = state_diff
        .iter_mut()
        .find(|stem_state_diff| &stem_state_diff.stem == storage_layout.account_storage_stem())
        .expect("to find StemStateDiff for EIP-2935");

    for (suffix, trie_value) in suffix_diffs {
        stem_state_diff.suffix_diffs.push(SuffixStateDiff {
            suffix: U8::from(suffix),
            current_value: None,
            new_value: Some(trie_value),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use alloy_primitives::{b256, B256};
    use anyhow::Result;

    use crate::{
        paths::{beacon_slot_path, test_path},
        types::SuccessMessage,
        utils::read_genesis_for_test,
    };

    use super::*;

    #[test]
    fn genesis() -> Result<()> {
        const STATE_ROOT: B256 =
            b256!("1fbf85345a3cbba9a6d44f991b721e55620a22397c2a93ee8d5011136ac300ee");

        let mut evm = VerkleEvm::new();
        evm.initialize_genesis(&read_genesis_for_test()?)?;

        assert_eq!(evm.state.root(), STATE_ROOT);
        Ok(())
    }

    #[test]
    fn process_block_1() -> Result<()> {
        let mut evm = VerkleEvm::new();
        evm.initialize_genesis(&read_genesis_for_test()?)?;

        let reader = BufReader::new(File::open(test_path(beacon_slot_path(1)))?);
        let response: SuccessMessage = serde_json::from_reader(reader)?;
        let execution_payload = response.data.message.body.execution_payload;
        evm.process_block(&execution_payload)?;
        assert_eq!(evm.state.root(), execution_payload.state_root);
        Ok(())
    }

    #[test]
    fn process_block_1000() -> Result<()> {
        let mut evm = VerkleEvm::new();
        evm.initialize_genesis(&read_genesis_for_test()?)?;

        for block in 1..=1000 {
            let path = test_path(beacon_slot_path(block));
            if !path.exists() {
                continue;
            }
            let reader = BufReader::new(File::open(path)?);
            let response: SuccessMessage = serde_json::from_reader(reader)?;
            let execution_payload = response.data.message.body.execution_payload;
            evm.process_block(&execution_payload)?;
        }
        Ok(())
    }
}
