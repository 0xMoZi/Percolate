TAKER ?= 0
ADDR  ?= 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

run-host:
	cargo run --bin gen_allowlist -- 8
	TAKER_SECRET=$$(jq -r .secret taker_secret_$(TAKER).json) \
	TAKER_INDEX=$(TAKER) \
	TAKER_ADDRESS=$(ADDR) \
	cargo run --release --bin percolate-host

dev:
	cd app && yarn run tauri dev
