# PhantomMesh VPN

> **Trans-dimensional agent-swarm orchestrated privacy fortress**
> Copyright © 2025 Stephen Bilodeau. All rights reserved.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue)](https://www.python.org/)

## Overview

PhantomMesh VPN is a revolutionary privacy platform that combines:

- **Quantum-resistant cryptography** (Kyber/CRYSTALS)
- **Semantic routing** with mesh topology optimization
- **Emergent threat intelligence** through Elite Agent collaboration
- **ΣVault dimensional scattering** for traffic obfuscation

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          PhantomMesh VPN                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────┐     ┌───────────────────┐     ┌───────────────┐ │
│  │   VPN Core        │     │   Agent Swarm     │     │  Observability │ │
│  │   (Rust)          │◄───►│   (Python)        │◄───►│  (Prometheus)  │ │
│  ├───────────────────┤     ├───────────────────┤     ├───────────────┤ │
│  │ • Tunnel Engine   │     │ • Orchestrator    │     │ • Metrics      │ │
│  │ • Crypto Manager  │     │ • Elite Agents    │     │ • Tracing      │ │
│  │ • Route Manager   │     │ • MNEMONIC Cache  │     │ • Alerting     │ │
│  │ • API Gateway     │     │ • VPN Hooks       │     │ • Dashboards   │ │
│  └───────────────────┘     └───────────────────┘     └───────────────┘ │
│           │                         │                        │          │
│           └─────────────────────────┼────────────────────────┘          │
│                                     │                                   │
│                          ┌──────────▼──────────┐                       │
│                          │   ΣVault Layer      │                       │
│                          │   (Sigma Integration)│                      │
│                          └─────────────────────┘                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Features

### Core Capabilities

- **WireGuard Protocol**: Industry-standard VPN tunneling
- **Post-Quantum Crypto**: Kyber-1024 key exchange, Dilithium-3 signatures
- **Mesh Networking**: Decentralized peer-to-peer routing
- **Agent Intelligence**: Autonomous threat detection and response
- **Dimensional Scattering**: ΣVault traffic obfuscation across 7 dimensions

### Elite Agent Swarm

| Agent      | Role               | Function                 |
| ---------- | ------------------ | ------------------------ |
| APEX       | Strategic Command  | Cross-agent coordination |
| PHANTOM    | Stealth Operations | Dynamic cloaking         |
| CIPHER     | Cryptographic Ops  | Key management           |
| VELOCITY   | Performance        | Route optimization       |
| FORTRESS   | Threat Detection   | Anomaly detection        |
| GENESIS    | Evolution          | Adaptive responses       |
| AEGIS      | Security Scanning  | Vulnerability assessment |
| NEXUS      | Integration        | CI/CD orchestration      |
| STREAM     | Traffic Analysis   | Flow control             |
| OMNISCIENT | Global Awareness   | State monitoring         |

## Quick Start

### Prerequisites

- Rust 1.70+
- Python 3.11+
- Docker & Docker Compose
- Linux/macOS (Windows via WSL2)

### Installation

```bash
# Clone repository
git clone https://github.com/phantommesh/phantom-mesh-vpn.git
cd phantom-mesh-vpn

# Build Rust core
cargo build --release

# Install Python agents
pip install -e ".[dev]"

# Start with Docker Compose
docker-compose up -d
```

### Basic Usage

```bash
# Start primary node
cargo run --bin phantom-node -- --config configs/mesh_configs.yaml

# In another terminal, start agent swarm
python -m src.agent_swarm.phantom_orchestrator
```

## Configuration

### Node Profiles

Configure different deployment scenarios in `configs/node_profiles.yaml`:

```yaml
profiles:
  primary:
    capabilities: [gateway, routing, key_exchange, threat_detection]
    resource_limits:
      max_peers: 1000
      max_bandwidth_mbps: 10000
  edge:
    capabilities: [routing]
    resource_limits:
      max_peers: 100
      max_bandwidth_mbps: 1000
```

### Mesh Configuration

Network settings in `configs/mesh_configs.yaml`:

```yaml
mesh:
  name: "PhantomMesh-Primary"
  protocol:
    wireguard:
      persistent_keepalive: 25
    sigma_vault:
      enabled: true
      dimensions: 7
```

## Development

### Development Environment

Use the provided devcontainer for consistent development:

```bash
# Open in VS Code
code .
# Use Command Palette: Dev Containers: Reopen in Container
```

### Testing

```bash
# Rust tests
cargo test

# Python tests
pytest tests/ -v --cov=src/agent_swarm

# Integration tests
docker-compose -f docker-compose.test.yml up --abort-on-container-exit
```

### Building

```bash
# Release build
cargo build --release

# Python wheel
maturin build --release

# Docker images
docker build -f Dockerfile.node -t phantom-mesh-vpn:latest .
docker build -f Dockerfile.agents -t phantom-mesh-agents:latest .
```

## API Documentation

### REST API

The VPN core exposes a REST API on port 8080:

```bash
# Get node status
curl http://localhost:8080/api/v1/status

# Add peer
curl -X POST http://localhost:8080/api/v1/peers \
  -H "Content-Type: application/json" \
  -d '{"public_key": "...", "endpoint": "1.2.3.4:51820"}'

# Get routes
curl http://localhost:8080/api/v1/routes
```

### Agent API

Agent swarm exposes additional endpoints:

```bash
# Get agent status
curl http://localhost:8080/api/v1/agents/status

# Trigger threat scan
curl -X POST http://localhost:8080/api/v1/agents/fortress/scan

# Get mnemonic cache stats
curl http://localhost:8080/api/v1/agents/mnemonic/stats
```

## Security

### Cryptographic Security

- **Key Exchange**: Kyber-1024 (post-quantum)
- **Signatures**: Dilithium-3 (post-quantum)
- **Symmetric**: ChaCha20-Poly1305
- **Hash**: BLAKE3

### Threat Intelligence

- Real-time anomaly detection
- Signature-based threat recognition
- Agent swarm collaborative defense
- Automatic quarantine and response

### Compliance

- SOC 2 Type II compliant architecture
- GDPR privacy by design
- HIPAA-ready for healthcare deployments

## Performance

| Metric            | Target  | Notes             |
| ----------------- | ------- | ----------------- |
| Handshake latency | < 100ms | WireGuard + Kyber |
| Packet encryption | < 1ms   | ChaCha20-Poly1305 |
| Route decision    | < 10ms  | Semantic routing  |
| Agent response    | < 50ms  | MNEMONIC cache    |
| Mesh convergence  | < 5s    | Gossip protocol   |

## Deployment

### Docker Compose

```bash
# Development
docker-compose up

# Production
docker-compose -f docker-compose.prod.yml up -d
```

### Kubernetes

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# Check status
kubectl get pods -l app=phantom-mesh
```

### Multi-Region

```bash
# Deploy primary region
kubectl apply -f k8s/primary-region/

# Deploy secondary region
kubectl apply -f k8s/secondary-region/
```

## Monitoring

### Metrics

Prometheus metrics available at `http://localhost:9090`:

- VPN tunnel statistics
- Agent performance metrics
- Threat detection events
- Route optimization success rates

### Dashboards

Grafana dashboards at `http://localhost:3000`:

- Network topology visualization
- Agent swarm activity
- Security incident timeline
- Performance monitoring

### Logging

Structured JSON logging with configurable levels:

```bash
# Debug logging
RUST_LOG=debug cargo run

# Structured output
docker-compose logs -f phantom-node | jq .
```

## Contributing

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run CI pipeline
5. Submit pull request

### Code Standards

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Python**: Follow PEP 8, use `ruff` and `mypy`
- **Documentation**: All public APIs documented
- **Testing**: 90%+ coverage required

### Commit Messages

```
feat: add semantic routing engine
fix: resolve memory leak in tunnel engine
docs: update API documentation
test: add integration tests for agent swarm
```

## License

### Dual License

**GPL-3.0** for personal and open-source use
**Commercial License** for production deployments

See [LICENSE](LICENSE) and [IP Protections](docs/ip_protections.md) for details.

## Roadmap

### Phase 1 (Current): Core Scaffold

- [x] Repository structure
- [x] Basic VPN functionality
- [x] Agent swarm framework
- [ ] CI/CD pipeline

### Phase 2: Intelligence Activation

- [ ] Full agent swarm implementation
- [ ] Threat detection engine
- [ ] ΣVault dimensional scattering

### Phase 3: Production Hardening

- [ ] Performance optimization
- [ ] Security auditing
- [ ] Enterprise features

### Phase 4: Ecosystem Expansion

- [ ] Multi-region federation
- [ ] Third-party integrations
- [ ] Mobile clients

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/phantommesh/phantom-mesh-vpn/issues)
- **Discussions**: [GitHub Discussions](https://github.com/phantommesh/phantom-mesh-vpn/discussions)

## Acknowledgments

- WireGuard protocol implementation
- Post-quantum cryptography research community
- Open-source Rust and Python ecosystems

---

**PhantomMesh VPN** — _Privacy through intelligence, security through mathematics._
