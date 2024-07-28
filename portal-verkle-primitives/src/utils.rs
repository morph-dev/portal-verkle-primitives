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
