#!/usr/bin/env python3
"""
PhantomMesh Load Testing Framework
Automated performance validation for production deployment
"""

import os
import json
import time
import logging
from datetime import datetime
from dataclasses import dataclass
from typing import List, Dict, Any
import random
import statistics

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('load_test.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


@dataclass
class LoadTestConfig:
    """Configuration for load tests"""
    target_url: str = "http://localhost:8080"
    test_duration_seconds: int = 300
    ramp_up_seconds: int = 60
    cooldown_seconds: int = 30
    initial_rate: int = 100  # req/sec
    peak_rate: int = 1000    # req/sec
    spike_rate: int = 2000   # req/sec
    spike_duration: int = 60
    concurrent_users: int = 10


class LoadTestRunner:
    """Main load testing orchestrator"""

    def __init__(self, config: LoadTestConfig):
        self.config = config
        self.session = self._create_session()
        self.results = {
            'latencies': [],
            'errors': [],
            'throughput': [],
            'timestamps': [],
            'test_name': '',
            'duration': 0,
            'start_time': None,
            'end_time': None
        }

    def _create_session(self) -> requests.Session:
        """Create HTTP session with retries"""
        session = requests.Session()
        retry_strategy = Retry(
            total=3,
            status_forcelist=[429, 500, 502, 503, 504],
            method_whitelist=["HEAD", "GET", "OPTIONS", "POST"],
            backoff_factor=1
        )
        adapter = HTTPAdapter(max_retries=retry_strategy, pool_connections=50, pool_maxsize=50)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        return session

    def generate_threat_signal(self) -> Dict[str, Any]:
        """Generate realistic threat signal"""
        return {
            "threat_signal": {
                "protocol": random.choice(["tcp", "udp", "icmp"]),
                "source_ip": f"192.168.{random.randint(0,255)}.{random.randint(0,255)}",
                "destination_ip": f"10.0.{random.randint(0,255)}.{random.randint(0,255)}",
                "port": random.choice([22, 80, 443, 445, 3306, 5432, 8080, 9090]),
                "severity": random.choice(["low", "medium", "high", "critical"]),
                "confidence": round(random.uniform(0.5, 1.0), 2),
                "timestamp": datetime.utcnow().isoformat()
            }
        }

    def send_threat_event(self, payload: Dict[str, Any]) -> tuple[float, bool, str]:
        """Send single threat event and measure latency"""
        start_time = time.time()
        try:
            response = self.session.post(
                f"{self.config.target_url}/api/v1/automation/process-threat",
                json=payload,
                timeout=10
            )
            latency = (time.time() - start_time) * 1000  # Convert to ms

            if response.status_code in [200, 201, 202]:
                return latency, True, response.text[:50]
            else:
                return latency, False, f"Status {response.status_code}"

        except requests.exceptions.RequestException as e:
            latency = (time.time() - start_time) * 1000
            return latency, False, str(e)

    def ramp_up_load(self) -> Dict[str, Any]:
        """Ramp up load gradually (0 -> peak_rate)"""
        logger.info(f"Starting ramp-up over {self.config.ramp_up_seconds}s to {self.config.peak_rate} req/s")
        
        test_name = f"ramp_up_{int(time.time())}"
        self.results['test_name'] = test_name
        self.results['start_time'] = datetime.now()
        
        start_time = time.time()
        request_count = 0
        error_count = 0

        while time.time() - start_time < self.config.ramp_up_seconds:
            elapsed = time.time() - start_time
            progress = elapsed / self.config.ramp_up_seconds
            
            # Calculate target rate at this point
            target_rate = self.config.initial_rate + (
                (self.config.peak_rate - self.config.initial_rate) * progress
            )
            
            # Send requests to reach target rate
            requests_this_second = int(target_rate / 10)  # Check every 100ms
            
            for _ in range(requests_this_second):
                payload = self.generate_threat_signal()
                latency, success, error = self.send_threat_event(payload)
                
                self.results['latencies'].append(latency)
                self.results['timestamps'].append(time.time())
                request_count += 1
                
                if not success:
                    self.results['errors'].append(error)
                    error_count += 1
            
            time.sleep(0.1)  # 100ms between checks

        self.results['duration'] = time.time() - start_time
        self.results['end_time'] = datetime.now()
        
        return self._generate_report(test_name, request_count, error_count)

    def sustained_load(self, rate: int = None, duration: int = None) -> Dict[str, Any]:
        """Run sustained load test"""
        rate = rate or self.config.peak_rate
        duration = duration or self.config.test_duration_seconds
        
        logger.info(f"Running sustained load: {rate} req/s for {duration}s")
        
        test_name = f"sustained_{rate}rps_{int(time.time())}"
        self.results['test_name'] = test_name
        self.results['start_time'] = datetime.now()
        
        start_time = time.time()
        request_count = 0
        error_count = 0

        while time.time() - start_time < duration:
            # Calculate requests to send this iteration
            elapsed = time.time() - start_time
            expected_requests = (rate * elapsed) / 1000  # rate is per second
            
            if request_count < expected_requests:
                payload = self.generate_threat_signal()
                latency, success, error = self.send_threat_event(payload)
                
                self.results['latencies'].append(latency)
                self.results['timestamps'].append(time.time())
                request_count += 1
                
                if not success:
                    self.results['errors'].append(error)
                    error_count += 1
            
            time.sleep(0.001)  # 1ms between request attempts

        self.results['duration'] = time.time() - start_time
        self.results['end_time'] = datetime.now()
        
        return self._generate_report(test_name, request_count, error_count)

    def spike_load(self) -> Dict[str, Any]:
        """Test sudden spike in load"""
        logger.info(f"Running spike test: {self.config.spike_rate} req/s for {self.config.spike_duration}s")
        
        test_name = f"spike_{int(time.time())}"
        self.results['test_name'] = test_name
        
        # First: sustained baseline load
        baseline = self.sustained_load(rate=self.config.peak_rate, duration=30)
        
        # Then: spike
        self.results['latencies'] = []
        self.results['errors'] = []
        self.results['timestamps'] = []
        self.results['start_time'] = datetime.now()
        
        spike_result = self.sustained_load(rate=self.config.spike_rate, duration=self.config.spike_duration)
        
        # Finally: recover back to baseline
        recovery = self.sustained_load(rate=self.config.peak_rate, duration=30)
        
        return {
            'test_name': test_name,
            'baseline': baseline,
            'spike': spike_result,
            'recovery': recovery
        }

    def _generate_report(self, test_name: str, request_count: int, error_count: int) -> Dict[str, Any]:
        """Generate detailed performance report"""
        if not self.results['latencies']:
            return {'test_name': test_name, 'error': 'No data collected'}

        latencies = self.results['latencies']
        
        report = {
            'test_name': test_name,
            'timestamp': datetime.now().isoformat(),
            'duration_seconds': self.results['duration'],
            'total_requests': request_count,
            'successful_requests': request_count - error_count,
            'failed_requests': error_count,
            'error_rate': (error_count / request_count * 100) if request_count > 0 else 0,
            'latency_stats': {
                'min_ms': min(latencies),
                'max_ms': max(latencies),
                'mean_ms': statistics.mean(latencies),
                'median_ms': statistics.median(latencies),
                'p95_ms': sorted(latencies)[int(len(latencies) * 0.95)] if len(latencies) > 0 else 0,
                'p99_ms': sorted(latencies)[int(len(latencies) * 0.99)] if len(latencies) > 0 else 0,
                'stdev_ms': statistics.stdev(latencies) if len(latencies) > 1 else 0
            },
            'throughput': request_count / self.results['duration'] if self.results['duration'] > 0 else 0
        }

        logger.info(f"Test Results for {test_name}:")
        logger.info(f"  Requests: {request_count} ({report['error_rate']:.2f}% error)")
        logger.info(f"  Latency: {report['latency_stats']['mean_ms']:.2f}ms mean, {report['latency_stats']['p99_ms']:.2f}ms p99")
        logger.info(f"  Throughput: {report['throughput']:.2f} req/s")

        return report

    def save_results(self, results: Dict[str, Any], filename: str = None):
        """Save test results to file"""
        if filename is None:
            filename = f"load_test_results_{int(time.time())}.json"
        
        with open(filename, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        
        logger.info(f"Results saved to {filename}")


def run_full_load_test_suite():
    """Run complete load testing suite"""
    config = LoadTestConfig(
        target_url="http://localhost:8080",
        test_duration_seconds=300,
        ramp_up_seconds=60,
        initial_rate=100,
        peak_rate=1000
    )

    runner = LoadTestRunner(config)
    all_results = {}

    try:
        # Test 1: Ramp up
        logger.info("=" * 60)
        logger.info("TEST 1: Ramp-up (0 -> 1000 req/s)")
        logger.info("=" * 60)
        all_results['ramp_up'] = runner.ramp_up_load()

        time.sleep(config.cooldown_seconds)

        # Test 2: Sustained load
        logger.info("=" * 60)
        logger.info("TEST 2: Sustained load (1000 req/s)")
        logger.info("=" * 60)
        all_results['sustained'] = runner.sustained_load()

        time.sleep(config.cooldown_seconds)

        # Test 3: Spike
        logger.info("=" * 60)
        logger.info("TEST 3: Spike test (0 -> 2000 req/s)")
        logger.info("=" * 60)
        all_results['spike'] = runner.spike_load()

        # Save all results
        runner.save_results(all_results)

        # Print summary
        logger.info("=" * 60)
        logger.info("LOAD TEST SUITE COMPLETE")
        logger.info("=" * 60)
        for test_name, result in all_results.items():
            if 'error' not in result:
                logger.info(f"\n{test_name}:")
                if 'latency_stats' in result:
                    logger.info(f"  Mean latency: {result['latency_stats']['mean_ms']:.2f}ms")
                    logger.info(f"  p99 latency: {result['latency_stats']['p99_ms']:.2f}ms")
                    logger.info(f"  Error rate: {result['error_rate']:.2f}%")

    except Exception as e:
        logger.error(f"Load test failed: {e}", exc_info=True)
        raise


if __name__ == '__main__':
    run_full_load_test_suite()
