use alloy_primitives::U256;

pub const PORTAL_NETWORK_NODE_WIDTH_BITS: usize = 4;
pub const PORTAL_NETWORK_NODE_WIDTH: usize = 1 << PORTAL_NETWORK_NODE_WIDTH_BITS; // 2^4 = 16

pub const VERKLE_NODE_WIDTH_BITS: usize = 8;
pub const VERKLE_NODE_WIDTH: usize = 1 << VERKLE_NODE_WIDTH_BITS; // 2^8 = 256
pub const VERKLE_NODE_WIDTH_U256: U256 = U256::from_limbs([VERKLE_NODE_WIDTH as u64, 0, 0, 0]);

// Storage layout parameters
pub const VERSION_LEAF_KEY: u8 = 0;
pub const BALANCE_LEAF_KEY: u8 = 1;
pub const NONCE_LEAF_KEY: u8 = 2;
pub const CODE_KECCAK_LEAF_KEY: u8 = 3;
pub const CODE_SIZE_LEAF_KEY: u8 = 4;
pub const HEADER_STORAGE_OFFSET: U256 = U256::from_limbs([64, 0, 0, 0]);
pub const CODE_OFFSET: U256 = U256::from_limbs([128, 0, 0, 0]);
pub const MAIN_STORAGE_OFFSET: U256 = U256::from_limbs([0, 0, 0, 1 << 56]);

// Extension node indices
pub const EXTENSION_MARKER_INDEX: usize = 0;
pub const EXTENSION_STEM_INDEX: usize = 1;
pub const EXTENSION_C1_INDEX: usize = 2;
pub const EXTENSION_C2_INDEX: usize = 3;
