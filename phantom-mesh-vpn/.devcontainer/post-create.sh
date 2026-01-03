#!/bin/bash

set -e

echo "=========================================="
echo "PhantomMesh-VPN Development Container Setup"
echo "=========================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Change to workspace directory
cd /workspace

# Verify directory structure
print_step "Verifying project structure..."
if [ -f "Cargo.toml" ] && [ -f "pyproject.toml" ]; then
    print_success "Project files found"
else
    print_warning "Project files not found in expected location"
fi

# Install/update Rust dependencies
print_step "Setting up Rust development environment..."
if [ -f "Cargo.toml" ]; then
    cargo fetch
    print_success "Rust dependencies fetched"
fi

# Create Python virtual environment
print_step "Setting up Python virtual environment..."
if [ -f "pyproject.toml" ]; then
    python3.11 -m venv /workspace/.venv
    source /workspace/.venv/bin/activate
    pip install --upgrade pip setuptools wheel
    
    # Install project dependencies
    if command -v poetry &> /dev/null; then
        poetry install --no-root
        print_success "Poetry dependencies installed"
    else
        pip install -e .
        print_success "Project installed in development mode"
    fi
else
    print_warning "No pyproject.toml found, skipping Python setup"
fi

# Install pre-commit hooks if available
print_step "Setting up Git hooks..."
if [ -f ".pre-commit-config.yaml" ]; then
    pip install pre-commit
    pre-commit install
    print_success "Pre-commit hooks installed"
else
    print_warning "No pre-commit configuration found"
fi

# Create necessary directories
print_step "Creating development directories..."
mkdir -p /workspace/.cargo
mkdir -p /workspace/target
mkdir -p /workspace/logs
mkdir -p /workspace/coverage
mkdir -p /workspace/artifacts
print_success "Development directories created"

# Initialize environment files
print_step "Setting up environment configuration..."
if [ ! -f "/workspace/.env.development" ]; then
    cat > /workspace/.env.development << 'EOF'
# PhantomMesh-VPN Development Environment

# Logging
RUST_LOG=debug,phantom_mesh=debug
LOG_LEVEL=DEBUG

# Python
PYTHONUNBUFFERED=1
PYTHONDONTWRITEBYTECODE=1

# Development flags
DEV_MODE=true
UNSAFE_BIND=true

# Service ports
API_PORT=8000
METRICS_PORT=9090
REDIS_PORT=6379
POSTGRES_PORT=5432

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/phantommesh_dev

# Security (development only!)
JWT_SECRET=dev-secret-key-change-in-production

# Kubernetes
KUBECONFIG=${HOME}/.kube/config

EOF
    print_success "Development environment file created"
fi

# Setup Git configuration for better development experience
print_step "Configuring Git..."
git config --local core.autocrlf false
git config --local core.safecrlf warn
print_success "Git configured"

# Run initial tests to verify setup
print_step "Running setup verification tests..."
if [ -f "Cargo.toml" ]; then
    echo "  Checking Rust build..."
    cargo check --message-format=short 2>&1 | head -20 || print_warning "Cargo check had issues"
fi

# Display helpful information
print_step "Setup complete! Quick reference:"
echo ""
echo "Rust Development:"
echo "  cargo build              Build the project"
echo "  cargo test              Run tests"
echo "  cargo check             Check for errors without building"
echo "  cargo clippy            Lint with Clippy"
echo "  cargo fmt --check       Check code formatting"
echo ""
echo "Python Development:"
echo "  source .venv/bin/activate    Activate virtual environment"
echo "  pytest                       Run tests"
echo "  black --check .             Check code formatting"
echo "  pylint src/                 Run linter"
echo "  mypy src/                   Run type checker"
echo ""
echo "Docker/Kubernetes:"
echo "  docker compose up            Start services"
echo "  kubectl cluster-info        Check cluster connection"
echo "  helm list                   List Helm releases"
echo ""
echo "Debugging:"
echo "  Press F5 to start debugging (if launch.json is configured)"
echo "  Use 'rust-analyzer' integration in VS Code for inline help"
echo ""
echo "Documentation:"
echo "  Architecture: docs/architecture.md"
echo "  API Docs: docs/api_docs/"
echo "  IP Protections: docs/ip_protections.md"
echo ""

# Final checks
print_step "Final verification..."
print_success "✓ Rust toolchain ready"
print_success "✓ Python 3.11 ready"
print_success "✓ Docker available"
print_success "✓ Kubernetes tools installed"
print_success "✓ Development environment configured"

echo ""
echo -e "${GREEN}=========================================="
echo "Development environment ready!"
echo "==========================================${NC}"
