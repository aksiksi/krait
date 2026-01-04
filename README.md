# krait

## Why Krait?

Krait's niche is **peer-to-peer mesh for non-Kubernetes Linux hosts** — bare metal, VMs, edge devices, multi-cloud links. If you're in Kubernetes, Cilium already solves identity-based networking better.

### Goals

1. **Pure kernel data path** — Zero userspace packet processing; TC eBPF → kernel WireGuard, no copies
2. **eBPF policy engine** — Identity-based allow/deny enforced per-packet in kernel, not iptables
3. **Microsecond policy updates** — Atomic map updates, no rule regeneration or conntrack flush
4. **L4 identity policy** — `(src_identity, dst_identity, port, proto)` granularity in data path
5. **eBPF-native observability** — Per-identity traffic counters, drops, exposed as Prometheus metrics without sidecars
6. **Lean agent** — Rust, passive after setup, near-zero CPU during data transfer
7. **O(1) policy scaling** — Map lookups regardless of mesh size (vs O(n) iptables rules)
8. **Mid-connection revocation** — Policy change takes effect on next packet, not next connection

### Non-Goals (Out of Scope)

1. **Cross-platform support** — Linux-only, no Windows/macOS/iOS/Android
2. **NAT traversal / STUN / DERP relays** — Assumes direct connectivity or external solution
3. **SSO / OIDC integration** — No built-in identity provider integration
4. **Admin console / UI** — CLI and config files only
5. **MagicDNS / service discovery** — No automatic DNS for peers
6. **Userspace fallback** — Requires kernel WireGuard, no wireguard-go fallback
7. **Managed SaaS** — Self-hosted coordinator only

### Target Users

- **Multi-cloud site-to-site** (VMs, not pods)
- **Edge / IoT fleets** (Linux devices outside K8s)
- **Homelab / self-hosters**
- **Hybrid infra** where some nodes aren't in K8s

## Krait vs. Cilium

| Use Case | Cilium | Krait |
|----------|--------|-------|
| K8s CNI (pod networking) | ✓ | ✗ |
| K8s Network Policy | ✓ | ✗ |
| Service mesh (L7) | ✓ | ✗ |
| Non-K8s hosts | Possible but awkward | ✓ |
| Site-to-site VPN | ✗ | ✓ |
| Peer-to-peer mesh | ✗ | ✓ |

