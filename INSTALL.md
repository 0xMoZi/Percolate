# Installation Guide

## Prerequisites

- **Rust + RISC Zero toolchain**
  ```bash
  curl -L https://risczero.com/install | bash
  rzup install
  ```
- **Foundry** (for `contracts/`)
  ```bash
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
  ```
- **Node.js 18+** and npm (for `app/`)
- **Docker** — only required if you intend to generate a Groth16-compressed proof. Not required to
  run the Maker flow in the app. See [Known Limitations](README.md#known-limitations).

## 1. Clone

```bash
git clone https://github.com/0xMoZi/Percolate.git
cd Percolate
```

## 2. Initialize submodules (required for `contracts/`)

`contracts/lib/forge-std` and `contracts/lib/risc0-ethereum` are git submodules. A plain
`git clone` leaves them empty — this is the single most common setup failure.

```bash
git submodule update --init --recursive
```

## 3. Build and test `core`/`host` (Rust)

```bash
cargo build --release
cargo test -p percolate-core
```

Generate an allowlist and a proof from the CLI:

```bash
make run-host TAKER=<index> ADDR=<your Sepolia address>
```

## 4. Build and test `contracts/` (Foundry)

```bash
cd contracts
forge build
forge test
```

## 5. Run the desktop app (Tauri)

```bash
cd app
yarn install
```

Create `app/.env` with a free [WalletConnect / Reown](https://reown.com) Project ID:

```
VITE_WALLETCONNECT_PROJECT_ID=your_project_id_here
```

Then:

```bash
make dev
```

This opens the desktop app. Connect a Sepolia wallet with test ETH, WETH, and USDC to exercise the
Maker flow (see [README](README.md) for token addresses and faucets).

## 6. (Optional) SwapVM fork with `PercolateGate`

The `PercolateGate` opcode lives in a separate fork, not this repository:
[0xMoZi/swap-vm-percolate](https://github.com/0xMoZi/swap-vm-percolate).

```bash
git clone https://github.com/0xMoZi/swap-vm-percolate.git
cd swap-vm-percolate
yarn install
forge build
forge test
```

## Troubleshooting

- **`forge build` fails with missing `node_modules` sources** — run `yarn install` (or
  `npm install`) in that project; some dependencies (`forge-std`, `@1inch/*`) are managed via
  Yarn/npm, not git submodules, in the `swap-vm-percolate` fork specifically.
- **Docker permission denied** (Linux/WSL2) — after installing Docker, run
  `sudo usermod -aG docker $USER`, then fully restart your shell/WSL session
  (`wsl --shutdown` from Windows, then reopen), not just log out of the terminal.
