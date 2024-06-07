build:
	cargo build
build_release:
	cargo build --release
aarch64:
	mkdir bin
	cross build --release --target aarch64-unknown-linux-gnu
	cp target/aarch64-unknown-linux-gnu/release/alfred-ai-callback bin/
	rm -rf bin