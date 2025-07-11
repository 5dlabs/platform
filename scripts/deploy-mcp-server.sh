#!/bin/bash
set -euo pipefail

# Quick deploy script for Orchestrator MCP Server
# Provides common deployment scenarios with sensible defaults

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}🚀 Quick MCP Server Deploy${NC}"
    echo ""
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# Show usage
usage() {
    print_header
    cat << EOF
Common deployment scenarios for the enhanced Orchestrator MCP Server.

USAGE:
    $0 [SCENARIO]

SCENARIOS:
    download    Download pre-built binary from GitHub (default, fastest)
    dev         Build and install for development
    local       Install to ~/.local/bin (same as dev)
    system      Install system-wide to /usr/local/bin
    build       Build only, don't install
    help        Show detailed setup options

EXAMPLES:
    $0              # Download pre-built binary (recommended)
    $0 download     # Download pre-built binary from GitHub releases
    $0 dev          # Build from source for development
    $0 system       # System-wide install (requires sudo)
    $0 build        # Build only for testing

DOWNLOAD vs BUILD:
    download: ⚡ Fast installation using pre-built GitHub release binaries
    dev/build: 🔨 Build from source (requires Rust toolchain)

For advanced options, use: ./scripts/setup-mcp-server.sh --help
EOF
}

# Get scenario
SCENARIO="${1:-download}"

case $SCENARIO in
    download)
        print_header
        print_status "⚡ Downloading pre-built binary from GitHub releases"
        print_status "This is the fastest installation method!"
        exec ./scripts/install-mcp-server.sh
        ;;
    dev|local)
        print_header
        print_status "🛠️  Development install to ~/.local/bin"
        print_status "Auto-detecting TASKMASTER_ROOT from example directory..."
        exec ./scripts/setup-mcp-server.sh
        ;;
    system)
        print_header
        print_status "🌐 System-wide install to /usr/local/bin"
        print_status "This will require sudo privileges..."
        exec ./scripts/setup-mcp-server.sh --system
        ;;
    build)
        print_header
        print_status "🔨 Build only (no installation)"
        exec ./scripts/setup-mcp-server.sh --build-only
        ;;
    help)
        print_header
        print_status "📖 Showing detailed setup options..."
        echo ""
        exec ./scripts/setup-mcp-server.sh --help
        ;;
    *)
        echo "❌ Unknown scenario: $SCENARIO"
        echo ""
        usage
        exit 1
        ;;
esac