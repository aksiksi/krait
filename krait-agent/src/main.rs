use aya::programs::{tc, SchedClassifier, TcAttachType};
use clap::Parser;

fn reset_tc(iface: &str) -> anyhow::Result<()> {
    std::process::Command::new("tc")
        .args(["qdisc", "del", "dev", iface, "clsact"])
        .output()?; // ignore errors
    Ok(())
}

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "wg0")]
    iface: String,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    env_logger::init();
    
    log::info!("Starting krait agent on interface: {}", opt.iface);

    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/krait"
    )))?;

    let mut logger = aya_log::EbpfLogger::init(&mut ebpf)?;

    // Create clsact qdisc (required for TC attach)
    reset_tc(&opt.iface)?;
    if let Err(e) = tc::qdisc_add_clsact(&opt.iface) {
        // EEXIST is fine, anything else is a real error
        match e {
            tc::TcError::AlreadyAttached => (),
            _ => return Err(e.into()),
        }
    }

    // Load and attach ingress
    log::info!("Loading ingress eBPF program");
    let ingress: &mut SchedClassifier = ebpf.program_mut("krait_ingress").unwrap().try_into()?;
    ingress.load()?;
    ingress.attach(&opt.iface, TcAttachType::Ingress)?;
    log::info!("Ingress eBPF program attached to {}", opt.iface);

    // Load and attach egress
    log::info!("Loading egress eBPF program");
    let egress: &mut SchedClassifier = ebpf.program_mut("krait_egress").unwrap().try_into()?;
    egress.load()?;
    egress.attach(&opt.iface, TcAttachType::Egress)?;
    log::info!("Egress eBPF program attached to {}", opt.iface);

    log::info!("eBPF programs loaded successfully, waiting for termination signal");
    
    // Handle Ctrl+C signal without tokio
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })?;
    
    // Poll eBPF logger in a loop until signal received
    loop {
        match rx.try_recv() {
            Ok(_) => {
                log::info!("Received termination signal");
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Poll eBPF logs
                logger.flush();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        }
    }

    Ok(())
}
