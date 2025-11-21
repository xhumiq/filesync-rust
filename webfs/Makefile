.PHONY: build run

build:
	cargo build --bin webfs

release:
	cargo build --bin webfs --release

webfs: build
	./target/debug/webfs --config=/ntc/filesync/rust/webfs/config-test.yaml

rel-webfs: release
	./target/release/webfs --config=/ntc/filesync/rust/webfs/config-test.yaml

rss: build
	./target/debug/rssfeed --days=7 --lang=en --channel=videos-all --config=/ntc/filesync/rust/webfs/config-test.yaml --log-file ../logs/rssfeed.log
	./target/debug/rssfeed --days=7 --lang=zh --channel=videos-all --config=/ntc/filesync/rust/webfs/config-test.yaml --log-file ../logs/rssfeed.log
