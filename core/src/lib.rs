use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInput {
    pub secret: [u8; 32],
    pub leaf_index: u32,
    pub siblings: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInput {
    pub merkle_root: [u8; 32],
    pub amount_in: u128,
    pub caller_binding: [u8; 20],
    pub valid_until: u64,
    pub reserve_in: u128,
    pub reserve_out: u128,
    pub fee_base_bps: u16,
    pub fee_max_bps: u16,
    pub decay_start: u64,
    pub decay_duration: u64,
    pub now: u64,
}

sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct Journal {
        bytes32 nullifier;
        bytes32 merkle_root;
        uint128 amount_in;
        uint128 amount_out;
        address caller_binding;
        uint64 valid_until;
    }
}

pub const BPS_BASE: u128 = 10_000;

#[derive(Debug, PartialEq, Eq)]
pub enum PercolateError {
    MerkleRootMismatch,
    ExpiredProof,
    DecayWindowNotStarted,
    ZeroReserves,
    InsufficientLiquidity,
    ZeroAmountIn,
}
