use anyhow::Result;
use aya::programs::{SchedClassifier, TcAttachType, tc};
use aya_log::EbpfLogger;

pub struct KraitAgent {
    ebpf: aya::Ebpf,
    logger: EbpfLogger<&'static dyn log::Log>,
    interface: String,
}

impl KraitAgent {
    /// Create a new Krait agent for the specified interface
    pub fn new(interface: &str) -> Result<Self> {
        // Load eBPF program
        let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
            env!("OUT_DIR"),
            "/krait"
        )))?;

        // Initialize eBPF logger
        let logger = EbpfLogger::init(&mut ebpf)?;

        Ok(Self {
            ebpf,
            logger,
            interface: interface.to_string(),
        })
    }

    /// Setup and attach eBPF programs to the interface
    pub fn attach(&mut self) -> Result<()> {
        // Create clsact qdisc (required for TC attach)
        Self::reset_tc(&self.interface)?;
        if let Err(e) = tc::qdisc_add_clsact(&self.interface) {
            // EEXIST is fine, anything else is a real error
            match e {
                tc::TcError::AlreadyAttached => (),
                _ => return Err(e.into()),
            }
        }

        // Load and attach ingress
        log::info!("Loading ingress eBPF program");
        let ingress: &mut SchedClassifier =
            self.ebpf.program_mut("krait_ingress").unwrap().try_into()?;
        ingress.load()?;
        ingress.attach(&self.interface, TcAttachType::Ingress)?;
        log::info!("Ingress eBPF program attached to {}", self.interface);

        // Load and attach egress
        log::info!("Loading egress eBPF program");
        let egress: &mut SchedClassifier =
            self.ebpf.program_mut("krait_egress").unwrap().try_into()?;
        egress.load()?;
        egress.attach(&self.interface, TcAttachType::Egress)?;
        log::info!("Egress eBPF program attached to {}", self.interface);

        Ok(())
    }

    /// Poll eBPF logger for new log messages
    pub fn poll_logs(&mut self) {
        self.logger.flush();
    }

    /// Get access to the underlying eBPF object for map access
    pub fn ebpf(&self) -> &aya::Ebpf {
        &self.ebpf
    }

    /// Get mutable access to the underlying eBPF object for map access
    pub fn ebpf_mut(&mut self) -> &mut aya::Ebpf {
        &mut self.ebpf
    }

    fn reset_tc(iface: &str) -> Result<()> {
        std::process::Command::new("tc")
            .args(["qdisc", "del", "dev", iface, "clsact"])
            .output()?; // ignore errors
        Ok(())
    }
}

impl Drop for KraitAgent {
    fn drop(&mut self) {
        log::info!("Detaching eBPF programs from {}", self.interface);
        // Programs will be automatically detached when the Ebpf object is dropped
    }
}
