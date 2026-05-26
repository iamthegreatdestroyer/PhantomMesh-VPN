//! Agent-based Tunnel Negotiation
//!
//! Implements capability advertisement, peer selection, and tunnel parameter
//! negotiation between mesh nodes. Integrates with the WireGuard-based
//! TunnelEngine without exposing or logging any key material.

use tracing::info;
use uuid::Uuid;

/// VPN protocol variants a node may support
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Protocol {
    WireGuard,
    KyberHybrid,
}

/// Capabilities advertised by a mesh node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeCapabilities {
    pub peer_id: String,
    pub bandwidth_mbps: u32,
    pub latency_ms: u32,
    pub protocols: Vec<Protocol>,
}

/// Parameters agreed upon after tunnel negotiation
#[derive(Debug, Clone)]
pub struct TunnelParams {
    pub peer_id: String,
    pub protocol: Protocol,
    pub mtu: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum NegotiationError {
    #[error("no compatible peers available")]
    NoPeers,
    #[error("no shared protocol with peer {0}")]
    NoSharedProtocol(String),
}

/// Negotiates tunnels between mesh peers using capability matching.
pub struct AgentTunnelNegotiator {
    node_id: String,
    bandwidth_mbps: u32,
    latency_ms: u32,
    supported_protocols: Vec<Protocol>,
}

impl AgentTunnelNegotiator {
    pub fn new(bandwidth_mbps: u32, latency_ms: u32, supported_protocols: Vec<Protocol>) -> Self {
        Self {
            node_id: Uuid::new_v4().to_string(),
            bandwidth_mbps,
            latency_ms,
            supported_protocols,
        }
    }

    /// Return this node's capability advertisement.
    pub fn advertise_capabilities(&self) -> NodeCapabilities {
        NodeCapabilities {
            peer_id: self.node_id.clone(),
            bandwidth_mbps: self.bandwidth_mbps,
            latency_ms: self.latency_ms,
            protocols: self.supported_protocols.clone(),
        }
    }

    /// Select the best peer from candidates: lowest latency among those that
    /// share at least one supported protocol with this node.
    pub fn select_peer<'a>(
        &self,
        candidates: &'a [NodeCapabilities],
    ) -> Result<&'a NodeCapabilities, NegotiationError> {
        candidates
            .iter()
            .filter(|c| c.protocols.iter().any(|p| self.supported_protocols.contains(p)))
            .min_by_key(|c| c.latency_ms)
            .ok_or(NegotiationError::NoPeers)
    }

    /// Negotiate tunnel parameters with the chosen peer.
    ///
    /// Picks the first mutually supported protocol (WireGuard preferred).
    /// Key material is never logged here or anywhere in this path.
    pub fn negotiate_tunnel(
        &self,
        peer: &NodeCapabilities,
    ) -> Result<TunnelParams, NegotiationError> {
        // Preferred order: WireGuard first, then KyberHybrid
        let preference = [Protocol::WireGuard, Protocol::KyberHybrid];
        let protocol = preference
            .iter()
            .find(|p| {
                self.supported_protocols.contains(p) && peer.protocols.contains(p)
            })
            .cloned()
            .ok_or_else(|| NegotiationError::NoSharedProtocol(peer.peer_id.clone()))?;

        info!(
            peer_id = %peer.peer_id,
            protocol = ?protocol,
            "tunnel negotiated"
        );

        Ok(TunnelParams {
            peer_id: peer.peer_id.clone(),
            protocol,
            mtu: 1420,
        })
    }

    /// Attempt negotiation with each candidate in order until one succeeds;
    /// falls back to direct connect (WireGuard, MTU 1280) if all fail.
    pub fn negotiate_with_fallback(
        &self,
        candidates: &[NodeCapabilities],
    ) -> TunnelParams {
        // Try best peer first
        if let Ok(peer) = self.select_peer(candidates) {
            if let Ok(params) = self.negotiate_tunnel(peer) {
                return params;
            }
        }

        // Try remaining peers
        for candidate in candidates {
            if let Ok(params) = self.negotiate_tunnel(candidate) {
                return params;
            }
        }

        // Direct-connect fallback
        info!("all peers failed negotiation, using direct-connect fallback");
        TunnelParams {
            peer_id: "direct".to_string(),
            protocol: Protocol::WireGuard,
            mtu: 1280,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_negotiator() -> AgentTunnelNegotiator {
        AgentTunnelNegotiator::new(
            1000,
            5,
            vec![Protocol::WireGuard, Protocol::KyberHybrid],
        )
    }

    #[test]
    fn test_advertise_capabilities() {
        let n = make_negotiator();
        let caps = n.advertise_capabilities();
        assert_eq!(caps.bandwidth_mbps, 1000);
        assert_eq!(caps.latency_ms, 5);
        assert!(caps.protocols.contains(&Protocol::WireGuard));
    }

    #[test]
    fn test_negotiate_with_peer() {
        let n = make_negotiator();
        let peer = NodeCapabilities {
            peer_id: "peer-a".to_string(),
            bandwidth_mbps: 500,
            latency_ms: 10,
            protocols: vec![Protocol::WireGuard],
        };
        let params = n.negotiate_tunnel(&peer).unwrap();
        assert_eq!(params.peer_id, "peer-a");
        assert_eq!(params.protocol, Protocol::WireGuard);
        assert_eq!(params.mtu, 1420);
    }

    #[test]
    fn test_no_shared_protocol_is_error() {
        let n = AgentTunnelNegotiator::new(100, 5, vec![Protocol::KyberHybrid]);
        let peer = NodeCapabilities {
            peer_id: "peer-b".to_string(),
            bandwidth_mbps: 100,
            latency_ms: 20,
            protocols: vec![Protocol::WireGuard],
        };
        assert!(matches!(
            n.negotiate_tunnel(&peer),
            Err(NegotiationError::NoSharedProtocol(_))
        ));
    }

    #[test]
    fn test_select_peer_picks_lowest_latency() {
        let n = make_negotiator();
        let candidates = vec![
            NodeCapabilities {
                peer_id: "fast".to_string(),
                bandwidth_mbps: 100,
                latency_ms: 5,
                protocols: vec![Protocol::WireGuard],
            },
            NodeCapabilities {
                peer_id: "slow".to_string(),
                bandwidth_mbps: 1000,
                latency_ms: 100,
                protocols: vec![Protocol::WireGuard],
            },
        ];
        let selected = n.select_peer(&candidates).unwrap();
        assert_eq!(selected.peer_id, "fast");
    }

    #[test]
    fn test_fallback_to_direct_connect() {
        // Negotiator only speaks KyberHybrid; candidates only speak WireGuard
        let n = AgentTunnelNegotiator::new(100, 5, vec![Protocol::KyberHybrid]);
        let candidates = vec![NodeCapabilities {
            peer_id: "wg-only".to_string(),
            bandwidth_mbps: 100,
            latency_ms: 10,
            protocols: vec![Protocol::WireGuard],
        }];
        let params = n.negotiate_with_fallback(&candidates);
        assert_eq!(params.peer_id, "direct");
        assert_eq!(params.protocol, Protocol::WireGuard);
        assert_eq!(params.mtu, 1280);
    }

    #[test]
    fn test_no_candidates_falls_back() {
        let n = make_negotiator();
        let params = n.negotiate_with_fallback(&[]);
        assert_eq!(params.peer_id, "direct");
    }
}
