.PHONY: all build release install uninstall clean test lint fmt check help

# Default target
all: build

# Build debug version
build:
	cargo build

# Build optimized release version
release:
	cargo build --release

# Install to ~/.cargo/bin (or DESTDIR if specified)
DESTDIR ?= $(HOME)/.cargo/bin
install: release
	@mkdir -p $(DESTDIR)
	cp target/release/duperr $(DESTDIR)/duperr
	@echo "Installed duperr to $(DESTDIR)/duperr"

# Uninstall
uninstall:
	rm -f $(DESTDIR)/duperr
	@echo "Removed duperr from $(DESTDIR)"

# Clean build artifacts
clean:
	cargo clean

# Run all tests
test:
	cargo test

# Run clippy linter
lint:
	cargo clippy -- -W clippy::all -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting without changing
fmt-check:
	cargo fmt -- --check

# Full check (format, lint, test)
check: fmt-check lint test

# Run with example
run:
	cargo run -- --help

# Build documentation
doc:
	cargo doc --no-deps --open

# Show help
help:
	@echo "Available targets:"
	@echo "  make build     - Build debug version"
	@echo "  make release   - Build optimized release"
	@echo "  make install   - Install to ~/.cargo/bin"
	@echo "  make uninstall - Remove from ~/.cargo/bin"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make test      - Run all tests"
	@echo "  make lint      - Run clippy linter"
	@echo "  make fmt       - Format code"
	@echo "  make check     - Run fmt-check, lint, and test"
	@echo "  make doc       - Build and open documentation"
	@echo "  make help      - Show this help"
