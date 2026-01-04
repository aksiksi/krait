# Krait Integration Tests

Simplified Rust integration test suite for the Krait eBPF mesh VPN project using libtest-mimic.

## Overview

This crate provides essential integration tests that verify core eBPF functionality:

1. **eBPF Program Attachment** - Verifies that programs are loaded correctly and appear in `tc filter` output
2. **Traffic Generation with eBPF** - Generates iperf3 traffic (30M bytes) with eBPF programs attached

## Usage

```bash
# Run all tests
./krait-tests --interface wg0 --target-ip 10.10.0.1

# Run with custom parameters  
./krait-tests --interface wg0 --target-ip 10.10.0.1 --num-bytes 50M --bandwidth 100M --iperf-port 5201

# Save results to JSON file
./krait-tests --interface wg0 --target-ip 10.10.0.1 --output-file results.json

# Run tests sequentially (recommended for eBPF)
./krait-tests --test-threads 1 --interface wg0 --target-ip 10.10.0.1

# List available tests
./krait-tests --list
```

## Command Line Options

- `--interface` - WireGuard interface name (default: wg0)
- `--num-bytes` - Amount of data to transfer via iperf3 (default: 30M)
- `--bandwidth` - Bandwidth limit for iperf3 (optional, e.g. 100M, 1G)
- `--target-ip` - Target IP for iperf3 traffic (default: 10.10.0.1)
- `--iperf-port` - Port for iperf3 server (default: 5201)
- `--output-file` - Save JSON results to file (optional)

## Test Details

### 1. eBPF Program Attachment Test

- Creates a `KraitAgent` instance
- Attaches eBPF programs to the specified interface
- Verifies programs appear in `tc filter show dev <interface> ingress/egress` output
- Looks for `krait_ingress` and `krait_egress` program names

### 2. Traffic Generation Test

- Creates and attaches eBPF programs 
- Runs `iperf3 --client <target> -n <num-bytes> --json` with optional bandwidth limit
- Parses JSON output to extract throughput statistics
- Optionally saves results to JSON file

## Building

```bash
# Debug build
just build-tests

# Release build (static linking with musl)
just release-tests
```

## Integration with NixOS Tests

This test suite is used by the NixOS integration test (`tests/nixos/wireguard.nix`) to verify eBPF functionality in a complete VM environment with WireGuard tunnels.

## Requirements

- Root privileges (for eBPF program loading)
- `tc` command available (from iproute2)
- `iperf3` command available
- Target system running iperf3 server on specified port
- WireGuard interface configured and active

## Example Output

```
test ebpf_program_attachment ... ok
test traffic_generation_with_ebpf ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Example JSON results file:
```json
{
  "success": true,
  "total_bytes": 31457280,
  "duration_seconds": 2.15,
  "throughput_mbps": 117.2,
  "tests_passed": 2,
  "errors": []
}
```