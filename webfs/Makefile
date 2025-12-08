.PHONY: build run
ENV_FILE:=.env.local

build:
	ENV_PROFILE=$(ENV_FILE) cargo build --bin webfs

utils:
	ENV_PROFILE=$(ENV_FILE) cargo build --bin utils
	RUST_BACKTRACE=1 ENV_PROFILE=$(ENV_FILE) ./target/debug/utils

release:
	cargo build --bin webfs --release

webfs: build
	RUST_BACKTRACE=1 ENV_PROFILE=$(ENV_FILE) ./target/debug/webfs --config=/ntc/filesync/rust/webfs/config-test.yaml

rel-webfs: release
	ENV_PROFILE=$(ENV_FILE) ./target/release/webfs --config=/ntc/filesync/rust/webfs/config.yaml
