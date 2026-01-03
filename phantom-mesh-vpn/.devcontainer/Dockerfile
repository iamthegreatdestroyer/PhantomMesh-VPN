# Development Container for PhantomMesh-VPN
# Multi-language development environment: Rust, Python, Go
# Base: Ubuntu 22.04 LTS for stability and compatibility

FROM mcr.microsoft.com/devcontainers/rust:1-ubuntu-22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_BACKTRACE=1
ENV PYTHONUNBUFFERED=1

# Install system dependencies for both Rust and Python development
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build essentials (already included in base, but ensure)
    build-essential \
    pkg-config \
    \
    # Python development
    python3.11 \
    python3.11-dev \
    python3.11-venv \
    python3-pip \
    python3-setuptools \
    \
    # Rust-specific tools
    rustfmt \
    clippy \
    cargo-audit \
    cargo-edit \
    cargo-watch \
    \
    # Docker in Docker (for container development)
    docker.io \
    \
    # Go support (for integration capabilities)
    golang-go \
    \
    # Kubernetes tools
    kubectl \
    \
    # Database tools
    postgresql-client \
    sqlite3 \
    redis-tools \
    \
    # Network and debugging tools
    curl \
    wget \
    git \
    git-lfs \
    htop \
    tmux \
    vim \
    nano \
    ripgrep \
    fd-find \
    jq \
    yq \
    \
    # Protobuf compiler (for gRPC)
    protobuf-compiler \
    \
    # OpenSSL development (required for many Rust crates)
    libssl-dev \
    \
    # TLS/Security tools
    openssl \
    ca-certificates \
    \
    # Development utilities
    tldr \
    dnsutils \
    net-tools \
    iputils-ping \
    traceroute \
    \
    # Cleanup
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain components
RUN rustup update && \
    rustup component add rustfmt clippy && \
    rustup target add x86_64-unknown-linux-gnu && \
    rustup target add wasm32-unknown-unknown

# Install Python development tools globally
RUN pip3 install --upgrade \
    pip \
    setuptools \
    wheel \
    virtualenv \
    poetry \
    black \
    flake8 \
    pylint \
    mypy \
    pytest \
    pytest-cov \
    pytest-asyncio \
    pytest-xdist \
    ipython \
    jupyter \
    jupyterlab \
    requests \
    cryptography \
    pyyaml \
    toml \
    attrs \
    dataclasses-json

# Install Node.js (for JavaScript/TypeScript support if needed)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g yarn pnpm

# Install Helm (Kubernetes package manager)
RUN curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Install Docker Compose
RUN curl -L "https://github.com/docker/compose/releases/download/v2.24.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose && \
    chmod +x /usr/local/bin/docker-compose

# Set up non-root user to have Docker access
RUN usermod -aG docker vscode

# Install Prometheus and related tools for observability development
RUN pip3 install prometheus-client alertmanager-client

# Create development workspace directories
RUN mkdir -p /workspace/src \
    && mkdir -p /workspace/tests \
    && mkdir -p /workspace/docs

# Set working directory
WORKDIR /workspace

# Default shell
CMD ["/bin/bash"]
