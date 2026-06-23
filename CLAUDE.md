# PhantomMesh VPN v2.0 — Autonomous Completion Brief

## Project Location
- Rust workspace: `phantom-mesh-vpn/`
- Main binary: `src/bin/cli.rs` (phantommesh CLI)
- Tunnel engine: `src/vpn_core/tunnel_engine.rs`
- Handshake: `src/security_layer/handshake.rs` (WORKING — 5/5 tests pass)
- Mesh healer: `src/mesh/healer.rs` (WIRED — 8/8 tests pass)
- Threat engine: `src/security_layer/threat_engine.rs` (WIRED — integrated into tunnel decrypt path)
- Config: `src/vpn_core/config.rs`

## What Already Works (Do NOT break these)
- `cargo test --lib security_layer::handshake` — 5 tests MUST pass
- `cargo test --lib vpn_core::tunnel_engine` — replay window + peer tests
- `cargo test --lib mesh::healer` — existing healer tests
- `cargo build --release --bin phantommesh` — MUST compile
- TUN/TAP device creation on Linux
- Bidirectional packet forwarding (UDP <-> TUN)
- Nonce counter with anti-replay window
- TOML config file loading
- CLI: up/down/status/genkey/pubkey/config
- Kill switch via iptables
- DNS leak prevention

## Security Rules (NON-NEGOTIABLE)
- Private keys NEVER logged
- Nonce NEVER reused (incrementing u64 counter)
- Zero nonce (0) is reserved/rejected
- ChaCha20-Poly1305 for all transport encryption
- BLAKE3 for all key derivation

## Sprint 4: Wire Mesh Healer + Threat Engine

### Goal
Integrate the existing MeshHealer and ThreatEngine into the live tunnel engine
so that peer disconnections are automatically detected and reconnected, and
incoming packets are inspected for threats.

### Tasks
1. In tunnel_engine.rs, when a peer fails to respond to 3 consecutive keepalives,
   emit a PeerDisconnected event and invoke MeshHealer to attempt reconnection
2. Add a keepalive task that sends PACKET_KEEPALIVE to all peers every 25 seconds
3. Wire ThreatEngine::analyze_packet() into the decrypt path — after decryption
   but before writing to TUN, pass the packet through threat analysis
4. If ThreatEngine flags a packet, log a warning and emit ThreatSignature event
   (do NOT drop the packet — detection only, not blocking)
5. Add integration test: start tunnel, simulate peer timeout, verify healer reconnects
6. All existing tests must still pass

### Done Criteria
- [x] Keepalive packets sent every 25 seconds to all peers
- [x] Peer timeout detected after 75 seconds (3 missed keepalives)
- [x] MeshHealer invoked on peer timeout
- [x] ThreatEngine inspects decrypted packets
- [x] Threat events emitted for flagged packets
- [x] `cargo test` — all tests pass
- [x] `cargo build --release --bin phantommesh` — compiles

## Sprint 5: Polish + v2.0.0

### Goal
Clean up warnings, add missing tests, update version to 2.0.0, ensure everything
compiles cleanly.

### Tasks
1. Fix all compiler warnings (unused imports, dead code)
2. Update Cargo.toml version to "2.0.0"
3. Add test for keepalive send/receive
4. Add test for threat detection in tunnel path
5. Update main.rs to wire handshake into tunnel startup
6. Run `cargo clippy` and fix all warnings
7. Run `cargo test` — all tests pass
8. Run `cargo build --release` — clean build, no warnings
9. Tag v2.0.0

### Done Criteria
- [x] Zero compiler warnings
- [x] Zero clippy warnings
- [x] All tests pass
- [x] Version in Cargo.toml = "2.0.0"
- [x] `cargo build --release` succeeds cleanly
- [x] Both binaries compile: phantommesh + phantom-node

## Build Commands
```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd phantom-mesh-vpn
cargo test          # Run all tests
cargo build --release --bin phantommesh  # Build CLI
cargo clippy        # Lint check
```

## Completion Signal
```bash
git add -A && git commit -m "PhantomMesh v2.0.0 — production VPN with mesh healing + threat detection" && git tag v2.0.0
```
