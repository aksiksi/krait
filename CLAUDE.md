## POC Implementation Plan

### Phase 0: Foundation (Week 1)

**Goal**: Minimal eBPF + kernel WG integration on a single node

1. Rust project scaffold with Aya
2. Load kernel WG module, create interface programmatically (`wg0`)
3. Attach TC eBPF program (ingress + egress) to `wg0`
4. eBPF program that passes all traffic (no-op) — verify packets flow

**Validation**: `iperf3` through WG tunnel, confirm eBPF program is hit via `bpf_printk`

---

### Phase 1: Identity Tagging (Week 2)

**Goal**: Attach identity to packets

1. Define identity model: `peer_pubkey → identity_tag` mapping
2. eBPF map: `BPF_MAP_TYPE_HASH` keyed by WG peer index (available in skb metadata) → identity tag
3. Rust agent populates map on peer registration
4. eBPF program reads identity, logs it

**Validation**: See identity tags in trace output for traffic between peers

---

### Phase 2: Policy Engine (Week 2-3)

**Goal**: Enforce allow/deny based on identity pairs

1. Policy map: `(src_identity, dst_identity)` → `action (allow/deny)`
2. Extend to L4: `(src_identity, dst_identity, dst_port, proto)` → `action`
3. eBPF parses IP + TCP/UDP headers, performs lookup, returns `TC_ACT_OK` or `TC_ACT_SHOT`
4. Rust agent exposes simple API/config to update policy map

**Validation**: Block traffic between two identities, confirm drops

---

### Phase 3: Observability (Week 3)

**Goal**: Per-identity metrics

1. Metrics map: `identity_tag` → `{ bytes_in, bytes_out, packets_in, packets_out, drops }`
2. eBPF increments atomically on each packet
3. Rust agent exposes `/metrics` endpoint (Prometheus format)

**Validation**: Grafana dashboard showing per-identity traffic

---

### Phase 4: Multi-Node (Week 4)

**Goal**: Centralized control plane, 3+ node mesh

1. Simple coordinator (can be SQLite + HTTP API initially)
2. Peer registration: node registers pubkey, receives identity tag + peer list
3. Rust agent syncs peer list, configures WG peers, populates eBPF maps
4. Policy distribution: coordinator pushes policy, agents apply to local maps

**Validation**: 3-node mesh with identity-based policy, metrics from all nodes

---

### Tech Stack

| Component | Choice |
|-----------|--------|
| eBPF framework | Aya (Rust) |
| WG management | `wireguard-rs` or shell out to `wg` |
| Coordinator | Axum + SQLite (simplest) |
| Agent-coordinator protocol | HTTP/JSON (upgrade to gRPC later) |
| Metrics | Prometheus exposition format |

---

### Deferred (Post-POC)

- NAT traversal / STUN / relay
- Key rotation
- Bandwidth quotas
- Stateful policy
- Proper auth between agent ↔ coordinator

