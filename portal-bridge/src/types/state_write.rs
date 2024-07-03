use derive_more::{Deref, Index, IndexMut};
use verkle_core::{Stem, TrieValue};

use super::witness::{StateDiff, StemStateDiff, SuffixStateDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuffixStateWrites {
    pub suffix: u8,
    pub old_value: Option<TrieValue>,
    pub new_value: TrieValue,
}

impl SuffixStateWrites {
    pub fn from(suffix_state_diff: SuffixStateDiff) -> Option<Self> {
        let SuffixStateDiff {
            suffix,
            current_value: old_value,
            new_value,
        } = suffix_state_diff;

        if old_value == new_value {
            return None;
        }

        new_value.map(|new_value| Self {
            suffix: suffix.to(),
            old_value,
            new_value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemStateWrite {
    pub stem: Stem,
    pub suffix_writes: Vec<SuffixStateWrites>,
}

impl StemStateWrite {
    pub fn from(stem_state_diff: StemStateDiff) -> Option<Self> {
        let StemStateDiff { stem, suffix_diffs } = stem_state_diff;

        let suffix_writes = suffix_diffs
            .into_iter()
            .filter_map(SuffixStateWrites::from)
            .collect::<Vec<_>>();

        if suffix_writes.is_empty() {
            None
        } else {
            Some(Self {
                stem,
                suffix_writes,
            })
        }
    }
}

#[derive(Debug, Clone, Deref, Index, IndexMut)]
pub struct StateWrite(Vec<StemStateWrite>);

impl From<StateDiff> for StateWrite {
    fn from(state_diff: StateDiff) -> Self {
        Self(
            state_diff
                .into_iter()
                .filter_map(StemStateWrite::from)
                .collect(),
        )
    }
}
