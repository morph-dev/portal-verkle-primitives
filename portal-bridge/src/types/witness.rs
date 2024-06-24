use alloy_primitives::{Bytes, U8};
use banderwagon::{Element, Fr};
use serde::{Deserialize, Serialize};
use serde_nested_with::serde_nested;
use verkle_core::{Stem, TrieValue};

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
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    pub commitments_by_path: Vec<Element>,
    #[serde(with = "verkle_core::serde::element")]
    pub d: Element,
    #[serde(alias = "ipaProof")]
    pub ipa_proof: IpaProof,
}

#[serde_nested]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpaProof {
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    pub cl: Vec<Element>,
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    pub cr: Vec<Element>,
    #[serde(alias = "finalEvaluation")]
    #[serde(with = "verkle_core::serde::fr")]
    pub final_evaluation: Fr,
}
