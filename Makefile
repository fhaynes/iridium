all:
	test production

test:
	cargo test

production:
	cargo build --release

dev:
	cargo build
	mv target/debug/iridium /usr/local/bin/
	chmod ugo+x /usr/local/bin/
