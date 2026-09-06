use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DOMAIN_LEAF: &[u8] = b"PERCOLATE_LEAF_V1";
const DOMAIN_NULLIFIER: &[u8] = b"PERCOLATE_NULLIFIER_V1";

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

fn hash2(domain: &[u8], data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(data);
    hasher.finalize().into()
}

fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn leaf_from_secret(secret: &[u8; 32]) -> [u8; 32] {
    hash2(DOMAIN_LEAF, secret)
}

#[cfg(test)]
mod percolate_core {
    use super::*;

    #[test]
    fn test_leaf_from_secret_determinism() {
        let secret = [42u8; 32];

        let leaf1 = leaf_from_secret(&secret);
        let leaf2 = leaf_from_secret(&secret);

        assert_eq!(leaf1, leaf2);
    }

    #[test]
    fn test_domain_separation() {
        let secret = [42u8; 32];

        let leaf_hash = hash2(DOMAIN_LEAF, &secret);
        let nullifier_hash = hash2(DOMAIN_NULLIFIER, &secret);

        assert_ne!(leaf_hash, nullifier_hash);
    }

    #[test]
    fn test_hash_pair_order_sensitivity() {
        let left = [42u8; 32];
        let right = [24u8; 32];

        let parent = hash_pair(&left, &right);
        let parent_swapped = hash_pair(&right, &left);

        assert_ne!(parent, parent_swapped);
    }
}
