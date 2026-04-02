build:
	cargo build

run:
	cargo clean
	cargo run

clean:
	cargo clean

check:
	cargo check

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt