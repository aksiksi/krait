use clap::Parser;
use krait_agent::KraitAgent;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "wg0")]
    iface: String,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    env_logger::init();

    log::info!("Starting krait agent on interface: {}", opt.iface);

    // Create and attach the agent
    let mut agent = KraitAgent::new(&opt.iface)?;
    agent.attach()?;

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
                agent.poll_logs();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        }
    }

    Ok(())
}
