# Krait Testing Framework

This directory contains the testing infrastructure for Krait, including both Rust integration tests and NixOS-based end-to-end tests.

## Overview

Krait uses a two-tier testing approach:

1. **Rust Integration Tests** (`../krait-tests/`) - Statically-linked Rust test suite using libtest-mimic that tests eBPF functionality with real network traffic
2. **NixOS End-to-End Tests** (`nixos/`) - Full system tests using NixOS VMs with WireGuard mesh networking

## Directory Structure

```
tests/
├── nixos/                # NixOS integration tests
│   ├── wireguard.nix     # Main test definition
│   ├── *.priv/*.pub      # WireGuard keys for test
└── README.md             # This file

krait-tests/              # Rust integration test suite (separate crate)
├── Cargo.toml           # Test dependencies and configuration
└── src/
    └── main.rs          # Self-contained test runner using libtest-mimic
```

## Rust Integration Test

### Purpose

The Rust integration test suite (`krait-tests`) is designed to:

- Load and attach eBPF programs to a WireGuard interface
- Generate controlled network traffic using iperf3
- Collect eBPF logs and metrics
- Validate that eBPF programs process packets correctly  
- Run structured tests with nice output using libtest-mimic
- Output structured test results in JSON format

### Usage

The integration test suite provides individual tests with proper test framework output:

```bash
# Run all tests
krait-tests --interface wg0 --target-ip 10.10.0.1 --duration-seconds 5

# Run with output file
krait-tests --interface wg0 --target-ip 10.10.0.1 --output-file results.json

# Run tests sequentially (recommended for eBPF)
krait-tests --test-threads 1 --interface wg0 --target-ip 10.10.0.1

# List available tests
krait-tests --list
```

### Test Structure

The test suite includes the following essential tests:

1. **ebpf_program_attachment** - Loads eBPF programs and verifies they appear in `tc filter` output
2. **traffic_generation_with_ebpf** - Generates iperf3 traffic (30M bandwidth) with eBPF programs attached

### Implementation Details

- **Static linking**: Built with musl target for portability
- **Test Framework**: Uses libtest-mimic for proper test runner output
- **eBPF Integration**: Uses the krait-agent library to load and manage eBPF programs
- **Traffic Generation**: Uses iperf3 with fixed 30M bandwidth for consistent testing
- **Verification**: Checks `tc filter` output to confirm eBPF program attachment
- **Results**: Outputs structured JSON with throughput, packet counts, and errors

## NixOS End-to-End Test

### Purpose

The NixOS test creates a complete testing environment with:

- Two VMs (server and client) connected via WireGuard
- Krait agents running on both nodes
- eBPF programs attached to WireGuard interfaces
- End-to-end traffic testing and validation

### Test Sequence

1. **VM Startup**: Creates server and client VMs
2. **WireGuard Setup**: Establishes encrypted tunnel between VMs
3. **Krait Agent Deployment**: Starts krait-agent service on both nodes
4. **eBPF Attachment**: Verifies eBPF programs are loaded and attached
5. **Connectivity Testing**: Basic ping tests through tunnel
6. **Throughput Testing**: iperf3 performance baseline
7. **Integration Testing**: Runs the Rust integration test
8. **Results Collection**: Gathers logs and metrics

### Running the Tests

```bash
# Full integration test (includes building binaries)
just integration-test

# Interactive test for debugging
just integration-test-interactive

# Build just the test binaries
just release-tests
just release-agent
```

### Test Configuration

The test uses hardcoded WireGuard keys for reproducibility:

- **Server**: `10.10.0.1/24` on port 51820
- **Client**: `10.10.0.2/24`, connects to server
- **Interface**: `wg0` on both nodes

## How It Works Together

1. **Build Phase**: 
   - `just release-agent` builds the krait agent
   - `just release-integration-test` builds the integration test binary
   - Both are statically linked for easy deployment

2. **NixOS Test Execution**:
   - NixOS test copies both binaries into VM environment
   - Starts krait-agent as systemd service on both VMs
   - Runs integration test binary to generate traffic and collect metrics

3. **Validation**:
   - Systemd logs show agent startup and eBPF attachment
   - Integration test logs show packet processing in eBPF
   - JSON output provides quantitative metrics

## Key Features

### Realistic Testing Environment
- Real WireGuard tunnels with actual encryption
- Kernel-level eBPF program execution
- Authentic network stack interaction

### Comprehensive Coverage
- eBPF program loading and attachment
- Traffic classification and policy enforcement
- Logging and observability features
- Error handling and edge cases

### Automated Validation
- Structured test output for CI/CD integration
- Performance benchmarking capabilities
- Regression detection for eBPF changes

### Debugging Support
- Interactive test mode for development
- Detailed logging at all levels
- Direct VM access for troubleshooting

## Development Workflow

1. **Make changes** to eBPF programs or agent code
2. **Run integration test** to verify functionality:
   ```bash
   just integration-test
   ```
3. **Debug issues** using interactive mode:
   ```bash
   just integration-test-interactive
   # Then connect to VM and inspect logs
   ```
4. **Iterate** based on test results and eBPF logs

## Expected Output

Successful test run produces:

- **Agent logs**: eBPF program loading and attachment
- **eBPF logs**: Packet processing with IP addresses and ports
- **Traffic metrics**: Throughput, packet counts, duration
- **JSON results**: Structured test outcomes for analysis

Example JSON output:
```json
{
  "success": true,
  "total_bytes": 52428800,
  "duration_seconds": 3.02,
  "throughput_mbps": 138.7,
  "packets_processed": 1245,
  "errors": []
}
```

This testing framework provides confidence that Krait's eBPF-based networking functions correctly in real-world scenarios while maintaining the performance benefits of kernel-space execution.