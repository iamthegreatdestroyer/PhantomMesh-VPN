//! Mock network layer for testing
//! 
//! Provides simulated network operations without actual socket I/O

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockNetwork {
    packets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    latency_ms: u64,
}

impl MockNetwork {
    pub fn new() -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
            latency_ms: 0,
        }
    }
    
    pub fn with_latency(latency_ms: u64) -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
            latency_ms,
        }
    }
    
    pub fn send(&self, dest: &str, data: &[u8]) -> Result<(), String> {
        let mut packets = self.packets.lock().unwrap();
        packets.insert(dest.to_string(), data.to_vec());
        Ok(())
    }
    
    pub fn receive(&self, src: &str) -> Option<Vec<u8>> {
        let packets = self.packets.lock().unwrap();
        packets.get(src).cloned()
    }
    
    pub fn clear(&self) {
        let mut packets = self.packets.lock().unwrap();
        packets.clear();
    }
}

impl Default for MockNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_send_receive() {
        let network = MockNetwork::new();
        network.send("peer1", b"hello").unwrap();
        let received = network.receive("peer1");
        assert_eq!(received, Some(b"hello".to_vec()));
    }
    
    #[test]
    fn test_mock_clear() {
        let network = MockNetwork::new();
        network.send("peer1", b"data").unwrap();
        network.clear();
        assert!(network.receive("peer1").is_none());
    }
}
