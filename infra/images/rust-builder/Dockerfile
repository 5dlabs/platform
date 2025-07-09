# Pre-staged Rust builder image for fast CI builds
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

# Install mold linker (fastest linker for Rust) - UPDATED VERSION
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    wget -qO mold.tar.gz https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz && \
    tar xz -f mold.tar.gz && \
    cp mold-2.4.0-x86_64-linux/bin/mold /usr/local/bin/ && \
    chmod +x /usr/local/bin/mold

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

# Install Rust as runner user (with caching)
RUN --mount=type=cache,target=/tmp/rustup,uid=1001,gid=1001 \
    RUSTUP_HOME=/tmp/rustup curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    echo 'source ~/.cargo/env' >> ~/.bashrc

# Add Rust to PATH
ENV PATH="/home/runner/.cargo/bin:${PATH}"

# Create cargo directories with proper ownership first
RUN mkdir -p /home/runner/.cargo/registry /home/runner/.cargo/git && \
    chown -R runner:runner /home/runner/.cargo

# Install additional Rust tools (with cargo cache)
RUN --mount=type=cache,target=/tmp/cargo-registry,uid=1001,gid=1001 \
    --mount=type=cache,target=/tmp/cargo-git,uid=1001,gid=1001 \
    # Link cache directories
    ln -sf /tmp/cargo-registry /home/runner/.cargo/registry && \
    ln -sf /tmp/cargo-git /home/runner/.cargo/git && \
    # Install components and tools
    ~/.cargo/bin/rustup component add clippy rustfmt && \
    ~/.cargo/bin/cargo install \
        # Build acceleration tools
        sccache \
        cargo-chef \
        cargo-nextest \
        # Utility tools
        cargo-audit \
        cargo-outdated \
        cargo-tree \
        cargo-watch

# === DEPENDENCY PRE-WARMING USING CARGO-CHEF ===
# This is the industry-standard approach for Docker layer caching of Rust dependencies

# Create a minimal project for dependency pre-warming
RUN mkdir -p /tmp/warmup-project
WORKDIR /tmp/warmup-project

# Create Cargo.toml with EXACT versions from orchestrator Cargo.lock
RUN cat > Cargo.toml << 'EOF'
[package]
name = "warmup"
version = "0.1.0"
edition = "2021"

[dependencies]
# EXACT versions from orchestrator/Cargo.lock to ensure cache hits
tokio = { version = "=1.45.1", features = ["full"] }
axum = "=0.8.4"
tower = "=0.5.2"
tower-http = "=0.6.6"
hyper = "=1.6.0"
serde = { version = "=1.0.219", features = ["derive"] }
serde_json = "=1.0.140"
anyhow = "=1.0.98"
thiserror = "=2.0.12"
tracing = "=0.1.41"
tracing-subscriber = "=0.3.19"
reqwest = { version = "=0.12.22", features = ["json"] }
uuid = { version = "=1.17.0", features = ["v4"] }
chrono = { version = "=0.4.41", features = ["serde"] }
kube = "=0.93.1"
k8s-openapi = { version = "=0.22.0", features = ["v1_30"] }
handlebars = "=6.3.2"
opentelemetry = "=0.30.0"
opentelemetry_sdk = "=0.30.0"
opentelemetry-otlp = "=0.17.0"
tracing-opentelemetry = "=0.31.0"
EOF

# Create minimal main.rs
RUN mkdir -p src && echo 'fn main() { println!("warmup"); }' > src/main.rs

# Pre-warm dependencies directly to the final cache location (NO cache mounts)
# This ensures dependencies are baked into the image layers
ENV CARGO_HOME="/home/runner/.cargo"
RUN ~/.cargo/bin/cargo chef prepare --recipe-path recipe.json && \
    ~/.cargo/bin/cargo chef cook --release --recipe-path recipe.json

# Build the warmup project to ensure all deps are compiled and cached
RUN ~/.cargo/bin/cargo build --release

# Clean up warmup project but KEEP the cargo cache
WORKDIR /home/runner
RUN rm -rf /tmp/warmup-project

# Switch back to root for final setup
USER root

# Create sccache directory
RUN mkdir -p /home/runner/.cache/sccache && \
    chown -R runner:runner /home/runner/.cache

# Set up ULTRA-OPTIMIZED Cargo config
RUN mkdir -p /home/runner/.cargo && cat > /home/runner/.cargo/config.toml << 'EOF'
[build]
rustc-wrapper = "sccache"
jobs = 16  # Use all cores

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "link-arg=-Wl,--threads=16",  # Parallel linking with mold
    "-C", "target-cpu=x86-64-v2",  # Modern baseline CPU
]

# ULTRA-FAST release profile
[profile.release]
codegen-units = 256  # Maximum parallelism
lto = false  # Disable LTO for faster builds
debug = false
strip = true
opt-level = 2  # Good enough, much faster than 3

[profile.release.build-override]
opt-level = 3
codegen-units = 1

# Fast dev builds too
[profile.dev]
debug = 0
strip = "debuginfo"
incremental = true
codegen-units = 256

[profile.dev.package."*"]
opt-level = 3

[profile.dev.build-override]
opt-level = 3

[registries.crates-io]
protocol = "sparse"

[net]
git-fetch-with-cli = true
jobs = 16

[cargo-new]
vcs = "none"
EOF

# Set ownership
RUN chown -R runner:runner /home/runner

# Environment variables (use pre-warmed locations, not /cache PVCs)
ENV CARGO_HOME="/home/runner/.cargo"
ENV RUSTUP_HOME="/home/runner/.rustup"
ENV SCCACHE_DIR="/home/runner/.cache/sccache"
ENV SCCACHE_CACHE_SIZE="2G"
ENV RUSTC_WRAPPER="sccache"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD sccache --show-stats || exit 1

# Use the default entrypoint from summerwind image