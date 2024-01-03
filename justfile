check: fmt lint test

fmt:
	cargo fmt

lint:
	cargo check
	cargo clippy --fix --allow-dirty

test:
	cargo test

install: check
	cargo install --path . --locked
