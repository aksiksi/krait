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
    #[clap(short, long, default_value = "ens18")]
    iface: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    env_logger::init();

    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/krait"
    )))?;
    match aya_log::EbpfLogger::init(&mut ebpf) {
        Err(e) => {
            // This can happen if you remove all log statements from your eBPF program.
            log::warn!("failed to initialize eBPF logger: {e}");
        }
        Ok(logger) => {
            let mut logger =
                tokio::io::unix::AsyncFd::with_interest(logger, tokio::io::Interest::READABLE)?;
            tokio::task::spawn(async move {
                loop {
                    let mut guard = logger.readable_mut().await.unwrap();
                    guard.get_inner_mut().flush();
                    guard.clear_ready();
                }
            });
        }
    }

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
    let ingress: &mut SchedClassifier = ebpf.program_mut("krait_ingress").unwrap().try_into()?;
    ingress.load()?;
    ingress.attach(&opt.iface, TcAttachType::Ingress)?;

    // Load and attach egress
    let egress: &mut SchedClassifier = ebpf.program_mut("krait_egress").unwrap().try_into()?;
    egress.load()?;
    egress.attach(&opt.iface, TcAttachType::Egress)?;

    let ctrl_c = tokio::signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    Ok(())
}
