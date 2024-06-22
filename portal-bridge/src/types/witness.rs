use alloy_primitives::{Bytes, U8};
use banderwagon::{Element, Fr};
use serde::{Deserialize, Serialize};
use serde_nested_with::serde_nested;
use verkle_core::{Stem, TrieValue};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionWitness {
    #[serde(alias = "stateDiff")]
    state_diff: Vec<StemStateDiff>,
    #[serde(alias = "verkleProof")]
    verkle_proof: VerkleProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StemStateDiff {
    stem: Stem,
    #[serde(alias = "suffixDiffs")]
    suffix_diffs: Vec<SuffixStateDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuffixStateDiff {
    suffix: U8,
    #[serde(alias = "currentValue")]
    current_value: Option<TrieValue>,
    #[serde(alias = "newValue")]
    new_value: Option<TrieValue>,
}

#[serde_nested]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerkleProof {
    #[serde(alias = "otherStems")]
    other_stems: Vec<Stem>,
    #[serde(alias = "depthExtensionPresent")]
    depth_extension_present: Bytes,
    #[serde(alias = "commitmentsByPath")]
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    commitments_by_path: Vec<Element>,
    #[serde(with = "verkle_core::serde::element")]
    d: Element,
    #[serde(alias = "ipaProof")]
    ipa_proof: IpaProof,
}

#[serde_nested]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpaProof {
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    cl: Vec<Element>,
    #[serde_nested(sub = "Element", serde(with = "verkle_core::serde::element"))]
    cr: Vec<Element>,
    #[serde(alias = "finalEvaluation")]
    #[serde(with = "verkle_core::serde::fr")]
    final_evaluation: Fr,
}
