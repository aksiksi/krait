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
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr, Ipv6Hdr},
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
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?;
    match unsafe { *ethhdr }.ether_type() {
        Ok(EtherType::Ipv4) => {
            let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN)?;
            let source_addr = unsafe { (*ipv4hdr).src_addr() };

            let source_port = match unsafe { (*ipv4hdr).proto } {
                IpProto::Tcp => {
                    let tcphdr: *const TcpHdr =
                        ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
                    u16::from_be_bytes(unsafe { (*tcphdr).source })
                }
                IpProto::Udp => {
                    let udphdr: *const UdpHdr =
                        ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
                    unsafe { (*udphdr).src_port() }
                }
                _ => return Ok(TC_ACT_OK),
            };

            info!(&ctx, "{} - SRC IP: {:i}, SRC PORT: {}", direction.as_str(), source_addr, source_port);
        }
        Ok(EtherType::Ipv6) => {
            let ipv6hdr: *const Ipv6Hdr = ptr_at(&ctx, EthHdr::LEN)?;
            let source_addr = unsafe { (*ipv6hdr).src_addr() };

            let source_port = match unsafe { (*ipv6hdr).next_hdr } {
                IpProto::Tcp => {
                    let tcphdr: *const TcpHdr =
                        ptr_at(&ctx, EthHdr::LEN  + Ipv6Hdr::LEN)?;
                    u16::from_be_bytes(unsafe { (*tcphdr).source })
                }
                IpProto::Udp => {
                    let udphdr: *const UdpHdr =
                        ptr_at(&ctx, EthHdr::LEN + Ipv6Hdr::LEN)?;
                    unsafe { (*udphdr).src_port() }
                }
                _ => return Ok(TC_ACT_OK),
            };

            info!(&ctx, "{} - SRC IP: {:i}, SRC PORT: {}", direction.as_str(), source_addr, source_port);
        }
        _ => {},
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
