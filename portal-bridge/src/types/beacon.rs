use alloy_primitives::{Bytes, B256, U64};
use serde::{Deserialize, Serialize};

use crate::types::witness::ExecutionWitness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlock {
    pub parent_root: B256,
    pub state_root: B256,
    pub body: BeaconBlockBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockBody {
    pub execution_payload: ExecutionPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub block_number: U64,
    pub block_hash: B256,
    pub parent_hash: B256,
    pub state_root: B256,
    pub timestamp: U64,
    pub transactions: Vec<Bytes>,
    pub execution_witness: ExecutionWitness,
}
