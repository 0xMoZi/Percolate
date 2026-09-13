# Percolate

Private, RISC Zero-verified allowlist gating for swaps on 1inch's SwapVM.

Powered by SwapVM — © Degensoft Ltd 2025. The `PercolateGate` opcode is a Modification to SwapVM
under [LicenseRef-Degensoft-SwapVM-1.1](https://github.com/1inch/swap-vm/blob/main/LICENSES/SwapVM-1.1.txt);
the modified source is published in full at
[0xMoZi/swap-vm-percolate](https://github.com/0xMoZi/swap-vm-percolate) per the license's copyleft terms.

## The problem

SwapVM ships a built-in whitelist opcode (`WhitelistCoequal` / `WhitelistSequential`) that encodes
allowed taker addresses **in plaintext inside the program bytecode**. Anyone can decode the order
and see exactly who is on a maker's allowlist — no privacy for either the maker's business
relationships or the takers themselves.

## The idea

Percolate replaces that with a RISC Zero zkVM proof: takers prove Merkle-tree membership in the
maker's allowlist **off-chain**, and only a one-time-use nullifier is revealed on-chain. Neither the
taker's identity nor their position in the allowlist is ever exposed. The maker's allowlist root
lives in a small, independently-deployed `PercolateVerifier` contract — decoupled from SwapVM's
order/strategy lifecycle — so it can be updated any time with a single transaction, no re-signing
or re-shipping required.

## Status: Maker flow complete, Taker flow partial (see Known Limitations)

Everything below is **real** — deployed on Sepolia, exercised through real transactions, not mocked
unless stated otherwise.

### What works end-to-end

- **`core/`** — Merkle tree construction and sibling-path computation over public leaf commitments
  (not raw secrets), so anyone can rebuild a proof path without learning other participants'
  secrets. 15 unit tests.
- **`host/`** — `gen_allowlist` (maker-side commitment generation) and a refactored prover binary
  that reads real allowlist/taker data instead of hardcoded values.
- **`contracts/`** — `PercolateVerifier.sol`: mutable allowlist root, nullifier tracking, calls the
  official permissionless `RiscZeroVerifierRouter` on Sepolia (no custom verifier deployment
  needed). Deployed and verified.
- **SwapVM integration** (in a separate vendored `swap-vm` fork —
  [0xMoZi/swap-vm-percolate](https://github.com/0xMoZi/swap-vm-percolate), not in this repo's
  history, per SwapVM-1.1's copyleft requirements for Modified Works) - a custom `PercolateGate`
  opcode that stores only a verifier *address* in the maker's program bytes (never allowlist
  data), and reads the taker's proof from taker-supplied calldata at swap time. Covered by an
  integration test that forks Sepolia and calls `LimitSwapVMRouter.swap()` end-to-end,
  confirming the opcode dispatches correctly, root/nullifier checks work, and token transfers
  execute. `RiscZeroVerifierRouter.verify()` is mocked in that test (see limitations).
- **`app/`** — a Tauri desktop app implementing the full **Maker** flow with real wallet
  transactions: generate allowlist, deploy a per-maker `PercolateVerifier`, set the root, approve
  tokens, and sign a gas-free EIP-712 order. Order encoding is produced by calling a small
  view-only `OrderBuilderHelper.sol` contract that wraps SwapVM's own `MakerTraitsLib`/`LimitSwap`
  libraries.

### Known limitations

**Groth16 proof compression is not wired into the app.** RISC Zero's STARK proving works natively
and completes in seconds (verified via the `percolate-host` CLI). Compressing that proof into a
Groth16 SNARK - required for cheap on-chain verification - currently requires a Docker container
running on x86 (`risc0-groth16`); this is a constraint of RISC Zero's tooling itself, not a
Percolate design choice. As a result, the Taker flow in the desktop app is a disabled placeholder;
the on-chain verification path it would call into is implemented and integration-tested, but not
exercised with a real Groth16 seal in this build.

**The `PercolateGate` opcode and its integration test live in a separate `swap-vm` checkout**, not
in this repository's commit history, because they modify 1inch's `swap-vm` source directly.

## Architecture

```
Maker                                   Taker
  │                                       │
  ├─ generate secrets + public leaves     ├─ receive secret + index (out-of-band)
  ├─ deploy PercolateVerifier             ├─ generate proof locally (RISC Zero STARK — works)
  ├─ setRoot() - mutable, no re-deploy    ├─ [compress to Groth16 — blocked, see above]
  ├─ approve tokens                       └─ submit swap() with proof (untested with real seal)
  └─ sign order (EIP-712, program
     embeds PercolateGate + verifier addr)
```

At swap time: `LimitSwapVMRouter.swap()` -> `PercolateGate` opcode reads proof from taker calldata ->
`PercolateVerifier.verifyAndConsume()` checks root/caller/nullifier -> `RiscZeroVerifierRouter.verify()`.

## Deployed contracts (Sepolia)

See [`contracts/deployment.txt`](contracts/deployment.txt) for the full, current list, including the
Aqua/LimitSwapVMRouter instances deployed for this project (no official Sepolia deployment of
Aqua/SwapVM exists, per confirmation from the 1inch team) and the `OrderBuilderHelper` used by the
app to construct valid order/taker-data encodings.

## Running it

```bash
# core + host (proof generation, CLI)
cargo build --release
make run-host TAKER=<index> ADDR=<your Sepolia address>

# contracts
cd contracts && forge test

# app
cd app && npm install && npm run tauri dev
```
See [INSTALL.md](INSTALL.md) for full setup instructions,

## Why this matters

This is for anyone who needs **trustless enforcement of membership without that membership being
public**: OTC desks pricing known counterparties without exposing a client list, DAOs running
gated allocations, compliance-gated pools that must still be verifiable on-chain. RISC Zero zkVM
proofs make it possible to enforce "you are on this list" without ever publishing the list.
