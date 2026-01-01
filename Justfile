# Krait build recipes

# Default recipe - build everything in debug mode
default: build-all

# Build the eBPF program (debug)
build-ebpf:
    cargo build -p krait-ebpf --target bpfel-unknown-none -Z build-std=core

# Build the agent (debug)
build-agent:
    cargo build -p krait-agent --target x86_64-unknown-linux-musl

# Build everything in debug mode (eBPF + agent)
build-all: build-ebpf build-agent

# Build everything in release mode
release: release-ebpf release-agent

# Build the eBPF program (release)
release-ebpf:
    cargo build -p krait-ebpf --target bpfel-unknown-none -Z build-std=core --release

# Build the agent (release)
release-agent:
    cargo build -p krait-agent --target x86_64-unknown-linux-musl --release

# Clean all build artifacts
clean:
    cargo clean

# Run the agent (requires sudo for eBPF loading)
run-agent: build-all
    sudo ./target/x86_64-unknown-linux-musl/debug/krait-agent

# Check code formatting
check-fmt:
    cargo fmt --check

# Format code
fmt:
    cargo fmt

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features

# Run tests
test:
    cargo test

# Development workflow - format, lint, test, build
dev: fmt clippy test build-all

# Show build status
status:
    @echo "=== Debug Build Status ==="
    @if [ -f "./target/bpfel-unknown-none/debug/krait-ebpf" ]; then \
        echo "✓ eBPF program (debug) built"; \
    else \
        echo "✗ eBPF program (debug) not built"; \
    fi
    @if [ -f "./target/x86_64-unknown-linux-musl/debug/krait-agent" ]; then \
        echo "✓ Agent (debug) built"; \
    else \
        echo "✗ Agent (debug) not built"; \
    fi
    @echo ""
    @echo "=== Release Build Status ==="
    @if [ -f "./target/bpfel-unknown-none/release/krait-ebpf" ]; then \
        echo "✓ eBPF program (release) built"; \
    else \
        echo "✗ eBPF program (release) not built"; \
    fi
    @if [ -f "./target/x86_64-unknown-linux-musl/release/krait-agent" ]; then \
        echo "✓ Agent (release) built"; \
    else \
        echo "✗ Agent (release) not built"; \
    fi