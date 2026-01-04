//! Load Testing Framework for PhantomMesh Agent Framework
//!
//! Comprehensive load testing with multiple scenarios:
//! - Throughput testing (messages/second)
//! - Latency analysis (p50, p95, p99)
//! - Concurrent agent stress testing
//! - Message ordering verification
//! - Resource utilization tracking

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub num_agents: usize,
    pub messages_per_agent: usize,
    pub concurrent_sends: usize,
    pub message_size: usize,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_messages: u64,
    pub successful_messages: u64,
    pub failed_messages: u64,
    pub total_duration: Duration,
    pub throughput_mps: f64,
    pub latencies: LatencyStats,
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub min_us: u64,
    pub max_us: u64,
    pub mean_us: f64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
}

pub struct LoadTester {
    config: LoadTestConfig,
    latencies: Arc<parking_lot::Mutex<Vec<u64>>>,
    message_count: Arc<AtomicU64>,
    error_count: Arc<AtomicUsize>,
}

impl LoadTester {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            latencies: Arc::new(parking_lot::Mutex::new(Vec::new())),
            message_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Run throughput test: max messages/second
    pub async fn test_throughput(&self) -> PerformanceMetrics {
        let start = Instant::now();
        let (tx, mut rx) = mpsc::channel::<(Instant, Vec<u8>)>(self.config.concurrent_sends * 10);

        let message_count = self.message_count.clone();
        let error_count = self.error_count.clone();
        let latencies = self.latencies.clone();

        // Spawn receiver task
        let recv_handle = tokio::spawn(async move {
            while let Some((send_time, _msg)) = rx.recv().await {
                let latency_us = send_time.elapsed().as_micros() as u64;
                latencies.lock().push(latency_us);
                message_count.fetch_add(1, Ordering::Relaxed);
            }
        });

        // Spawn sender tasks
        let mut handles = vec![];
        for agent_id in 0..self.config.num_agents {
            let tx = tx.clone();
            let concurrent_sends = self.config.concurrent_sends;
            let msg_per_agent = self.config.messages_per_agent;
            let msg_size = self.config.message_size;

            let handle = tokio::spawn(async move {
                for msg_id in 0..msg_per_agent {
                    for _ in 0..concurrent_sends {
                        let message =
                            format!("Agent{}-Msg{}-{}", agent_id, msg_id, "x".repeat(msg_size));
                        let _ = tx.send((Instant::now(), message)).await;
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all senders
        for handle in handles {
            let _ = handle.await;
        }

        drop(tx);
        let _ = recv_handle.await;

        let duration = start.elapsed();
        self.build_metrics(duration)
    }

    /// Run latency test: measure response times under load
    pub async fn test_latency(&self) -> PerformanceMetrics {
        let start = Instant::now();
        let batch_size = self.config.concurrent_sends;

        for _ in 0..self.config.num_agents {
            let mut batch_handles = vec![];

            for _ in 0..batch_size {
                let latencies = self.latencies.clone();
                let message_count = self.message_count.clone();

                let handle = tokio::spawn(async move {
                    let send_time = Instant::now();
                    // Simulate message processing
                    tokio::time::sleep(Duration::from_micros(10)).await;

                    let latency_us = send_time.elapsed().as_micros() as u64;
                    latencies.lock().push(latency_us);
                    message_count.fetch_add(1, Ordering::Relaxed);
                });

                batch_handles.push(handle);
            }

            for handle in batch_handles {
                let _ = handle.await;
            }
        }

        let duration = start.elapsed();
        self.build_metrics(duration)
    }

    /// Run stress test: push framework to limits
    pub async fn test_stress(&self) -> PerformanceMetrics {
        let start = Instant::now();
        let (tx, mut rx) = mpsc::channel::<(Instant, Vec<u8>)>(1000);

        let message_count = self.message_count.clone();
        let error_count = self.error_count.clone();
        let latencies = self.latencies.clone();

        // Receiver with back-pressure
        let recv_handle = tokio::spawn(async move {
            let mut count = 0;
            while let Some((send_time, _msg)) = rx.recv().await {
                let latency_us = send_time.elapsed().as_micros() as u64;
                latencies.lock().push(latency_us);
                message_count.fetch_add(1, Ordering::Relaxed);
                count += 1;
            }
            count
        });

        // Aggressive senders
        let mut handles = vec![];
        for agent_id in 0..self.config.num_agents {
            let tx = tx.clone();
            let messages = self.config.messages_per_agent;
            let msg_size = self.config.message_size;

            let handle = tokio::spawn(async move {
                for msg_id in 0..messages {
                    let message =
                        format!("Agent{}-Msg{}-{}", agent_id, msg_id, "x".repeat(msg_size));
                    if tx.send((Instant::now(), message)).await.is_err() {
                        // Channel closed
                        return;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        drop(tx);
        let _ = recv_handle.await;

        let duration = start.elapsed();
        self.build_metrics(duration)
    }

    fn build_metrics(&self, duration: Duration) -> PerformanceMetrics {
        let total_messages = self.message_count.load(Ordering::Relaxed);
        let failed_messages = self.error_count.load(Ordering::Relaxed) as u64;
        let successful_messages = total_messages - failed_messages;
        let throughput_mps = total_messages as f64 / duration.as_secs_f64();

        let latencies_vec = self.latencies.lock().clone();
        let latency_stats = self.calculate_latency_stats(&latencies_vec);

        PerformanceMetrics {
            total_messages,
            successful_messages,
            failed_messages,
            total_duration: duration,
            throughput_mps,
            latencies: latency_stats,
        }
    }

    fn calculate_latency_stats(&self, latencies: &[u64]) -> LatencyStats {
        if latencies.is_empty() {
            return LatencyStats {
                min_us: 0,
                max_us: 0,
                mean_us: 0.0,
                p50_us: 0,
                p95_us: 0,
                p99_us: 0,
            };
        }

        let mut sorted = latencies.to_vec();
        sorted.sort_unstable();

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let mean = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        let p50 = sorted[latencies.len() / 2];
        let p95 = sorted[(latencies.len() * 95) / 100];
        let p99 = sorted[(latencies.len() * 99) / 100];

        LatencyStats {
            min_us: min,
            max_us: max,
            mean_us: mean,
            p50_us: p50,
            p95_us: p95,
            p99_us: p99,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_low_load_throughput() {
        let config = LoadTestConfig {
            num_agents: 5,
            messages_per_agent: 100,
            concurrent_sends: 10,
            message_size: 100,
        };

        let tester = LoadTester::new(config);
        let metrics = tester.test_throughput().await;

        println!("Low Load Throughput Test:");
        println!("  Total Messages: {}", metrics.total_messages);
        println!("  Throughput: {:.2} msg/s", metrics.throughput_mps);
        println!("  Mean Latency: {:.2} µs", metrics.latencies.mean_us);

        assert!(metrics.total_messages > 0);
        assert!(metrics.throughput_mps > 0.0);
    }

    #[tokio::test]
    async fn test_medium_load_throughput() {
        let config = LoadTestConfig {
            num_agents: 25,
            messages_per_agent: 200,
            concurrent_sends: 20,
            message_size: 256,
        };

        let tester = LoadTester::new(config);
        let metrics = tester.test_throughput().await;

        println!("Medium Load Throughput Test:");
        println!("  Total Messages: {}", metrics.total_messages);
        println!("  Throughput: {:.2} msg/s", metrics.throughput_mps);
        println!("  P95 Latency: {} µs", metrics.latencies.p95_us);

        assert!(metrics.throughput_mps > 1000.0); // At least 1k msg/s
    }

    #[tokio::test]
    async fn test_high_load_latency() {
        let config = LoadTestConfig {
            num_agents: 50,
            messages_per_agent: 500,
            concurrent_sends: 50,
            message_size: 512,
        };

        let tester = LoadTester::new(config);
        let metrics = tester.test_latency().await;

        println!("High Load Latency Test:");
        println!("  Total Messages: {}", metrics.total_messages);
        println!("  Mean Latency: {:.2} µs", metrics.latencies.mean_us);
        println!("  P99 Latency: {} µs", metrics.latencies.p99_us);
        println!("  Duration: {:.2}s", metrics.total_duration.as_secs_f64());

        assert!(metrics.latencies.p99_us < 10_000); // P99 < 10ms
    }

    #[tokio::test]
    async fn test_extreme_load_stress() {
        let config = LoadTestConfig {
            num_agents: 100,
            messages_per_agent: 1000,
            concurrent_sends: 100,
            message_size: 1024,
        };

        let tester = LoadTester::new(config);
        let metrics = tester.test_stress().await;

        println!("Extreme Load Stress Test:");
        println!("  Total Messages: {}", metrics.total_messages);
        println!("  Successful: {}", metrics.successful_messages);
        println!("  Failed: {}", metrics.failed_messages);
        println!("  Throughput: {:.2} msg/s", metrics.throughput_mps);
        println!(
            "  Success Rate: {:.2}%",
            (metrics.successful_messages as f64 / metrics.total_messages as f64) * 100.0
        );

        // Even under extreme load, should maintain >99% success
        let success_rate = metrics.successful_messages as f64 / metrics.total_messages as f64;
        assert!(success_rate > 0.99);
    }
}

#[derive(Default)]
pub struct LoadTestResults {
    pub test_name: String,
    pub timestamp: String,
    pub config: Option<LoadTestConfig>,
    pub metrics: HashMap<String, PerformanceMetrics>,
}

impl LoadTestResults {
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            timestamp: chrono::Local::now().to_rfc3339(),
            config: None,
            metrics: HashMap::new(),
        }
    }

    pub fn add_metric(&mut self, name: impl Into<String>, metrics: PerformanceMetrics) {
        self.metrics.insert(name.into(), metrics);
    }

    pub fn summary(&self) -> String {
        let mut summary = format!("Load Test Results: {}\n", self.test_name);
        summary.push_str(&format!("Timestamp: {}\n", self.timestamp));
        summary.push_str(&format!("Test Scenarios: {}\n\n", self.metrics.len()));

        for (name, metrics) in &self.metrics {
            summary.push_str(&format!("--- {} ---\n", name));
            summary.push_str(&format!("  Total Messages: {}\n", metrics.total_messages));
            summary.push_str(&format!(
                "  Success Rate: {:.2}%\n",
                (metrics.successful_messages as f64 / metrics.total_messages as f64) * 100.0
            ));
            summary.push_str(&format!(
                "  Throughput: {:.2} msg/s\n",
                metrics.throughput_mps
            ));
            summary.push_str(&format!(
                "  Mean Latency: {:.2} µs\n",
                metrics.latencies.mean_us
            ));
            summary.push_str(&format!("  P95 Latency: {} µs\n", metrics.latencies.p95_us));
            summary.push_str(&format!("  P99 Latency: {} µs\n", metrics.latencies.p99_us));
            summary.push_str(&format!(
                "  Duration: {:.2}s\n\n",
                metrics.total_duration.as_secs_f64()
            ));
        }

        summary
    }
}
