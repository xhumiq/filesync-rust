.PHONY: build run

build:
	cargo build --bin webfs
	cargo build --bin rssfeed

webfs: build
	./target/debug/webfs --config=/ntc/filesync/rust/webfs/config-test.yaml

rss: build
	./target/debug/rssfeed --days=7 --lang=en --channel=videos-all --config=/ntc/filesync/rust/webfs/config-test.yaml --log-file ../logs/rssfeed.log
	./target/debug/rssfeed --days=7 --lang=zh --channel=videos-all --config=/ntc/filesync/rust/webfs/config-test.yaml --log-file ../logs/rssfeed.log
