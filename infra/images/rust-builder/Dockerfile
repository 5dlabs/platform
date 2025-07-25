# Simplified Rust builder image for fast CI builds with PVC caching
# Based on summerwind/actions-runner-dind to include GitHub Actions runner software
FROM summerwind/actions-runner-dind:latest

# Switch to root for installations
USER root

# Add LLVM repository for modern clang/lld (early in build for better caching)
RUN wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - \
    && echo "deb http://apt.llvm.org/focal/ llvm-toolchain-focal-15 main" >> /etc/apt/sources.list.d/llvm.list

# Install system dependencies in a single layer for better caching
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    # Build essentials (some may already be installed)
    build-essential \
    pkg-config \
    libssl-dev \
    # Modern tools
    curl \
    wget \
    git \
    unzip \
    # Clang for mold linker (from LLVM repo)
    clang-15 \
    lld-15 \
    # Additional tools
    jq \
    ca-certificates \
    gnupg \
    software-properties-common \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/clang-15 /usr/bin/clang \
    && ln -sf /usr/bin/clang++-15 /usr/bin/clang++

# Install mold linker (fastest linker for Rust)
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    wget -qO mold.tar.gz https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz && \
    tar xz -f mold.tar.gz && \
    cp mold-2.4.0-x86_64-linux/bin/mold /usr/local/bin/ && \
    chmod +x /usr/local/bin/mold && \
    # Create wrapper script for clang to use mold
    echo '#!/bin/sh' > /usr/local/bin/clang-mold && \
    echo 'exec clang -fuse-ld=/usr/local/bin/mold "$@"' >> /usr/local/bin/clang-mold && \
    chmod +x /usr/local/bin/clang-mold

# Install kubectl (with caching)
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install Helm (with caching)
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    curl -fsSL -o get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 && \
    chmod 700 get_helm.sh && \
    ./get_helm.sh

# Install GitHub CLI (separate layer for better caching)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Switch to runner user (already exists in summerwind image)
USER runner
WORKDIR /home/runner

# Install Rust as runner user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    echo 'source ~/.cargo/env' >> ~/.bashrc

# Add Rust to PATH
ENV PATH="/home/runner/.cargo/bin:${PATH}"

# Install essential Rust tools only (no pre-warming)
RUN ~/.cargo/bin/rustup component add clippy rustfmt && \
    ~/.cargo/bin/cargo install \
        sccache \
        cargo-nextest

# Switch back to root for final setup
USER root

# Set up optimized Cargo config for PVC caching
RUN mkdir -p /home/runner/.cargo && cat > /home/runner/.cargo/config.toml << 'EOF'
[build]
rustc-wrapper = "sccache"
jobs = 16  # Use all cores

[target.x86_64-unknown-linux-gnu]
# Use mold linker for faster linking
linker = "/usr/local/bin/clang-mold"
rustflags = [
    "-C", "link-arg=-Wl,--threads=16",  # Parallel linking with mold
    "-C", "target-cpu=x86-64-v2",  # Modern baseline CPU
]

# Fast release profile
[profile.release]
codegen-units = 16  # Good parallelism
lto = false  # Disable LTO for faster builds
debug = false
strip = true
opt-level = 2  # Good balance of speed and performance

# Fast dev builds
[profile.dev]
debug = 0
strip = "debuginfo"
incremental = true
codegen-units = 16

[registries.crates-io]
protocol = "sparse"

[net]
git-fetch-with-cli = true
jobs = 16
EOF

# Set ownership
RUN chown -R runner:runner /home/runner

# Environment variables for PVC-based caching
# These will be overridden by the workflow to point to PVC mounts
ENV CARGO_HOME="/cache/cargo"
ENV RUSTUP_HOME="/cache/rustup"
ENV SCCACHE_DIR="/cache/sccache"
ENV SCCACHE_CACHE_SIZE="2G"
ENV RUSTC_WRAPPER="sccache"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD which rustc && which cargo && which sccache || exit 1

# Use the default entrypoint from summerwind image