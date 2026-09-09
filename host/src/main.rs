use alloy_sol_types::SolValue;
use percolate_core::{
    Journal, PrivateInput, PublicInput, build_tree_from_leaves, siblings_for_from_leaves,
};
use percolate_methods::PERCOLATE_GUEST_ELF;
use risc0_zkvm::{ExecutorEnv, default_prover};
use std::{
    env, fs, panic,
    time::{SystemTime, UNIX_EPOCH},
};

fn parse_hex32(hex_str: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_str.trim_start_matches("0x")).expect("invalid hex");
    bytes.try_into().expect("byte length should be 32 byte")
}

fn parse_hex20(hex_str: &str) -> [u8; 20] {
    let bytes = hex::decode(hex_str.trim_start_matches("0x")).expect("invalid hex");
    bytes.try_into().expect("byte length should be 20 byte")
}

fn main() {
    let allowlist_path = env::var("ALLOWLIST_PATH").unwrap_or_else(|_| "allowlist.json".into());
    let allowlist_raw = fs::read_to_string(&allowlist_path)
        .unwrap_or_else(|_| panic!("failed to read {allowlist_path}, run gen_allowlist first"));
    let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_raw).unwrap();
    let leaves: Vec<[u8; 32]> = allowlist_json["leaves"]
        .as_array()
        .expect("allowlist.json format invalid")
        .iter()
        .map(|v| parse_hex32(v.as_str().unwrap()))
        .collect();

    let secret = parse_hex32(&env::var("TAKER_SECRET").expect("set TAKER_SECRET"));
    let taker_idx: usize = env::var("TAKER_INDEX")
        .expect("set TAKER_INDEX")
        .parse()
        .expect("TAKER_INDEX should be a number");
    let caller_binding = parse_hex20(&env::var("TAKER_ADDRESS").expect("set TAKER_ADDRESS"));

    let (_, root) = build_tree_from_leaves(&leaves);
    let siblings = siblings_for_from_leaves(&leaves, taker_idx);

    println!("Merkle root: 0x{}", hex::encode(root));

    let priv_in = PrivateInput {
        secret,
        leaf_index: taker_idx as u32,
        siblings,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before epoch")
        .as_secs();

    let pub_in = PublicInput {
        merkle_root: root,
        caller_binding,
        valid_until: now + 3600,
        now,
    };

    let env = ExecutorEnv::builder()
        .write(&priv_in)
        .expect("failed to serialize private input")
        .write(&pub_in)
        .expect("failed to serialize public input")
        .build()
        .expect("failed to build ExecutorEnv");

    let prover = default_prover();
    let prove_info = prover
        .prove(env, PERCOLATE_GUEST_ELF)
        .expect("proving failed");
    let receipt = prove_info.receipt;

    let journal =
        Journal::abi_decode(&receipt.journal.bytes).expect("failed to abi-decode journal");
    println!("Proof successfull! Journal: {:?}", journal);
    println!(
        "journal bytes (hex): 0x{}",
        hex::encode(&receipt.journal.bytes)
    );
}
