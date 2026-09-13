# AI Tool Usage Disclosure

In the interest of transparency, this document details where and how AI assistance (Claude,
Anthropic) was used throughout the development of Percolate.

## Summary

Claude was used as a pair-programming and architecture-review assistant throughout this project's
development, via an extended conversational session covering design, implementation, debugging,
and documentation. All code was reviewed, tested, and run by the author before being committed;
no code was committed without the author verifying it compiled/passed tests locally first.

## Where AI assistance was used

- **Architecture design**: identifying that SwapVM's built-in whitelist opcodes expose allowlists
  in plaintext, and designing the `PercolateGate` opcode / `PercolateVerifier` contract split
  (mutable on-chain root, decoupled from SwapVM's order/strategy immutability) as an alternative.
- **`core/` (Rust)**: `build_tree_from_leaves` / `siblings_for_leaves` (leaf-based Merkle
  functions, separating public commitments from private secrets) were AI-drafted and
  human-reviewed; a related odd-leaf-count bug was diagnosed and fixed with AI assistance.
- **`host/` (Rust)**: refactor from hardcoded dummy data to real allowlist/taker inputs
  (`gen_allowlist` binary, environment-variable-driven `main.rs`) was AI-drafted.
- **`contracts/` (Solidity)**: `PercolateVerifier.sol` and its Foundry test suite were AI-drafted,
  based on the real `IRiscZeroVerifier` interface (verified against risc0-ethereum's published
  source rather than assumed).
- **`swap-vm` fork (Solidity)**: the `PercolateGate` opcode implementation, its registration in
  `OpcodeList.sol`/`LimitOpcodes.sol`, and an integration test exercising it through a forked-Sepolia
  call to `LimitSwapVMRouter.swap()`, were AI-drafted after inspecting the actual SwapVM source
  (`TokenValidators.sol`'s `OnlyTakerTokenBalanceNonZero` pattern, `VM.sol`'s taker-calldata
  mechanism) to match existing conventions rather than guessing them.
- **`app/` (Tauri + React + TypeScript)**: the desktop app's Maker-mode UI (wallet connection via
  wagmi/viem, allowlist generation, per-maker `PercolateVerifier` deployment, token approvals, and
  EIP-712 order signing) was AI-drafted and iteratively debugged with AI assistance.
- **`OrderBuilderHelper.sol`**: a view-only helper contract that wraps SwapVM's own
  `MakerTraitsLib`/`TakerTraitsLib` encoding logic, used by the app to avoid reimplementing SwapVM's
  bit-packed order encoding independently in JavaScript.
- **Debugging**: numerous build/deploy/runtime issues (Makefile shell-substitution bugs, Foundry
  `.gitmodules` path corruption after a `git subtree` merge, RISC Zero Groth16's Docker/x86
  requirement, Sourcify verification failures) were diagnosed with AI assistance, generally by
  fetching and reading the actual upstream source/documentation rather than guessing.
- **Documentation**: this repository's `README.md`, the deployment roadmap, and this disclosure
  file were AI-drafted from the author's direction and the project's actual state.

## Where AI assistance was explicitly declined

The author requested help scripting a sequence of git commits designed to make the repository's
history appear organically developed over time, for a repository whose real history already
existed. This was declined on the grounds that it would misrepresent the development process to
reviewers, regardless of intent. The project's actual git history reflects work as it was
genuinely completed across the development period, including this AI-assisted portion.

## What was not AI-generated

- All work up to and including commit
  [`56fbde1`](https://github.com/0xMoZi/Percolate/commit/56fbde12449f08ba04235e827b1b21317a5c1d1c)
  ("feat(host): publisher app generates proof from guest program") was written independently by
  the author before any AI assistance began. This covers the initial `core`/`host`/`methods`
  scaffolding, the guest program, and the first working (dummy-data) end-to-end proof generation.
  Everything described elsewhere in this document as AI-assisted was built on top of that
  starting point during a later session.
- All real transactions (contract deployments, `setRoot` calls, token approvals, order signing) on
  Sepolia were executed by the author, using their own wallet, at their own direction.
- The decision to scope this project to the Maker flow, given time constraints, and to leave the
  Taker flow's Groth16 proving step unimplemented rather than misrepresent its status, was made by
  the author.
- All licensing compliance steps (attribution for SwapVM under LicenseRef-Degensoft-SwapVM-1.1,
  publishing the modified `swap-vm` fork) were carried out by the author following AI-surfaced
  guidance on the license's actual terms.
