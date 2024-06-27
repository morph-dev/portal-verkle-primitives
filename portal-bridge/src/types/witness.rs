use alloy_primitives::{Bytes, U8};
use serde::{Deserialize, Serialize};
use serde_nested_with::serde_nested;
use verkle_core::{constants::VERKLE_NODE_WIDTH_BITS, Point, ScalarField, Stem, TrieValue};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionWitness {
    #[serde(alias = "stateDiff")]
    pub state_diff: Vec<StemStateDiff>,
    #[serde(alias = "verkleProof")]
    pub verkle_proof: VerkleProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StemStateDiff {
    pub stem: Stem,
    #[serde(alias = "suffixDiffs")]
    pub suffix_diffs: Vec<SuffixStateDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuffixStateDiff {
    pub suffix: U8,
    #[serde(alias = "currentValue")]
    pub current_value: Option<TrieValue>,
    #[serde(alias = "newValue")]
    pub new_value: Option<TrieValue>,
}

#[serde_nested]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerkleProof {
    #[serde(alias = "otherStems")]
    pub other_stems: Vec<Stem>,
    #[serde(alias = "depthExtensionPresent")]
    pub depth_extension_present: Bytes,
    #[serde(alias = "commitmentsByPath")]
    pub commitments_by_path: Vec<Point>,
    pub d: Point,
    #[serde(alias = "ipaProof")]
    pub ipa_proof: IpaProof,
}

#[serde_nested]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpaProof {
    pub cl: [Point; VERKLE_NODE_WIDTH_BITS],
    pub cr: [Point; VERKLE_NODE_WIDTH_BITS],
    #[serde(alias = "finalEvaluation")]
    pub final_evaluation: ScalarField,
}
