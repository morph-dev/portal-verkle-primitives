use alloy_primitives::{Bytes, B256, U64};
use serde::{Deserialize, Serialize};

use super::witness::ExecutionWitness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBeaconBlock {
    message: BeaconBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlock {
    parent_root: B256,
    state_root: B256,
    body: BeaconBlockBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockBody {
    execution_payload: ExecutionPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    block_number: U64,
    block_hash: B256,
    parent_hash: B256,
    state_root: B256,
    timestamp: U64,
    transactions: Vec<Bytes>,
    execution_witness: ExecutionWitness,
}
