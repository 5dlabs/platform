# Pre-staged Rust builder image for fast CI builds
FROM ubuntu:22.04

# Avoid interactive prompts during build
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Modern tools
    curl \
    wget \
    git \
    unzip \
    # Clang for mold linker
    clang-15 \
    lld-15 \
    # Docker for DinD
    docker.io \
    # Additional tools
    jq \
    ca-certificates \
    gnupg \
    software-properties-common \
    && rm -rf /var/lib/apt/lists/*

# Install mold linker (fastest linker for Rust)
RUN wget -qO- https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz | tar xz \
    && cp mold-2.4.0-x86_64-linux/bin/mold /usr/local/bin/ \
    && chmod +x /usr/local/bin/mold \
    && rm -rf mold-2.4.0-x86_64-linux

# Install kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" \
    && chmod +x kubectl \
    && mv kubectl /usr/local/bin/

# Install Helm
RUN curl -fsSL -o get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 \
    && chmod 700 get_helm.sh \
    && ./get_helm.sh \
    && rm get_helm.sh

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install -y gh \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

# Install Rust as app user
USER app
WORKDIR /app

# Install rustup and Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && echo 'source ~/.cargo/env' >> ~/.bashrc

# Add Rust to PATH
ENV PATH="/app/.cargo/bin:${PATH}"

# Install additional Rust tools
RUN ~/.cargo/bin/rustup component add clippy rustfmt \
    && ~/.cargo/bin/cargo install \
        # Build acceleration tools
        sccache \
        cargo-chef \
        cargo-nextest \
        # Utility tools
        cargo-audit \
        cargo-outdated \
        cargo-tree \
        cargo-watch

# Pre-warm common dependencies by building a dummy project
RUN ~/.cargo/bin/cargo new --bin dummy-project \
    && cd dummy-project \
    && echo 'tokio = { version = "1.40", features = ["full"] }' >> Cargo.toml \
    && echo 'axum = "0.8"' >> Cargo.toml \
    && echo 'serde = { version = "1.0", features = ["derive"] }' >> Cargo.toml \
    && echo 'anyhow = "1.0"' >> Cargo.toml \
    && echo 'tracing = "0.1"' >> Cargo.toml \
    && ~/.cargo/bin/cargo build \
    && cd .. \
    && rm -rf dummy-project

# Switch back to root for runner setup
USER root

# Create cache directories
RUN mkdir -p /cache/{cargo,rustup,sccache} \
    && chown -R app:app /cache

# Set up default Cargo config with optimizations
RUN mkdir -p /app/.cargo && cat > /app/.cargo/config.toml << 'EOF'
[build]
jobs = 0  # Use all available cores

[target.x86_64-unknown-linux-gnu]
linker = "/usr/bin/clang-15"
rustflags = ["-C", "link-arg=--ld-path=/usr/local/bin/mold"]

[profile.dev]
debug = 0
strip = "debuginfo"
incremental = true

[profile.dev.package."*"]
opt-level = 3
debug = 2

[profile.dev.build-override]
opt-level = 3

[net]
git-fetch-with-cli = true

[cargo-new]
vcs = "none"
EOF

# Set ownership
RUN chown -R app:app /app

# Environment variables
ENV CARGO_HOME="/cache/cargo"
ENV RUSTUP_HOME="/cache/rustup"
ENV SCCACHE_DIR="/cache/sccache"
ENV SCCACHE_CACHE_SIZE="20G"
ENV RUSTC_WRAPPER="sccache"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD sccache --show-stats || exit 1

# Default command
CMD ["/bin/bash"]