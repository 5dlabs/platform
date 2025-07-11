#!/bin/bash
set -euo pipefail

# Orchestrator MCP Server Installation Script
# This script downloads and installs the latest MCP server binary from GitHub releases

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default values
REPO="5dlabs/agent-platform"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="orchestrator-mcp-server"
VERSION="latest"
FORCE=false
UPDATE_CONFIG=true
UPDATE_PATH=true
MCP_CONFIG_FILE="$HOME/.cursor/mcp.json"

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}${BOLD}🚀 Orchestrator MCP Server Installer${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}${BOLD}✅ $1${NC}"
}

# Show usage
usage() {
    print_header
    cat << EOF
Download and install the Orchestrator MCP Server from GitHub releases.

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -v, --version VERSION   Install specific version (default: latest)
    -d, --dir DIR          Installation directory (default: ~/.local/bin)
    -f, --force            Force reinstall even if already installed
    --no-config            Skip MCP configuration update
    --no-path              Skip updating PATH in shell profile
    --config FILE          Custom MCP config file path
    --repo REPO            GitHub repository (default: 5dlabs/agent-platform)

EXAMPLES:
    $0                                    # Install latest version
    $0 --version v1.2.3                  # Install specific version
    $0 --dir /usr/local/bin --force       # System install with force
    $0 --no-config                        # Skip config update
    $0 --no-path                          # Skip PATH update

The installer will:
  ✅ Auto-detect your platform (Linux, macOS, Windows)
  ✅ Download the appropriate binary
  ✅ Verify checksums for security
  ✅ Install to specified directory
  ✅ Update your .cursor/mcp.json configuration
  ✅ Add install directory to PATH (bash/zsh)
  ✅ Provide next steps for usage

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --no-config)
            UPDATE_CONFIG=false
            shift
            ;;
        --config)
            MCP_CONFIG_FILE="$2"
            shift 2
            ;;
        --repo)
            REPO="$2"
            shift 2
            ;;
        --no-path)
            UPDATE_PATH=false
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Detect platform and architecture
detect_platform() {
    local os arch

    case "$OSTYPE" in
        linux-gnu*)
            os="linux"
            ;;
        darwin*)
            os="macos"
            ;;
        msys*|mingw*|cygwin*)
            os="windows"
            ;;
        *)
            print_error "Unsupported operating system: $OSTYPE"
            print_info "Supported platforms: Linux, macOS, Windows"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            if [[ "$os" == "macos" ]]; then
                arch="aarch64"
            else
                print_error "ARM64 architecture only supported on macOS"
                exit 1
            fi
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            print_info "Supported architectures: x86_64, aarch64 (macOS only)"
            exit 1
            ;;
    esac

    if [[ "$os" == "windows" ]]; then
        echo "orchestrator-mcp-server-${os}-${arch}.exe"
    else
        echo "orchestrator-mcp-server-${os}-${arch}"
    fi
}

# Get latest release version from GitHub API
get_latest_version() {
    print_info "Fetching latest release information..."

    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases" | \
                  grep -o '"tag_name": *"mcp-v[^"]*"' | \
                  head -1 | \
                  sed 's/"tag_name": *"mcp-v\([^"]*\)"/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases" | \
                  grep -o '"tag_name": *"mcp-v[^"]*"' | \
                  head -1 | \
                  sed 's/"tag_name": *"mcp-v\([^"]*\)"/\1/')
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [[ -z "$VERSION" ]]; then
        print_error "Could not determine latest version"
        print_info "You can specify a version manually with --version"
        exit 1
    fi

    print_info "Latest version: $VERSION"
}

# Download file with curl or wget
download_file() {
    local url="$1"
    local output="$2"

    print_info "Downloading: $url"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$output" "$url"
    else
        print_error "Neither curl nor wget found"
        exit 1
    fi
}

# Verify checksum
verify_checksum() {
    local binary_file="$1"
    local checksum_file="$2"
    local expected_filename="$3"

    if [[ ! -f "$checksum_file" ]]; then
        print_warning "Checksum file not found, skipping verification"
        return 0
    fi

    print_info "Verifying checksum..."

    # Create a temporary directory for verification
    local verify_dir
    verify_dir=$(mktemp -d)

    # Copy binary to verification directory with expected filename
    cp "$binary_file" "$verify_dir/$expected_filename"
    cp "$checksum_file" "$verify_dir/"

    # Change to verification directory and verify
    local verification_result=0
    if command -v shasum >/dev/null 2>&1; then
        if (cd "$verify_dir" && shasum -a 256 -c "$(basename "$checksum_file")" >/dev/null 2>&1); then
            print_success "Checksum verification passed"
        else
            print_error "Checksum verification failed"
            verification_result=1
        fi
    elif command -v sha256sum >/dev/null 2>&1; then
        if (cd "$verify_dir" && sha256sum -c "$(basename "$checksum_file")" >/dev/null 2>&1); then
            print_success "Checksum verification passed"
        else
            print_error "Checksum verification failed"
            verification_result=1
        fi
    else
        print_warning "No checksum utility found, skipping verification"
    fi

    # Cleanup verification directory
    rm -rf "$verify_dir"

    if [[ $verification_result -ne 0 ]]; then
        exit 1
    fi
}

# Update MCP configuration
update_mcp_config() {
    local binary_path="$1"

    if [[ "$UPDATE_CONFIG" != true ]]; then
        return 0
    fi

    print_info "Updating MCP configuration..."

    # Create config directory if it doesn't exist
    mkdir -p "$(dirname "$MCP_CONFIG_FILE")"

    # Create backup if file exists
    if [[ -f "$MCP_CONFIG_FILE" ]]; then
        local backup_file="${MCP_CONFIG_FILE}.backup.$(date +%Y%m%d-%H%M%S)"
        cp "$MCP_CONFIG_FILE" "$backup_file"
        print_info "Backup created: $backup_file"
    fi

    # Try to auto-detect TASKMASTER_ROOT
    local taskmaster_root=""
    if [[ -d "$(pwd)/example/.taskmaster" ]]; then
        taskmaster_root="$(pwd)/example"
    elif [[ -d "$(pwd)/.taskmaster" ]]; then
        taskmaster_root="$(pwd)"
    fi

    # Update configuration with jq if available
    if command -v jq >/dev/null 2>&1; then
        if [[ ! -f "$MCP_CONFIG_FILE" ]]; then
            echo '{"mcpServers": {}}' > "$MCP_CONFIG_FILE"
        fi

        local config="{\"command\": \"$binary_path\", \"args\": []"
        if [[ -n "$taskmaster_root" ]]; then
            config="$config, \"env\": {\"TASKMASTER_ROOT\": \"$taskmaster_root\"}"
        fi
        config="$config}"

        echo "$config" | jq '.' > /tmp/orchestrator-config.json
        jq --slurpfile new /tmp/orchestrator-config.json '.mcpServers.orchestrator = $new[0]' "$MCP_CONFIG_FILE" > "${MCP_CONFIG_FILE}.tmp"
        mv "${MCP_CONFIG_FILE}.tmp" "$MCP_CONFIG_FILE"
        rm -f /tmp/orchestrator-config.json

        print_success "MCP configuration updated"
    else
        print_warning "jq not found - you'll need to manually update your MCP configuration"
        echo ""
        print_info "Add this to your $MCP_CONFIG_FILE:"
        echo '{'
        echo '  "mcpServers": {'
        echo '    "orchestrator": {'
        echo "      \"command\": \"$binary_path\","
        echo '      "args": []'
        if [[ -n "$taskmaster_root" ]]; then
            echo '      "env": {'
            echo "        \"TASKMASTER_ROOT\": \"$taskmaster_root\""
            echo '      }'
        fi
        echo '    }'
        echo '  }'
        echo '}'
    fi
}

# Update shell profile to include install directory in PATH
update_shell_path() {
    local install_dir="$1"

    # Skip if directory is already in PATH or is a system directory
    if [[ ":$PATH:" == *":$install_dir:"* ]] || [[ "$install_dir" == "/usr/local/bin" ]]; then
        return 0
    fi

    # Detect shell and appropriate profile file
    local shell_name profile_file
    shell_name=$(basename "$SHELL")

    case "$shell_name" in
        bash)
            # Check for different bash profile files in order of preference
            if [[ -f "$HOME/.bash_profile" ]]; then
                profile_file="$HOME/.bash_profile"
            elif [[ -f "$HOME/.bashrc" ]]; then
                profile_file="$HOME/.bashrc"
            else
                # Create .bashrc if neither exists
                profile_file="$HOME/.bashrc"
            fi
            ;;
        zsh)
            profile_file="$HOME/.zshrc"
            ;;
        *)
            print_warning "Unsupported shell: $shell_name"
            print_info "Supported shells: bash, zsh"
            print_info "Please manually add $install_dir to your PATH:"
            print_info "  export PATH=\"\$PATH:$install_dir\""
            return 0
            ;;
    esac

    # Check if PATH export already exists in profile
    if [[ -f "$profile_file" ]] && grep -q "export PATH.*$install_dir" "$profile_file" 2>/dev/null; then
        print_info "PATH already configured in $profile_file"
        return 0
    fi

    print_warning "⚠️  $install_dir is not in your PATH"
    echo ""
    print_info "To use the binary directly from anywhere, we can add it to your PATH."

    # Ask user if they want to update PATH
    if [[ -t 0 ]]; then  # Only prompt if running interactively
        echo -n "Add $install_dir to PATH in $profile_file? (y/N): "
        read -r response

        if [[ "$response" =~ ^[Yy]$ ]]; then
            # Add PATH export to profile file
            echo "" >> "$profile_file"
            echo "# Added by orchestrator-mcp-server installer" >> "$profile_file"
            echo "export PATH=\"\$PATH:$install_dir\"" >> "$profile_file"

            print_success "✅ PATH updated in $profile_file"
            echo ""
            print_info "To use the new PATH in this session, run:"
            print_info "  source $profile_file"
            print_info "Or restart your terminal."

            return 0
        else
            print_info "Skipped PATH update."
        fi
    else
        print_info "Running non-interactively, skipping PATH update."
    fi

    echo ""
    print_info "To manually add to PATH, add this line to $profile_file:"
    print_info "  export PATH=\"\$PATH:$install_dir\""
}

# Main installation function
main() {
    print_header

    # Check prerequisites
    if [[ ! "$INSTALL_DIR" =~ ^/ ]] && [[ ! "$INSTALL_DIR" =~ ^\$HOME ]] && [[ ! "$INSTALL_DIR" =~ ^~ ]]; then
        # Convert relative path to absolute
        INSTALL_DIR="$(pwd)/$INSTALL_DIR"
    fi

    # Expand ~ to home directory
    INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"
    INSTALL_DIR="${INSTALL_DIR/#\$HOME/$HOME}"

    print_info "Installation directory: $INSTALL_DIR"

    # Detect platform
    local binary_filename
    binary_filename=$(detect_platform)
    print_info "Detected platform: $binary_filename"

    # Get version
    if [[ "$VERSION" == "latest" ]]; then
        get_latest_version
    fi

    # Prepare URLs and paths
    local tag="mcp-v$VERSION"
    local base_url="https://github.com/$REPO/releases/download/$tag"
    local binary_url="$base_url/$binary_filename"
    local checksum_url="$base_url/${binary_filename}.sha256"

    local dest_path="$INSTALL_DIR/$BINARY_NAME"
    local temp_dir
    temp_dir=$(mktemp -d)
    local temp_binary="$temp_dir/$binary_filename"
    local temp_checksum="$temp_dir/${binary_filename}.sha256"

    # Check if already installed
    if [[ -f "$dest_path" ]] && [[ "$FORCE" != true ]]; then
        print_warning "MCP server already installed at $dest_path"
        echo "Use --force to reinstall or --help for options"
        exit 0
    fi

    # Create installation directory
    mkdir -p "$INSTALL_DIR"

    # Download binary and checksum
    download_file "$binary_url" "$temp_binary"
    download_file "$checksum_url" "$temp_checksum" || true

    # Verify checksum
    verify_checksum "$temp_binary" "$temp_checksum" "$binary_filename"

    # Install binary
    print_info "Installing binary to $dest_path"
    mv "$temp_binary" "$dest_path"
    chmod +x "$dest_path"

    # Update MCP configuration
    update_mcp_config "$dest_path"

    # Update shell PATH
    if [[ "$UPDATE_PATH" == true ]]; then
        update_shell_path "$INSTALL_DIR"
    fi

    # Cleanup
    rm -rf "$temp_dir"

    # Success message
    print_success "Installation completed successfully!"
    echo ""
    print_info "📋 Installation Summary:"
    print_info "  Binary: $dest_path"
    print_info "  Version: $VERSION"
    if [[ "$UPDATE_CONFIG" == true ]]; then
        print_info "  Config: $MCP_CONFIG_FILE"
    fi

    echo ""
    print_info "🔄 Next Steps:"
    print_info "  1. Restart Cursor to load the new MCP server"
    print_info "  2. Test connectivity: ping()"
    print_info "  3. Generate docs: init_docs({})"

    echo ""
    print_info "📖 Usage Examples:"
    print_info "  init_docs({})                    # Generate docs for all tasks"
    print_info "  init_docs({model: 'opus'})       # Use specific model"
    print_info "  init_docs({task_id: 5})          # Generate docs for task 5 only"
    print_info "  ping()                           # Test MCP connectivity"

    echo ""
    print_success "🎉 Ready to use the enhanced MCP server!"
}

# Run main function
main "$@"