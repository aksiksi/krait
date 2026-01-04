use std::{process::Command, time::Instant};

use anyhow::Result;
use clap::Parser;
use krait_agent::KraitAgent;
use libtest_mimic::{Arguments, Failed, Trial};
use serde_json::Value;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "wg0")]
    interface: String,

    #[clap(short, long, default_value = "10.10.0.1")]
    target_ip: String,

    #[clap(short, long, default_value = "30M")]
    num_bytes: String,

    #[clap(short, long)]
    bandwidth: Option<String>,

    #[clap(long, default_value = "5201")]
    iperf_port: u16,

    #[clap(short, long)]
    output_file: Option<String>,

    #[clap(flatten)]
    test_args: Arguments,
}

#[derive(Debug, Clone)]
struct TestConfig {
    interface: String,
    target_ip: String,
    num_bytes: String,
    bandwidth: Option<String>,
    iperf_port: u16,
    output_file: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    env_logger::init();

    let config = TestConfig {
        interface: opt.interface.clone(),
        target_ip: opt.target_ip.clone(),
        iperf_port: opt.iperf_port,
        num_bytes: opt.num_bytes.clone(),
        bandwidth: opt.bandwidth.clone(),
        output_file: opt.output_file.clone(),
    };

    let tests = vec![
        Trial::test("ebpf_program_attachment", {
            let config = config.clone();
            move || test_ebpf_program_attachment(&config)
        }),
        Trial::test("traffic_generation_with_ebpf", {
            let config = config.clone();
            move || test_traffic_generation(&config)
        }),
    ];

    libtest_mimic::run(&opt.test_args, tests).exit();
}

fn test_ebpf_program_attachment(config: &TestConfig) -> Result<(), Failed> {
    log::info!(
        "Testing eBPF program attachment to interface: {}",
        config.interface
    );

    // Create and attach the agent
    let mut agent = KraitAgent::new(&config.interface)
        .map_err(|e| Failed::from(format!("Failed to create agent: {}", e)))?;

    // The critical test: does attachment succeed without error?
    agent
        .attach()
        .map_err(|e| Failed::from(format!("Failed to attach eBPF programs: {}", e)))?;

    // Verify the clsact qdisc was created (required for TC attachment)
    let qdisc_output = Command::new("tc")
        .args(["qdisc", "show", "dev", &config.interface])
        .output()
        .map_err(|e| Failed::from(format!("Failed to run tc qdisc command: {}", e)))?;
    let qdisc_str = String::from_utf8_lossy(&qdisc_output.stdout);
    log::info!("TC qdisc output: {}", qdisc_str);

    if !qdisc_str.contains("clsact") {
        return Err(Failed::from(format!(
            "clsact qdisc not found on interface {}. eBPF attachment may have failed",
            config.interface
        )));
    }

    log::info!("✓ eBPF programs attached successfully (verified via Aya and clsact qdisc presence)");
    Ok(())
}

fn test_traffic_generation(config: &TestConfig) -> Result<(), Failed> {
    log::info!("Testing traffic generation with eBPF programs attached");

    // Create and attach the agent
    let mut agent = KraitAgent::new(&config.interface)
        .map_err(|e| Failed::from(format!("Failed to create agent: {}", e)))?;

    agent
        .attach()
        .map_err(|e| Failed::from(format!("Failed to attach eBPF programs: {}", e)))?;

    // Generate traffic with fixed 30M bandwidth
    log::info!(
        "Sending 30M of traffic via iperf3 limit to {}",
        config.target_ip
    );

    let start_time = Instant::now();

    let iperf_port = config.iperf_port.to_string();
    let mut args = vec![
        "--client",
        &config.target_ip,
        "--port",
        &iperf_port,
        "-n",
        &config.num_bytes,
        "--json",
    ];
    if let Some(bandwidth) = &config.bandwidth {
        args.extend(&["--bandwidth", bandwidth]);
    }
    let output = Command::new("iperf3")
        .args(&args)
        .output()
        .map_err(|e| Failed::from(format!("Failed to run iperf3: {}", e)))?;

    let actual_duration = start_time.elapsed().as_secs_f64();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Failed::from(format!("iperf3 failed: {}", stderr)));
    }

    // Parse JSON output to get statistics
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| Failed::from(format!("Failed to parse iperf3 JSON output: {}", e)))?;

    let bytes_transferred = json["end"]["sum_sent"]["bytes"]
        .as_u64()
        .ok_or_else(|| Failed::from("Could not parse bytes from iperf3 output"))?;

    if bytes_transferred == 0 {
        return Err(Failed::from("No traffic was generated"));
    }

    let throughput_mbps = (bytes_transferred as f64 * 8.0) / (actual_duration * 1_000_000.0);

    log::info!(
        "✓ Generated {} bytes of traffic in {:.2}s ({:.2} Mbps)",
        bytes_transferred,
        actual_duration,
        throughput_mbps
    );

    // Write results if requested
    if let Some(output_file) = &config.output_file {
        let results = serde_json::json!({
            "success": true,
            "total_bytes": bytes_transferred,
            "duration_seconds": actual_duration,
            "throughput_mbps": throughput_mbps,
            "tests_passed": 2,
            "errors": []
        });

        std::fs::write(output_file, serde_json::to_string_pretty(&results).unwrap())
            .map_err(|e| Failed::from(format!("Failed to write results: {}", e)))?;

        log::info!("✓ Results written to {}", output_file);
    }

    Ok(())
}
