#!/usr/bin/env python3
"""
P0-003: Dependency Audit Report Generator
PhantomMesh-VPN Supply Chain Analysis

This script generates a comprehensive audit report from collected dependency data.
"""

import json
import sys
from pathlib import Path
from typing import Any, Dict, List
from datetime import datetime

class DependencyAuditReporter:
    """Generate comprehensive dependency audit reports."""
    
    def __init__(self, audit_dir: Path):
        self.audit_dir = audit_dir
        self.audit_dir.mkdir(exist_ok=True)
        self.report = {
            "timestamp": datetime.now().isoformat(),
            "rust_audit": {},
            "python_audit": {},
            "summary": {},
            "recommendations": []
        }
    
    def load_json_file(self, filename: str) -> Dict[str, Any]:
        """Safely load JSON file."""
        filepath = self.audit_dir / filename
        if not filepath.exists():
            return {}
        
        try:
            with open(filepath, 'r') as f:
                return json.load(f)
        except json.JSONDecodeError:
            print(f"Warning: Could not parse {filename}")
            return {}
    
    def analyze_rust_audit(self):
        """Analyze Rust security audit results."""
        data = self.load_json_file("rust_audit.json")
        
        if not data:
            return
        
        vulnerabilities = data.get("vulnerabilities", [])
        self.report["rust_audit"] = {
            "total_vulnerabilities": len(vulnerabilities),
            "vulnerabilities": vulnerabilities,
            "audit_datetime": data.get("audit_datetime", "unknown")
        }
        
        if vulnerabilities:
            self.report["recommendations"].append(
                f"🔴 CRITICAL: {len(vulnerabilities)} Rust security vulnerability(ies) found. "
                f"Review and update immediately."
            )
    
    def analyze_python_audit(self):
        """Analyze Python security audit results."""
        data = self.load_json_file("python_audit.json")
        
        if not data:
            return
        
        vulnerabilities = data.get("vulnerabilities", [])
        self.report["python_audit"] = {
            "total_vulnerabilities": len(vulnerabilities),
            "vulnerabilities": vulnerabilities,
            "scanned_environments": data.get("scanned_environments", [])
        }
        
        if vulnerabilities:
            self.report["recommendations"].append(
                f"🔴 CRITICAL: {len(vulnerabilities)} Python security vulnerability(ies) found. "
                f"Review and update immediately."
            )
    
    def generate_summary(self):
        """Generate executive summary."""
        rust_vulns = len(self.report["rust_audit"].get("vulnerabilities", []))
        python_vulns = len(self.report["python_audit"].get("vulnerabilities", []))
        total_vulns = rust_vulns + python_vulns
        
        self.report["summary"] = {
            "total_vulnerabilities": total_vulns,
            "rust_vulnerabilities": rust_vulns,
            "python_vulnerabilities": python_vulns,
            "audit_status": "CRITICAL" if total_vulns > 0 else "SAFE",
            "risk_level": self._calculate_risk_level(total_vulns)
        }
        
        if total_vulns == 0:
            self.report["recommendations"].append(
                "✅ No known security vulnerabilities found in current versions."
            )
        
        if rust_vulns > 5:
            self.report["recommendations"].append(
                "⚠️  HIGH: Consider upgrading Rust dependencies more frequently."
            )
        
        if python_vulns > 5:
            self.report["recommendations"].append(
                "⚠️  HIGH: Consider updating Python dependencies."
            )
    
    def _calculate_risk_level(self, vuln_count: int) -> str:
        """Determine overall risk level."""
        if vuln_count == 0:
            return "LOW"
        elif vuln_count < 3:
            return "MEDIUM"
        else:
            return "HIGH"
    
    def generate_report(self):
        """Generate the complete audit report."""
        self.analyze_rust_audit()
        self.analyze_python_audit()
        self.generate_summary()
        return self.report
    
    def save_report(self, output_file: Path):
        """Save report to JSON file."""
        with open(output_file, 'w') as f:
            json.dump(self.report, f, indent=2, default=str)
        print(f"✓ Report saved to: {output_file}")
    
    def print_report(self):
        """Print human-readable report."""
        summary = self.report["summary"]
        print("\n" + "="*70)
        print("DEPENDENCY AUDIT REPORT")
        print("="*70)
        print(f"Generated: {self.report['timestamp']}")
        print(f"\nOVERALL STATUS: {summary.get('audit_status', 'UNKNOWN')}")
        print(f"Risk Level: {summary.get('risk_level', 'UNKNOWN')}")
        print(f"\nVulnerability Count:")
        print(f"  Rust:   {summary.get('rust_vulnerabilities', 0)}")
        print(f"  Python: {summary.get('python_vulnerabilities', 0)}")
        print(f"  Total:  {summary.get('total_vulnerabilities', 0)}")
        
        if self.report["recommendations"]:
            print("\nRECOMMENDATIONS:")
            for i, rec in enumerate(self.report["recommendations"], 1):
                print(f"  {i}. {rec}")
        
        print("\n" + "="*70)


def main():
    """Main entry point."""
    # Assume we're running from project root or audit script directory
    current_dir = Path.cwd()
    if (current_dir / "audit_results").exists():
        audit_dir = current_dir / "audit_results"
    else:
        audit_dir = current_dir / "audit_results"
        audit_dir.mkdir(exist_ok=True)
    
    print("📊 Generating Dependency Audit Report...")
    
    reporter = DependencyAuditReporter(audit_dir)
    report = reporter.generate_report()
    
    # Save report files
    reporter.save_report(audit_dir / "dependency_audit_report.json")
    
    # Print summary
    reporter.print_report()
    
    # Exit with appropriate code
    status = report["summary"].get("audit_status", "UNKNOWN")
    if status == "CRITICAL":
        sys.exit(1)
    elif status == "HIGH":
        sys.exit(0)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()
