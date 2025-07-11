#!/bin/bash
set -euo pipefail

# Setup script for Orchestrator MCP Server
# Builds and installs the enhanced MCP server with improved error handling and documentation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
INSTALL_DIR="$HOME/.local/bin"
MCP_CONFIG_FILE="$HOME/.cursor/mcp.json"
BUILD_ONLY=false
FORCE=false
TASKMASTER_ROOT=""

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

# Function to show usage
usage() {
    cat << EOF
🚀 Orchestrator MCP Server Setup

Usage: $0 [OPTIONS]

Build and install the enhanced Orchestrator MCP Server with improved error handling,
parameter validation, and comprehensive documentation.

OPTIONS:
    -h, --help                  Show this help message
    -d, --install-dir DIR       Installation directory (default: ~/.local/bin)
    -c, --config FILE           MCP config file to update (default: ~/.cursor/mcp.json)
    -t, --taskmaster-root DIR   Set TASKMASTER_ROOT environment variable
    -b, --build-only            Only build, don't install or configure
    -f, --force                 Force overwrite existing installation
    --system                    Install system-wide to /usr/local/bin (requires sudo)

FEATURES:
    ✅ Enhanced parameter validation with clear error messages
    ✅ Comprehensive tool documentation with examples
    ✅ Connectivity testing with ping tool
    ✅ Automatic environment detection
    ✅ Improved error handling and logging

EXAMPLES:
    $0                                          # Build and install to ~/.local/bin
    $0 --system                                 # Install system-wide
    $0 -t /path/to/project                      # Set specific TASKMASTER_ROOT
    $0 -d ~/bin -c ~/.cursor/custom-mcp.json    # Custom install location

AFTER INSTALLATION:
    • Restart Cursor to pick up the new MCP server
    • Test with: init_docs({}) in Cursor
    • Use ping() tool to verify connectivity
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -d|--install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -c|--config)
            MCP_CONFIG_FILE="$2"
            shift 2
            ;;
        -t|--taskmaster-root)
            TASKMASTER_ROOT="$2"
            shift 2
            ;;
        -b|--build-only)
            BUILD_ONLY=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --system)
            INSTALL_DIR="/usr/local/bin"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check prerequisites
print_header "🔍 Checking prerequisites..."

# Check if we're in the right directory
if [[ ! -d "orchestrator/mcp-server" ]]; then
    print_error "Please run this script from the platform repository root"
    print_error "Current directory: $(pwd)"
    exit 1
fi

# Check Rust/Cargo
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Check if install directory is writable or needs sudo
if [[ "$INSTALL_DIR" == "/usr/local/bin" ]] && [[ "$BUILD_ONLY" != true ]]; then
    if [[ $EUID -ne 0 ]] && ! sudo -n true 2>/dev/null; then
        print_warning "System-wide installation requires sudo privileges"
    fi
fi

print_status "✅ Prerequisites check passed!"

# Build the MCP server
print_header "🔨 Building MCP server..."
cd orchestrator

print_status "Running cargo build for mcp-server..."
cargo build --bin mcp-server --release || {
    print_error "Build failed!"
    exit 1
}

BINARY_PATH="target/release/mcp-server"
if [[ ! -f "$BINARY_PATH" ]]; then
    print_error "Binary not found at $BINARY_PATH"
    exit 1
fi

print_status "✅ Build completed successfully!"

# Get binary info
BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
print_status "Binary size: $BINARY_SIZE"

# Exit if build-only
if [[ "$BUILD_ONLY" == true ]]; then
    print_status "🎯 Build-only mode: Binary available at orchestrator/$BINARY_PATH"
    exit 0
fi

# Install the binary
print_header "📦 Installing MCP server..."

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR" || {
    if [[ "$INSTALL_DIR" == "/usr/local/bin" ]]; then
        sudo mkdir -p "$INSTALL_DIR"
    else
        print_error "Cannot create install directory: $INSTALL_DIR"
        exit 1
    fi
}

DEST_PATH="$INSTALL_DIR/orchestrator-mcp-server"

# Check if already exists
if [[ -f "$DEST_PATH" ]] && [[ "$FORCE" != true ]]; then
    print_warning "MCP server already exists at $DEST_PATH"
    read -p "Overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_status "Installation cancelled"
        exit 0
    fi
fi

# Copy binary
if [[ "$INSTALL_DIR" == "/usr/local/bin" ]]; then
    sudo cp "$BINARY_PATH" "$DEST_PATH" || {
        print_error "Failed to install binary to $DEST_PATH"
        exit 1
    }
    sudo chmod +x "$DEST_PATH"
else
    cp "$BINARY_PATH" "$DEST_PATH" || {
        print_error "Failed to install binary to $DEST_PATH"
        exit 1
    }
    chmod +x "$DEST_PATH"
fi

print_status "✅ Binary installed to $DEST_PATH"

# Update MCP configuration
if [[ -f "$MCP_CONFIG_FILE" ]]; then
    print_header "⚙️ Updating MCP configuration..."

    # Backup existing config
    BACKUP_FILE="${MCP_CONFIG_FILE}.backup.$(date +%Y%m%d-%H%M%S)"
    cp "$MCP_CONFIG_FILE" "$BACKUP_FILE"
    print_status "Backup created: $BACKUP_FILE"

    # Prepare environment variables
    ENV_VARS=""
    if [[ -n "$TASKMASTER_ROOT" ]]; then
        ENV_VARS="\"TASKMASTER_ROOT\": \"$TASKMASTER_ROOT\""
    else
        # Try to auto-detect from example directory
        EXAMPLE_DIR="$(pwd)/example"
        if [[ -d "$EXAMPLE_DIR/.taskmaster" ]]; then
            ENV_VARS="\"TASKMASTER_ROOT\": \"$EXAMPLE_DIR\""
            print_status "Auto-detected TASKMASTER_ROOT: $EXAMPLE_DIR"
        fi
    fi

    # Create new orchestrator MCP server config
    cat > /tmp/orchestrator-mcp.json << EOF
{
    "command": "$DEST_PATH",
    "args": []$(if [[ -n "$ENV_VARS" ]]; then echo ",
    \"env\": {
        $ENV_VARS
    }"; fi)
}
EOF

    # Use jq to update the config if available, otherwise warn user
    if command -v jq &> /dev/null; then
        jq --slurpfile new /tmp/orchestrator-mcp.json '.mcpServers.orchestrator = $new[0]' "$MCP_CONFIG_FILE" > "${MCP_CONFIG_FILE}.tmp" && mv "${MCP_CONFIG_FILE}.tmp" "$MCP_CONFIG_FILE"
        print_status "✅ MCP configuration updated"
    else
        print_warning "jq not found - please manually update $MCP_CONFIG_FILE"
        print_status "Add this to your mcpServers section:"
        cat /tmp/orchestrator-mcp.json
    fi

    rm -f /tmp/orchestrator-mcp.json
else
    print_warning "MCP config file not found: $MCP_CONFIG_FILE"
    print_status "Please create the config manually or specify with -c option"
fi

# Return to original directory
cd ..

# Show completion status
print_header "🎉 Installation completed!"

echo ""
print_status "📋 Installation Summary:"
print_status "  Binary: $DEST_PATH"
print_status "  Size: $BINARY_SIZE"
if [[ -f "$MCP_CONFIG_FILE" ]]; then
    print_status "  Config: $MCP_CONFIG_FILE"
fi
if [[ -n "$TASKMASTER_ROOT" ]]; then
    print_status "  TASKMASTER_ROOT: $TASKMASTER_ROOT"
fi

echo ""
print_status "🚀 Enhanced Features Available:"
print_status "  ✅ Parameter validation with clear error messages"
print_status "  ✅ Comprehensive documentation with examples"
print_status "  ✅ Connectivity testing with ping() tool"
print_status "  ✅ Automatic environment detection"
print_status "  ✅ Enhanced logging and error context"

echo ""
print_status "🔄 Next Steps:"
print_status "  1. Restart Cursor to load the new MCP server"
print_status "  2. Test connectivity: ping()"
print_status "  3. Generate docs: init_docs({})"
print_status "  4. Try examples: init_docs({model: 'opus', task_id: 5})"

echo ""
print_status "📖 Usage Examples:"
print_status "  init_docs({})                    # Generate docs for all tasks"
print_status "  init_docs({model: 'opus'})       # Use specific model"
print_status "  init_docs({task_id: 5})          # Generate docs for task 5 only"
print_status "  init_docs({force: true})         # Force overwrite existing docs"
print_status "  ping()                           # Test MCP connectivity"

if [[ "$INSTALL_DIR" != "/usr/local/bin" ]] && [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    print_warning "⚠️  $INSTALL_DIR is not in your PATH"
    print_status "Add this to your shell profile to use the binary directly:"
    print_status "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi