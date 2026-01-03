#!/bin/bash
# P0-003: Dependency Audit Script
# PhantomMesh-VPN Automation
#
# Execute with: bash scripts/audits/p0-003-dependency-audit.sh
#
# This script performs a comprehensive security audit of all dependencies

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
AUDIT_RESULTS_DIR="${PROJECT_ROOT}/audit_results"

# Create results directory
mkdir -p "$AUDIT_RESULTS_DIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  P0-003: Dependency Audit                                    ║"
echo "║  PhantomMesh-VPN Supply Chain Analysis                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}▶ Analyzing Rust Dependencies${NC}"
echo "  Running cargo audit..."

if command -v cargo-audit &> /dev/null; then
    cargo audit --json > "$AUDIT_RESULTS_DIR/rust_audit.json" 2>&1 || true
    echo "  ✓ Rust security audit complete"
else
    echo -e "${YELLOW}  ⚠ cargo-audit not installed${NC}"
    echo "  Install with: cargo install cargo-audit"
fi

echo "  Running cargo outdated..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated --format json > "$AUDIT_RESULTS_DIR/rust_outdated.json" 2>&1 || true
    echo "  ✓ Rust outdated check complete"
else
    echo -e "${YELLOW}  ⚠ cargo-outdated not installed${NC}"
    echo "  Install with: cargo install cargo-outdated"
fi

echo "  Generating SBOM (Software Bill of Materials)..."
if command -v cargo-sbom &> /dev/null; then
    cargo sbom > "$AUDIT_RESULTS_DIR/rust_sbom.spdx" 2>&1 || true
    echo "  ✓ Rust SBOM generated"
else
    echo -e "${YELLOW}  ⚠ cargo-sbom not installed${NC}"
fi

echo ""
echo -e "${BLUE}▶ Analyzing Python Dependencies${NC}"
echo "  Running pip audit..."

if command -v pip-audit &> /dev/null; then
    pip-audit --output json --progress-spinner off > "$AUDIT_RESULTS_DIR/python_audit.json" 2>&1 || true
    echo "  ✓ Python security audit complete"
else
    echo -e "${YELLOW}  ⚠ pip-audit not installed${NC}"
    echo "  Install with: pip install pip-audit"
fi

echo "  Analyzing installed packages..."
pip list --format json > "$AUDIT_RESULTS_DIR/python_dependencies.json" 2>&1 || true
echo "  ✓ Python dependencies listed"

echo "  Running pipdeptree..."
if command -v pipdeptree &> /dev/null; then
    pipdeptree --json > "$AUDIT_RESULTS_DIR/python_dep_tree.json" 2>&1 || true
    echo "  ✓ Python dependency tree generated"
else
    echo -e "${YELLOW}  ⚠ pipdeptree not installed${NC}"
    echo "  Install with: pip install pipdeptree"
fi

echo ""
echo -e "${BLUE}▶ License Compliance Check${NC}"
echo "  Analyzing Rust licenses..."

if command -v cargo-license &> /dev/null; then
    cargo license --json > "$AUDIT_RESULTS_DIR/rust_licenses.json" 2>&1 || true
    cargo license -t > "$AUDIT_RESULTS_DIR/rust_license_tree.txt" 2>&1 || true
    echo "  ✓ Rust license analysis complete"
else
    echo -e "${YELLOW}  ⚠ cargo-license not installed${NC}"
    echo "  Install with: cargo install cargo-license"
fi

echo ""
echo -e "${BLUE}▶ Supply Chain Risk Analysis${NC}"

# Count total dependencies
echo "  Analyzing dependency metrics..."
if [ -f "$AUDIT_RESULTS_DIR/rust_dependencies.json" ]; then
    RUST_DEPS=$(jq 'length' "$AUDIT_RESULTS_DIR/rust_dependencies.json" 2>/dev/null || echo "?")
    echo "  ✓ Rust direct dependencies: $RUST_DEPS"
fi

# Check for dependency updates
echo "  Checking for available updates..."
cargo update --dry-run 2>&1 | grep "Updating\|Removing" | head -10 || true
echo "  ✓ Update check complete"

echo ""
echo -e "${GREEN}✓ Audit complete! Results saved to:${NC}"
echo "  $AUDIT_RESULTS_DIR/"
echo ""
echo "Key files:"
echo "  - rust_audit.json          Security vulnerabilities (Rust)"
echo "  - rust_outdated.json       Outdated packages (Rust)"
echo "  - python_audit.json        Security vulnerabilities (Python)"
echo "  - python_dep_tree.json     Dependency tree (Python)"
echo "  - rust_licenses.json       License information"
echo ""
echo "Generate report with: python scripts/audits/generate_audit_report.py"
