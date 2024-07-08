use std::collections::HashSet;

use alloy_primitives::{address, keccak256, U8};
use portal_verkle_trie::nodes::portal::ssz::TriePath;
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

pub struct VerkleEvm {
    block: u64,
    state_trie: VerkleTrie,
}

pub struct ProcessBlockResult {
    pub state_writes: StateWrites,
    pub new_branch_nodes: HashSet<TriePath>,
}

impl VerkleEvm {
    pub fn new(genesis_config: &GenesisConfig) -> Result<Self, EvmError> {
        let mut state_trie = VerkleTrie::new();
        state_trie
            .update(&genesis_config.generate_state_diff().into())
            .map_err(EvmError::TrieError)?;
        Ok(Self {
            block: 0,
            state_trie,
        })
    }

    pub fn state_trie(&self) -> &VerkleTrie {
        &self.state_trie
    }

    pub fn block(&self) -> u64 {
        self.block
    }

    pub fn process_block(
        &mut self,
        execution_payload: &ExecutionPayload,
    ) -> Result<ProcessBlockResult, EvmError> {
        if self.block + 1 != execution_payload.block_number.to::<u64>() {
            return Err(EvmError::UnexpectedBlock {
                expected: self.block + 1,
                actual: execution_payload.block_number.to(),
            });
        }

        let mut state_diff = execution_payload.execution_witness.state_diff.clone();

        if self.block == 0 {
            update_state_diff_for_eip2935(&mut state_diff);
        }

        let state_writes = StateWrites::from(state_diff);

        let new_branch_nodes = self
            .state_trie
            .update(&state_writes)
            .map_err(EvmError::TrieError)?;
        self.block += 1;

        if self.state_trie.root() != execution_payload.state_root {
            return Err(EvmError::WrongStateRoot {
                expected: execution_payload.state_root,
                actual: self.state_trie.root(),
            });
        }
        Ok(ProcessBlockResult {
            state_writes,
            new_branch_nodes,
        })
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

        let evm = VerkleEvm::new(&read_genesis_for_test()?)?;

        assert_eq!(evm.state_trie.root(), STATE_ROOT);
        Ok(())
    }

    #[test]
    fn process_block_1() -> Result<()> {
        let mut evm = VerkleEvm::new(&read_genesis_for_test()?)?;

        let reader = BufReader::new(File::open(test_path(beacon_slot_path(1)))?);
        let response: SuccessMessage = serde_json::from_reader(reader)?;
        let execution_payload = response.data.message.body.execution_payload;
        evm.process_block(&execution_payload)?;
        assert_eq!(evm.state_trie.root(), execution_payload.state_root);
        Ok(())
    }

    #[test]
    fn process_block_1000() -> Result<()> {
        let mut evm = VerkleEvm::new(&read_genesis_for_test()?)?;

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
