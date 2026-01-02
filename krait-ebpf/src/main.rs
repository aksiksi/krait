#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::TC_ACT_OK,
    macros::{classifier, map},
    maps::HashMap,
    programs::TcContext,
};
use aya_log_ebpf::info;
use network_types::{
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

/// peer_index → identity_tag
#[map]
static PEER_IDENTITY: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

/// identity_tag → metadata (for later: counters, policy hints)
#[map]
static IDENTITY_META: HashMap<u32, u64> = HashMap::with_max_entries(1024, 0);

enum Direction {
    Ingress,
    Egress,
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::Ingress => "ingress",
            Direction::Egress => "egress",
        }
    }
}

#[classifier]
pub fn krait_egress(ctx: TcContext) -> i32 {
    match try_krait(ctx, Direction::Egress) {
        Ok(ret) => ret,
        Err(_) => TC_ACT_OK,
    }
}

#[classifier]
pub fn krait_ingress(ctx: TcContext) -> i32 {
    match try_krait(ctx, Direction::Ingress) {
        Ok(ret) => ret,
        Err(_) => TC_ACT_OK,
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: &TcContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = core::mem::size_of::<T>();
    if start + offset + len > end {
        return Err(());
    }
    Ok((start + offset) as *const T)
}

#[inline(always)]
fn try_krait(ctx: TcContext, direction: Direction) -> Result<i32, ()> {
    info!(&ctx, "krait: packet received on {} direction", direction.as_str());
    
    // wg0 is a TUN device - no Ethernet header, start directly with IP
    let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, 0)?;
    
    // Check if this looks like an IPv4 packet (basic validation)
    let version = unsafe { (*ipv4hdr).version() };
    if version != 4 {
        info!(&ctx, "krait: not IPv4 packet, version = {}", version);
        return Ok(TC_ACT_OK);
    }
    
    let source_addr = unsafe { (*ipv4hdr).src_addr() };
    let dest_addr = unsafe { (*ipv4hdr).dst_addr() };
    let protocol = unsafe { (*ipv4hdr).proto };
    
    info!(&ctx, "krait: {} packet - SRC: {:i} -> DST: {:i}, proto: {}", 
          direction.as_str(), source_addr, dest_addr, protocol as u8);

    // Parse L4 information if TCP/UDP
    match protocol {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr = ptr_at(&ctx, Ipv4Hdr::LEN)?;
            let src_port = unsafe { u16::from_be_bytes((*tcphdr).source) };
            let dst_port = unsafe { u16::from_be_bytes((*tcphdr).dest) };
            info!(&ctx, "krait: TCP {}:{} -> {}:{}", 
                  source_addr, src_port, dest_addr, dst_port);
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr = ptr_at(&ctx, Ipv4Hdr::LEN)?;
            let src_port = unsafe { (*udphdr).src_port() };
            let dst_port = unsafe { (*udphdr).dst_port() };
            info!(&ctx, "krait: UDP {}:{} -> {}:{}", 
                  source_addr, src_port, dest_addr, dst_port);
        }
        IpProto::Icmp => {
            info!(&ctx, "krait: ICMP packet");
        }
        _ => {
            info!(&ctx, "krait: other protocol: {}", protocol as u8);
        }
    }

    Ok(TC_ACT_OK)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
