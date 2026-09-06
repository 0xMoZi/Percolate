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

pub fn leaf_from_secret(secret: &[u8; 32]) -> [u8; 32] {
    hash2(DOMAIN_LEAF, secret)
}

pub fn verify_merkle_proof(
    leaf: [u8; 32],
    leaf_index: u32,
    siblings: &[[u8; 32]],
    expected_root: &[u8; 32],
) -> bool {
    let mut current = leaf;
    let mut index = leaf_index;

    for sibling in siblings {
        current = if index & 1 == 0 {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
        index >>= 1;
    }

    index == 0 && &current == expected_root
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

    fn build_tree(secrets: &[[u8; 32]]) -> (Vec<[u8; 32]>, [u8; 32]) {
        let mut level: Vec<[u8; 32]> = secrets.iter().map(leaf_from_secret).collect();
        while level.len() > 1 {
            level = level
                .chunks(2)
                .map(|pair| hash_pair(&pair[0], &pair[1]))
                .collect();
        }
        (secrets.to_vec(), level[0])
    }

    fn siblings_for(secrets: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        let mut level: Vec<[u8; 32]> = secrets.iter().map(leaf_from_secret).collect();
        let mut idx = index;
        let mut sibs = vec![];
        while level.len() > 1 {
            let sib_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            sibs.push(level[sib_idx]);
            level = level
                .chunks(2)
                .map(|pair| hash_pair(&pair[0], &pair[1]))
                .collect();
            idx /= 2;
        }
        sibs
    }

    fn setup_sample_tree() -> (Vec<[u8; 32]>, [u8; 32]) {
        let secrets = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let (secrets, root) = build_tree(&secrets);

        (secrets, root)
    }

    #[test]
    fn test_verify_merkle_proof_success() {
        let (secrets, root) = setup_sample_tree();

        let target_idx = 2;
        let target_secret = secrets[target_idx];
        let target_leaf = leaf_from_secret(&target_secret);
        let siblings = siblings_for(&secrets, target_idx);

        let is_valid = verify_merkle_proof(target_leaf, target_idx as u32, &siblings, &root);

        assert!(is_valid);
    }

    #[test]
    fn test_verify_merkle_proof_invalid_leaf() {
        let (secrets, root) = setup_sample_tree();
        let target_idx = 2;

        let fake_secret = [99u8; 32];
        let fake_leaf = leaf_from_secret(&fake_secret);
        let siblings = siblings_for(&secrets, target_idx);

        let is_valid = verify_merkle_proof(fake_leaf, target_idx as u32, &siblings, &root);
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_merkle_proof_wrong_root() {
        let (secrets, _root) = setup_sample_tree();
        let target_idx = 2;

        let target_leaf = leaf_from_secret(&secrets[target_idx]);
        let siblings = siblings_for(&secrets, target_idx);
        let fake_root = [88u8; 32];

        let is_valid = verify_merkle_proof(target_leaf, target_idx as u32, &siblings, &fake_root);
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_merkle_proof_wrong_index() {
        let (secrets, root) = setup_sample_tree();
        let target_idx = 2;

        let target_leaf = leaf_from_secret(&secrets[target_idx]);
        let siblings = siblings_for(&secrets, target_idx);

        let is_valid = verify_merkle_proof(target_leaf, 1, &siblings, &root);
        assert!(!is_valid);
    }
}
