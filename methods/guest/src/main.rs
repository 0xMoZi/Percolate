#![no_main]

use alloy_sol_types::SolValue;
use percolate_core::{PrivateInput, PublicInput, compute_journal};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let priv_in: PrivateInput = env::read();
    let pub_in: PublicInput = env::read();

    let journal = compute_journal(&pub_in, &priv_in)
        .unwrap_or_else(|e| panic!("Percolate guest: failed: {:?}", e));

    env::commit_slice(&*journal.abi_encode().as_slice());
}
