use alloy_sol_types::SolValue;
use percolate_core::{Journal, PrivateInput, PublicInput, build_tree, siblings_for};
use percolate_methods::PERCOLATE_GUEST_ELF;
use risc0_zkvm::{ExecutorEnv, default_prover};

fn main() {
    let secrets: Vec<[u8; 32]> = (0..8u8).map(|i| [i; 32]).collect();
    let taker_index = 3usize;
    let (_, root) = build_tree(&secrets);
    let siblings = siblings_for(&secrets, taker_index);

    println!("Merkle root: 0x{}", hex::encode(root));

    let priv_in = PrivateInput {
        secret: secrets[taker_index],
        leaf_index: taker_index as u32,
        siblings,
    };
    let pub_in = PublicInput {
        merkle_root: root,
        caller_binding: [0xAAu8; 20], // ganti dengan alamat EVM taker asli
        valid_until: 9_999_999_999,
        now: 1,
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
    println!("Proof successful! Journal: {:?}", journal);
    println!(
        "journal bytes (hex): 0x{}",
        hex::encode(&receipt.journal.bytes)
    );
}
