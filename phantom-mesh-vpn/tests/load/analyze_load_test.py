#!/usr/bin/env python3
"""
PhantomMesh Load Test Metrics Analysis
Analyzes and visualizes load test results
"""

import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, List
import statistics

def load_results(filepath: str) -> Dict[str, Any]:
    """Load test results from JSON file"""
    with open(filepath, 'r') as f:
        return json.load(f)

def analyze_latencies(latencies: List[float]) -> Dict[str, float]:
    """Analyze latency distribution"""
    if not latencies:
        return {}
    
    sorted_lats = sorted(latencies)
    return {
        'min': min(latencies),
        'max': max(latencies),
        'mean': statistics.mean(latencies),
        'median': statistics.median(latencies),
        'stdev': statistics.stdev(latencies) if len(latencies) > 1 else 0,
        'p50': sorted_lats[int(len(sorted_lats) * 0.50)],
        'p95': sorted_lats[int(len(sorted_lats) * 0.95)],
        'p99': sorted_lats[int(len(sorted_lats) * 0.99)],
        'p999': sorted_lats[int(len(sorted_lats) * 0.999)],
    }

def validate_targets(metrics: Dict[str, float], targets: Dict[str, float]) -> Dict[str, bool]:
    """Validate metrics against targets"""
    results = {}
    for key, target in targets.items():
        if key in metrics:
            results[key] = metrics[key] <= target
    return results

def print_report(results: Dict[str, Any]):
    """Print formatted test report"""
    
    print("\n" + "="*60)
    print("PhantomMesh Load Test Analysis Report")
    print("="*60 + "\n")
    
    # Test summary
    print("TEST SUMMARY")
    print("-" * 60)
    print(f"Test Name: {results.get('test_name', 'N/A')}")
    print(f"Duration: {results.get('duration', 0)} seconds")
    print(f"Total Requests: {results.get('request_count', 0)}")
    print(f"Successful: {results.get('success_count', 0)}")
    print(f"Failed: {results.get('error_count', 0)}")
    
    if results.get('request_count', 0) > 0:
        error_rate = (results['error_count'] / results['request_count']) * 100
        print(f"Error Rate: {error_rate:.2f}%")
    print()
    
    # Latency analysis
    print("LATENCY ANALYSIS (milliseconds)")
    print("-" * 60)
    metrics = results.get('metrics', {})
    latency = metrics.get('latency_percentiles', {})
    
    print(f"Mean:  {metrics.get('latency_mean', 0):.2f} ms")
    print(f"Min:   {metrics.get('latency_min', 0):.2f} ms")
    print(f"Max:   {metrics.get('latency_max', 0):.2f} ms")
    print()
    print(f"P50:   {latency.get('p50', 0):.2f} ms")
    print(f"P95:   {latency.get('p95', 0):.2f} ms")
    print(f"P99:   {latency.get('p99', 0):.2f} ms")
    print(f"P999:  {latency.get('p999', 0):.2f} ms")
    print()
    
    # Throughput
    print("THROUGHPUT")
    print("-" * 60)
    print(f"Peak RPS:    {metrics.get('peak_rps', 0):.2f}")
    print(f"Average RPS: {metrics.get('avg_rps', 0):.2f}")
    print()
    
    # Target validation
    print("LATENCY TARGET VALIDATION")
    print("-" * 60)
    targets = {
        'p50': 50,
        'p99': 200,
        'p999': 500,
        'mean': 100
    }
    
    validation_passed = True
    for target_name, target_value in targets.items():
        actual = latency.get(target_name) or metrics.get(f'latency_{target_name}')
        if actual:
            status = "✅ PASS" if actual <= target_value else "❌ FAIL"
            if "FAIL" in status:
                validation_passed = False
            print(f"{target_name.upper()}: {actual:.2f} ms / {target_value} ms - {status}")
    
    print()
    print("="*60)
    if validation_passed:
        print("RESULT: ✅ ALL TARGETS PASSED")
    else:
        print("RESULT: ❌ SOME TARGETS FAILED")
    print("="*60 + "\n")

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_load_test.py <results.json>")
        sys.exit(1)
    
    results_file = sys.argv[1]
    
    if not Path(results_file).exists():
        print(f"Error: File not found: {results_file}")
        sys.exit(1)
    
    results = load_results(results_file)
    print_report(results)

if __name__ == "__main__":
    main()
