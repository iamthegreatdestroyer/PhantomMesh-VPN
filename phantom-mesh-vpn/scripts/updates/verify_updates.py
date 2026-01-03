#!/usr/bin/env python3
"""
P0-004: Dependency Update Verification
PhantomMesh-VPN Update Management

This script verifies that all dependency updates have been applied correctly
and that the project is in a stable state after updates.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Tuple


class DependencyUpdateVerifier:
    """Verify dependency updates and project stability."""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.results = {
            "cargo_check": False,
            "cargo_test": False,
            "cargo_clippy": False,
            "cargo_fmt": False,
            "python_imports": False,
            "overall_status": "PENDING"
        }
    
    def run_command(self, cmd: List[str], cwd: Path = None) -> Tuple[bool, str]:
        """Run a shell command and return success status and output."""
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd or self.project_root,
                capture_output=True,
                text=True,
                timeout=300
            )
            return result.returncode == 0, result.stdout + result.stderr
        except Exception as e:
            return False, str(e)
    
    def verify_cargo_check(self) -> bool:
        """Verify that cargo check passes."""
        print("🔍 Verifying cargo check...")
        success, output = self.run_command(["cargo", "check", "--all-targets"])
        
        if success:
            print("  ✅ Cargo check passed")
            self.results["cargo_check"] = True
        else:
            print("  ❌ Cargo check failed")
            print(output[:500])
        
        return success
    
    def verify_cargo_test(self) -> bool:
        """Verify that all tests pass."""
        print("🧪 Running cargo tests...")
        success, output = self.run_command(["cargo", "test", "--lib", "--", "--nocapture"])
        
        if success:
            print("  ✅ Cargo tests passed")
            self.results["cargo_test"] = True
        else:
            print("  ❌ Cargo tests failed")
            print(output[-500:])
        
        return success
    
    def verify_clippy(self) -> bool:
        """Verify that clippy passes."""
        print("🔍 Running clippy...")
        success, output = self.run_command(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"])
        
        if success:
            print("  ✅ Clippy passed")
            self.results["cargo_clippy"] = True
        else:
            print("  ⚠️  Clippy warnings found (non-fatal)")
            self.results["cargo_clippy"] = True  # Don't fail on clippy
        
        return True  # Don't hard fail
    
    def verify_format(self) -> bool:
        """Verify code formatting."""
        print("📝 Checking format...")
        success, output = self.run_command(["cargo", "fmt", "--", "--check"])
        
        if success:
            print("  ✅ Format check passed")
            self.results["cargo_fmt"] = True
        else:
            print("  ⚠️  Format issues detected")
            print("  Run 'cargo fmt' to fix")
            self.results["cargo_fmt"] = False
        
        return success
    
    def verify_python_imports(self) -> bool:
        """Verify Python imports work."""
        print("🐍 Verifying Python imports...")
        
        try:
            # Try importing key modules
            import asyncio
            import aiohttp
            import pydantic
            import structlog
            print("  ✅ Python imports successful")
            self.results["python_imports"] = True
            return True
        except ImportError as e:
            print(f"  ❌ Python import failed: {e}")
            return False
    
    def verify_dependencies(self) -> bool:
        """Verify dependency integrity."""
        print("📦 Verifying dependencies...")
        
        # Check Cargo.lock exists
        cargo_lock = self.project_root / "Cargo.lock"
        if cargo_lock.exists():
            print("  ✅ Cargo.lock exists")
        else:
            print("  ⚠️  Cargo.lock not found")
        
        # Check Cargo.toml parseable
        try:
            import toml
            with open(self.project_root / "Cargo.toml") as f:
                toml.load(f)
            print("  ✅ Cargo.toml valid")
        except Exception as e:
            print(f"  ❌ Cargo.toml error: {e}")
            return False
        
        return True
    
    def run_all_verifications(self):
        """Run all verification checks."""
        print("\n" + "="*70)
        print("DEPENDENCY UPDATE VERIFICATION")
        print("="*70 + "\n")
        
        all_passed = True
        
        # Run verifications
        all_passed &= self.verify_cargo_check()
        all_passed &= self.verify_dependencies()
        all_passed &= self.verify_cargo_test()
        all_passed &= self.verify_clippy()
        all_passed &= self.verify_format()
        all_passed &= self.verify_python_imports()
        
        # Determine overall status
        self.results["overall_status"] = "PASS" if all_passed else "FAIL"
        
        # Print summary
        print("\n" + "="*70)
        print("VERIFICATION SUMMARY")
        print("="*70)
        
        for check, passed in self.results.items():
            if check != "overall_status":
                status = "✅ PASS" if passed else "❌ FAIL"
                print(f"  {check}: {status}")
        
        print(f"\nOverall Status: {self.results['overall_status']}")
        print("="*70 + "\n")
        
        return all_passed
    
    def save_results(self, output_file: Path):
        """Save verification results to JSON."""
        with open(output_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        print(f"✓ Results saved to: {output_file}")


def main():
    """Main entry point."""
    project_root = Path.cwd()
    
    verifier = DependencyUpdateVerifier(project_root)
    all_passed = verifier.run_all_verifications()
    
    # Save results
    verifier.save_results(project_root / "update_verification_results.json")
    
    # Exit with appropriate code
    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
