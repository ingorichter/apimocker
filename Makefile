# Variables
BINARY_NAME=apimocker

# Build in debug mode
build:
	cargo build

# Build in release mode
release:               ## Build optimized binary and strip symbols
	cargo build --release
	strip target/release/$(BINARY_NAME)

# Run the server (requires --file)
run:
	cargo run -- --file data.json

# Run with a custom file
run-file:
	cargo run -- --file custom.json

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Lint with Clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Fully clean and remove binary
purge: clean
	rm -f target/debug/$(BINARY_NAME)
	rm -f target/release/$(BINARY_NAME)