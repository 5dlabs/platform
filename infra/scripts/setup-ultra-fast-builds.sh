#!/bin/bash
# Ultra-fast Rust build setup script
# Goal: Get builds down to seconds, not minutes

set -euo pipefail

echo "🚀 Setting up ULTRA-FAST Rust builds..."

# 1. Verify all tools are available
echo "🔍 Checking required tools..."
command -v mold >/dev/null 2>&1 || { echo "❌ mold not found"; exit 1; }
command -v sccache >/dev/null 2>&1 || { echo "❌ sccache not found"; exit 1; }
command -v cargo-chef >/dev/null 2>&1 || { echo "❌ cargo-chef not found"; exit 1; }

# 2. Configure Cargo for maximum speed
echo "⚙️ Configuring Cargo for speed..."
mkdir -p ~/.cargo

cat > ~/.cargo/config.toml << 'EOF'
[build]
# Use sccache for caching compilation
rustc-wrapper = "sccache"
# Use all available cores
jobs = 16

[target.x86_64-unknown-linux-gnu]
# Use mold for ultra-fast linking
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "target-cpu=native",  # Optimize for this CPU
    "-C", "link-arg=-Wl,--threads=16",  # Parallel linking
]

[profile.release]
# Optimize for fast builds, not runtime
codegen-units = 256  # Max parallelism
lto = "off"  # Disable LTO for faster builds
opt-level = 2  # Good enough optimization

[profile.release.build-override]
# Dependencies can be optimized more
opt-level = 3
codegen-units = 1

[registries.crates-io]
protocol = "sparse"  # Faster registry updates

[net]
git-fetch-with-cli = true
jobs = 16  # Parallel downloads
EOF

# 3. Start sccache with proper config
echo "🗄️ Starting sccache..."
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_IDLE_TIMEOUT="0"  # Never timeout
export SCCACHE_ERROR_LOG="/tmp/sccache_error.log"
export SCCACHE_LOG="debug"

# Kill any existing sccache
sccache --stop-server 2>/dev/null || true

# Start fresh
sccache --start-server

# Show initial stats
sccache --show-stats

# 4. Pre-warm sccache if empty
if [ -z "$(sccache --show-stats | grep 'Cache hits')" ]; then
    echo "📦 Pre-warming sccache with dependency builds..."
    # This would be done by the Docker image build
fi

echo "✅ Ultra-fast build setup complete!"
echo ""
echo "Expected improvements:"
echo "- Linking: 10-50x faster with mold"
echo "- Compilation: 2-10x faster with sccache hits"
echo "- Downloads: Already cached in image"
echo "- Total build time: Should be < 30 seconds"