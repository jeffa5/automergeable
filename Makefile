test:
	cargo test
	wasm-pack test automergeable --node -- --no-default-features
