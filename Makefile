all:
	test production

test:
	cargo test

production:
	cargo build --release

dev:
	cargo build
