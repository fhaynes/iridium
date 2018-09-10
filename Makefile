all:
	test production

test:
	cargo test

production:
	cargo build --release
	mv target/debug/iridium /usr/local/bin/
	chmod ugo+x /usr/local/bin/

dev:
	cargo build
	mv target/debug/iridium /usr/local/bin/
	chmod ugo+x /usr/local/bin/
