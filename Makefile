start:
	cargo build && target/debug/scavenger
test:
	cargo test
debug:
	cargo build
release:
	cargo build --release
release-gpu:
	cargo build --release --features=opencl
