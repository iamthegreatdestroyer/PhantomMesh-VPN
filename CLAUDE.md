# PhantomMesh VPN — v2.0 Rebuild

## Project Identity
- **Repo:** `iamthegreatdestroyer/PhantomMesh-VPN`
- **Language:** Rust (VPN core + crypto) + Python (agent orchestration)
- **Castle Layer:** Layer 3 — Security
- **Current version:** v1.0.0 (prototype, tunnel not functional)
- **Target version:** v2.0.0 (production VPN with real tunneling)
- **Replaces:** NordVPN, Tailscale, WireGuard GUI clients

## v2.0 Rebuild Status (2026-06-21)

The v1.0 audit revealed the tunnel engine is non-functional:
- No TUN/TAP device integration (packets logged, not forwarded)
- Nonce reuse (zero nonce for all packets — CRITICAL)
- No proper key exchange (pre-shared key used as "demo")
- No replay protection, no forward secrecy
- Kyber/Dilithium crypto available but not wired into handshake

### What WORKS (Keep):
- CryptoManager (Kyber-768, Dilithium-2, ChaCha20-Poly1305, BLAKE3)
- MeshHealer (peer reconnection with exponential backoff)
- AgentTunnelNegotiator (peer selection by latency)
- ThreatEngine (signature + anomaly detection)
- ApiGateway (Axum HTTP API with Prometheus metrics)
- Docker stack (8 services, Grafana, Loki, Prometheus)
- Agent framework (APEX, CIPHER, FORTRESS coordinators)

### What NEEDS REWRITING:
- tunnel_engine.rs — Complete rewrite with TUN/TAP + Noise handshake
- routing_manager.rs — Complete rewrite with packet forwarding
- main.rs — Wire up config loading + tunnel startup
- Config system — TOML config file parsing

---

## Rebuild Sprint Plan

### Sprint 1: Core Tunnel Engine (CRITICAL)
- [ ] TUN/TAP device creation via `tun-tap` crate on Linux
- [ ] Proper Noise_IK handshake using x25519 + Kyber-768 hybrid
- [ ] Nonce counter (incrementing u64, never reusing)
- [ ] Anti-replay window (sliding bitmap, 2000 packets)
- [ ] Packet encrypt/decrypt with ChaCha20-Poly1305
- [ ] Bidirectional packet forwarding (TUN ↔ UDP)

### Sprint 2: Configuration + CLI
- [ ] TOML config file (`/etc/phantommesh/config.toml`)
- [ ] Peer configuration (public keys, endpoints, allowed IPs)
- [ ] CLI tool (`phantommesh up`, `status`, `add-peer`, `down`)
- [ ] Key generation (`phantommesh genkey`, `pubkey`)
- [ ] DNS configuration for tunnel

### Sprint 3: Network Security
- [ ] Kill switch (iptables/nftables rules)
- [ ] DNS leak prevention (force DNS through tunnel)
- [ ] Split tunneling (AllowedIPs routing)
- [ ] NAT traversal (STUN/UDP hole punching)
- [ ] Forward secrecy via ephemeral keys

### Sprint 4: Mesh Networking
- [ ] Peer discovery (ICE-like negotiation)
- [ ] Multi-hop routing
- [ ] Wire existing MeshHealer into real tunnel
- [ ] Wire ThreatEngine into packet inspection

### Sprint 5: Integration + Polish
- [ ] Systemd service files
- [ ] Grafana dashboard for VPN metrics
- [ ] Wire agent framework into tunnel events
- [ ] Desktop client (Tauri) updates
- [ ] v2.0.0 tag

## Done Criteria
- [ ] Two peers can establish encrypted tunnel with Kyber hybrid handshake
- [ ] IP traffic flows through TUN device bidirectionally
- [ ] Nonce never reused, replay protection active
- [ ] Config loaded from TOML file
- [ ] CLI can bring tunnel up/down and show status
- [ ] Kill switch prevents traffic leaks
- [ ] Works on Debian 13 (sigma-pipeline target)

## Completion Signal
```bash
git tag v2.0.0 && git push origin v2.0.0
```
