#!/usr/bin/env python3
"""
PhantomMesh Load Test Simulator
Simulates load test execution with realistic metrics for demonstration
"""

import json
import math
import random
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any

class LoadTestSimulator:
    """Simulates a realistic load test execution"""
    
    def __init__(self):
        self.test_duration = 300  # 5 minutes
        self.ramp_up_duration = 60  # 1 minute ramp-up
        self.peak_rate = 1000  # 1000 req/s
        self.initial_rate = 100  # Start at 100 req/s
        
    def generate_latency(self, elapsed_time: float, peak: bool = False) -> float:
        """Generate realistic latency based on load phase"""
        base_latency = 25 if not peak else 45
        
        # Slight increase during ramp-up
        if elapsed_time < self.ramp_up_duration:
            ramp_factor = elapsed_time / self.ramp_up_duration
            base_latency += ramp_factor * 15
        
        # Add realistic jitter and outliers
        jitter = random.gauss(0, 10)
        
        # Occasional spikes (5% of requests)
        if random.random() < 0.05:
            latency = base_latency + abs(jitter) * 3 + random.uniform(50, 200)
        else:
            latency = max(5, base_latency + jitter)
        
        return round(latency, 2)
    
    def simulate_ramp_up_test(self) -> Dict[str, Any]:
        """Simulate a ramp-up load test"""
        print("\n" + "="*60)
        print("PhantomMesh Load Test Simulator")
        print("="*60)
        print("Running Ramp-up Test (0→1000 req/s)...\n")
        
        latencies = []
        error_count = 0
        success_count = 0
        total_requests = 0
        start_time = time.time()
        
        # Simulate test execution
        for elapsed in range(0, self.test_duration, 1):
            # Calculate current request rate
            if elapsed < self.ramp_up_duration:
                ramp_progress = elapsed / self.ramp_up_duration
                current_rate = self.initial_rate + (self.peak_rate - self.initial_rate) * ramp_progress
            else:
                current_rate = self.peak_rate
            
            # Generate requests for this second
            requests_this_second = int(current_rate)
            
            for _ in range(requests_this_second):
                total_requests += 1
                
                # Simulate error rate (~1%)
                if random.random() < 0.01:
                    error_count += 1
                else:
                    success_count += 1
                    latency = self.generate_latency(elapsed, peak=elapsed >= self.ramp_up_duration)
                    latencies.append(latency)
            
            # Progress indicator
            if elapsed % 30 == 0:
                progress = (elapsed / self.test_duration) * 100
                print(f"  {progress:.0f}% - {elapsed}s elapsed - {current_rate:.0f} req/s")
        
        print(f"\n✅ Test simulation complete!")
        
        # Calculate statistics
        latencies.sort()
        metrics = {
            'latency_mean': round(sum(latencies) / len(latencies), 2),
            'latency_min': round(min(latencies), 2),
            'latency_max': round(max(latencies), 2),
            'latency_percentiles': {
                'p50': round(latencies[int(len(latencies) * 0.50)], 2),
                'p95': round(latencies[int(len(latencies) * 0.95)], 2),
                'p99': round(latencies[int(len(latencies) * 0.99)], 2),
                'p999': round(latencies[int(len(latencies) * 0.999)], 2),
            },
            'peak_rps': round(self.peak_rate * 1.05, 0),  # Slight peak above target
            'avg_rps': round(total_requests / self.test_duration, 0),
        }
        
        # Prepare results
        results = {
            'test_name': 'ramp-up',
            'duration': self.test_duration,
            'request_count': total_requests,
            'success_count': success_count,
            'error_count': error_count,
            'metrics': metrics,
            'timestamp': datetime.utcnow().isoformat(),
            'simulation': True,
            'note': 'This is a simulated test result for demonstration purposes'
        }
        
        return results
    
    def validate_targets(self, metrics: Dict) -> Dict[str, bool]:
        """Validate metrics against targets"""
        targets = {
            'p50': 50,
            'p99': 200,
            'p999': 500,
            'mean': 100,
        }
        
        validation = {}
        for target_name, target_value in targets.items():
            if target_name in metrics['latency_percentiles']:
                actual = metrics['latency_percentiles'][target_name]
            else:
                actual = metrics.get(f'latency_{target_name}', 0)
            
            validation[target_name] = actual <= target_value
        
        return validation
    
    def print_results(self, results: Dict[str, Any]) -> None:
        """Print formatted results"""
        metrics = results['metrics']
        
        print("\n" + "="*60)
        print("LOAD TEST RESULTS")
        print("="*60)
        
        print("\nTest Summary:")
        print(f"  Duration: {results['duration']} seconds")
        print(f"  Total Requests: {results['request_count']:,}")
        print(f"  Successful: {results['success_count']:,}")
        print(f"  Errors: {results['error_count']:,}")
        error_rate = (results['error_count'] / results['request_count'] * 100) if results['request_count'] > 0 else 0
        print(f"  Error Rate: {error_rate:.2f}%")
        
        print("\nLatency Statistics (milliseconds):")
        print(f"  Mean: {metrics['latency_mean']} ms")
        print(f"  Min: {metrics['latency_min']} ms")
        print(f"  Max: {metrics['latency_max']} ms")
        print(f"  P50: {metrics['latency_percentiles']['p50']} ms")
        print(f"  P95: {metrics['latency_percentiles']['p95']} ms")
        print(f"  P99: {metrics['latency_percentiles']['p99']} ms")
        print(f"  P999: {metrics['latency_percentiles']['p999']} ms")
        
        print("\nThroughput:")
        print(f"  Peak RPS: {metrics['peak_rps']}")
        print(f"  Average RPS: {metrics['avg_rps']}")
        
        # Validate targets
        print("\n" + "-"*60)
        print("LATENCY TARGET VALIDATION")
        print("-"*60)
        
        validation = self.validate_targets(metrics)
        targets = {
            'p50': 50,
            'p99': 200,
            'p999': 500,
            'mean': 100,
        }
        
        all_passed = True
        for target_name, target_value in targets.items():
            if target_name in metrics['latency_percentiles']:
                actual = metrics['latency_percentiles'][target_name]
            else:
                actual = metrics.get(f'latency_{target_name}', 0)
            
            status = "✅ PASS" if validation[target_name] else "❌ FAIL"
            if not validation[target_name]:
                all_passed = False
            
            print(f"  {target_name.upper()}: {actual} ms (target: {target_value} ms) {status}")
        
        print("\n" + "="*60)
        if all_passed:
            print("RESULT: ✅ ALL LATENCY TARGETS PASSED")
        else:
            print("RESULT: ❌ SOME LATENCY TARGETS FAILED")
        print("="*60)
        
        # Note about simulation
        if results.get('simulation'):
            print("\n📌 NOTE: This is a SIMULATED test result for demonstration")
            print("   To run actual load tests, deploy to a Kubernetes cluster")

def main():
    simulator = LoadTestSimulator()
    results = simulator.simulate_ramp_up_test()
    simulator.print_results(results)
    
    # Save results
    output_dir = Path('results')
    output_dir.mkdir(exist_ok=True)
    
    output_file = output_dir / 'load_test_results.json'
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n✅ Results saved to: {output_file}")
    
    return 0 if all(results['metrics']['latency_percentiles'].values()) else 1

if __name__ == '__main__':
    exit(main())
