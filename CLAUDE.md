# CLAUDE.md - Krait Project Context

## What is Krait?

High-performance mesh VPN using kernel WireGuard + eBPF for identity-based policy enforcement. Goal: eliminate userspace overhead of traditional mesh VPNs.

**Key Innovation**: Policy decisions happen in kernel eBPF maps, not userspace ACLs.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   krait-agent   │    │   krait-ebpf    │
│ (Rust userspace)│───▶│ (kernel eBPF)   │
│ • WG management │    │ • Policy engine │
│ • Map updates   │    │ • Identity tags │
└─────────────────┘    └─────────────────┘
          │                       │
          ▼                       ▼
┌─────────────────────────────────────────┐
│        WireGuard (wg0 interface)        │
│           Kernel data plane             │
└─────────────────────────────────────────┘
```

## Project Structure

- `krait-agent/` - Userspace Rust binary (loads eBPF, manages WG)
- `krait-ebpf/` - eBPF TC classifier programs
- `krait-common/` - Shared types between agent and eBPF
- `tests/` - NixOS integration tests

## Quick Start

```bash
# Setup environment
nix develop

# Build everything
just release

# Run integration test
just integration-test

# Run agent manually
sudo ./target/x86_64-unknown-linux-musl/release/krait --iface wg0
```

## How It Works

1. **Identity Mapping**: WG peer index → identity tag (u32)
2. **Policy Engine**: eBPF maps store `(src_identity, dst_identity) → allow/deny`
3. **Packet Flow**: eBPF programs on wg0 check policy, allow/drop packets
4. **Updates**: Agent updates eBPF maps when peers/policies change

## Build Commands

```bash
just build-all         # Debug build
just release           # Release build
just test              # Unit tests
just integration-test  # Full NixOS test
```

## Common Issues

**"No such file or directory"**: Agent can't find `tc` command
- Solution: Ensure `iproute2` is in PATH or systemd service path

## Requirements

- Linux kernel ≥5.11 (WireGuard support w/ memcg-based accounting)
- Rust nightly (for eBPF compilation)
- sudo + CAP_SYS_ADMIN (for eBPF loading)

Use `nix develop` to get all dependencies automatically.

