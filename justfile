check: fix test

fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

test:
	cargo test

install: check
	cargo install --path . --locked
