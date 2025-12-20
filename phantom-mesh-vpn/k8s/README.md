# PhantomMesh Kubernetes Deployment

This directory contains comprehensive Kubernetes manifests for deploying PhantomMesh VPN with agent swarm orchestration to production environments.

## Directory Structure

```
k8s/
├── base/                          # Base Kubernetes manifests
│   ├── autoscaling/              # Horizontal Pod Autoscalers
│   ├── configmaps/               # Configuration data
│   ├── deployments/              # Application deployments
│   ├── ingress/                  # Ingress controllers
│   ├── namespace/                # Namespace definitions
│   ├── networkpolicies/          # Network security policies
│   ├── persistentvolumes/        # PVC definitions
│   ├── rbac/                     # Service accounts and RBAC
│   └── services/                 # Service definitions
├── overlays/                     # Environment-specific overlays
│   ├── dev/                      # Development overrides
│   ├── prod/                     # Production overrides
│   └── multi-region/             # Multi-region deployment
├── helm/                         # Helm chart for easy deployment
│   └── phantommesh/
│       ├── Chart.yaml
│       ├── values.yaml
│       └── templates/
├── istio/                        # Istio service mesh configuration
└── linkerd/                      # Linkerd service mesh configuration
```

## Quick Start

### Using kubectl

1. Create the namespace:

```bash
kubectl apply -f k8s/base/namespace/
```

2. Apply base manifests:

```bash
kubectl apply -k k8s/base/
```

3. For production environment:

```bash
kubectl apply -k k8s/overlays/prod/
```

### Using Helm

1. Add the Helm repository (if hosted):

```bash
helm repo add phantommesh https://charts.phantommesh.com
helm repo update
```

2. Install the chart:

```bash
helm install phantommesh ./k8s/helm/phantommesh
```

3. For custom values:

```bash
helm install phantommesh ./k8s/helm/phantommesh -f values-prod.yaml
```

## Components

### Core Services

- **phantom-node**: Rust-based VPN core with WireGuard
- **agent-swarm**: Python-based autonomous agent orchestrator
- **discovery**: Service discovery for dynamic agents
- **prometheus**: Metrics collection and monitoring
- **grafana**: Dashboard and visualization

### Security Features

- **Network Policies**: Default deny with specific allow rules
- **RBAC**: Service accounts with minimal required permissions
- **Pod Security Standards**: Restricted security context
- **Secrets Management**: Encrypted secrets for sensitive data

### Scalability

- **Horizontal Pod Autoscaling**: CPU/memory-based scaling
- **Rolling Updates**: Zero-downtime deployments
- **Multi-region Support**: Cross-region deployments
- **Load Balancing**: Service mesh integration

## Service Mesh Integration

### Istio

Apply Istio configuration:

```bash
kubectl apply -f k8s/istio/
```

Features:

- Mutual TLS encryption
- Traffic management and routing
- Observability and tracing
- Security policies

### Linkerd

Apply Linkerd configuration:

```bash
kubectl apply -f k8s/linkerd/
```

Features:

- Automatic mTLS
- HTTP/gRPC load balancing
- Request-level routing
- Service profiles

## Multi-Region Deployment

For multi-region deployments:

1. Create separate namespaces for each region
2. Apply region-specific overlays
3. Configure global load balancer
4. Set up cross-region networking

```bash
kubectl apply -k k8s/overlays/multi-region/
```

## Monitoring and Observability

### Prometheus Metrics

The deployment includes comprehensive monitoring:

- Application metrics from all services
- Kubernetes cluster metrics
- Custom PhantomMesh-specific metrics
- Alert rules for critical events

### Grafana Dashboards

Pre-configured dashboards for:

- VPN tunnel status and performance
- Agent swarm activity
- System resource usage
- Network traffic analysis

Access Grafana at: `https://grafana.phantom-mesh.example.com`

## Backup and Recovery

### Data Persistence

- PVCs for stateful data
- ConfigMaps for configuration
- Secrets for sensitive data

### Backup Strategy

1. **Automated Backups**: Use Velero for cluster backups
2. **PVC Snapshots**: Regular snapshots of persistent volumes
3. **Config Backup**: GitOps for configuration versioning

### Disaster Recovery

1. **RTO/RPO**: 4-hour RTO, 1-hour RPO
2. **Failover**: Automatic failover to secondary regions
3. **Data Recovery**: Point-in-time recovery from backups

## Troubleshooting

### Common Issues

1. **Pod Security Violations**:

   - Check pod security standards
   - Update security contexts

2. **Network Connectivity**:

   - Verify network policies
   - Check service mesh configuration

3. **Resource Constraints**:
   - Monitor HPA events
   - Adjust resource limits

### Logs and Debugging

```bash
# View pod logs
kubectl logs -f deployment/phantom-node -n phantom-mesh

# Debug network issues
kubectl exec -it deployment/phantom-node -n phantom-mesh -- /bin/bash

# Check service mesh
linkerd check
istioctl proxy-status
```

## Security Hardening

### Production Checklist

- [ ] Enable pod security standards
- [ ] Configure network policies
- [ ] Set up RBAC with least privilege
- [ ] Enable service mesh mTLS
- [ ] Configure secrets management
- [ ] Enable audit logging
- [ ] Set up vulnerability scanning
- [ ] Configure backup encryption

### Compliance

- SOC 2 Type II compliant
- GDPR compliant data handling
- NIST cybersecurity framework
- CIS Kubernetes benchmarks

## Performance Tuning

### Resource Optimization

- CPU requests: Based on average usage
- Memory limits: Based on peak usage + 20%
- Storage: SSD-backed for performance

### Scaling Guidelines

- **Development**: 1 replica per service
- **Staging**: 2-3 replicas per service
- **Production**: 3+ replicas with HPA

### Network Performance

- Service mesh for efficient routing
- UDP optimization for WireGuard
- Connection pooling and keep-alive

## Contributing

When making changes to the Kubernetes manifests:

1. Update base manifests first
2. Test in development environment
3. Update Helm chart templates
4. Update documentation
5. Test in staging environment

## Support

For issues and questions:

- Check the troubleshooting guide
- Review logs and metrics
- Contact the DevOps team
- Create GitHub issues for bugs
