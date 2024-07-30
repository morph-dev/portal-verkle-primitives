use std::array;

use crate::constants::{PORTAL_NETWORK_NODE_WIDTH, VERKLE_NODE_WIDTH};

/// Creates the array that has [VERKLE_NODE_WIDTH] length.
pub fn array_long<T>(f: impl Fn(u8) -> T) -> [T; VERKLE_NODE_WIDTH] {
    array::from_fn(|index| f(index as u8))
}

pub fn array_long_const<T: Clone>(value: T) -> [T; VERKLE_NODE_WIDTH] {
    array_long(|_| value.clone())
}

/// Creates the array that has [PORTAL_NETWORK_NODE_WIDTH] length.
pub fn array_short<T>(f: impl Fn(u8) -> T) -> [T; PORTAL_NETWORK_NODE_WIDTH] {
    array::from_fn(|index| f(index as u8))
}

pub(crate) mod branch_utils {
    use crate::constants::PORTAL_NETWORK_NODE_WIDTH;

    use super::array_short;

    pub fn child_index(fragment_index: u8, fragment_child_index: u8) -> u8 {
        fragment_child_index + fragment_index * PORTAL_NETWORK_NODE_WIDTH as u8
    }

    pub fn fragment_index(child_index: u8) -> u8 {
        child_index / PORTAL_NETWORK_NODE_WIDTH as u8
    }

    pub fn fragment_child_index(child_index: u8) -> u8 {
        child_index % PORTAL_NETWORK_NODE_WIDTH as u8
    }

    pub fn openings(fragment_index: u8) -> impl IntoIterator<Item = u8> {
        array_short(|fragment_child_index| child_index(fragment_index, fragment_child_index))
    }

    pub fn openings_for_bundle(fragment_index: u8) -> impl IntoIterator<Item = u8> {
        array_short(|other_fragment_index| other_fragment_index)
            .into_iter()
            .filter(move |other_fragment_index| other_fragment_index != &fragment_index)
            .flat_map(openings)
    }
}

pub(crate) mod leaf_utils {
    use crate::constants::{LEAF_C1_INDEX, LEAF_C2_INDEX, PORTAL_NETWORK_NODE_WIDTH};

    use super::array_short;

    pub fn leaf_suffix_index(fragment_index: u8) -> u8 {
        if fragment_index < PORTAL_NETWORK_NODE_WIDTH as u8 / 2 {
            LEAF_C1_INDEX
        } else {
            LEAF_C2_INDEX
        }
    }

    pub fn suffix_fragment_index(fragment_index: u8) -> u8 {
        fragment_index % (PORTAL_NETWORK_NODE_WIDTH / 2) as u8
    }

    pub fn suffix_indices(fragment_index: u8, fragment_child_index: u8) -> [u8; 2] {
        let suffix_fragment_index = suffix_fragment_index(fragment_index);
        let suffix_child_index =
            fragment_child_index + suffix_fragment_index * PORTAL_NETWORK_NODE_WIDTH as u8;
        let low_index = 2 * suffix_child_index;
        let high_index = low_index + 1;
        [low_index, high_index]
    }

    pub fn suffix_openings(fragment_index: u8) -> impl IntoIterator<Item = u8> {
        array_short(|fragment_child_index| suffix_indices(fragment_index, fragment_child_index))
            .into_iter()
            .flatten()
    }

    pub fn suffix_openings_for_bundle(fragment_index: u8) -> impl IntoIterator<Item = u8> {
        array_short(|other_fragment_index| other_fragment_index)
            .into_iter()
            .filter(move |other_fragment_index| {
                other_fragment_index != &fragment_index
                    && leaf_suffix_index(*other_fragment_index) == leaf_suffix_index(fragment_index)
            })
            .flat_map(suffix_openings)
    }
}
