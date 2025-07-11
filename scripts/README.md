# Orchestrator Scripts

## 🚀 Quick Start: MCP Server Installation

**One-line installation for the enhanced Orchestrator MCP Server:**

```bash
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh | bash
```

This will:
- ✅ Auto-detect your platform (Linux, macOS, Windows)
- ✅ Download the latest pre-built binary
- ✅ Install to `~/.local/bin`
- ✅ Configure your `.cursor/mcp.json`
- ✅ Ready to use in seconds!

**Then restart Cursor and test:** `ping()` and `init_docs({})`

---

## MCP Server Deployment

### install-mcp-server.sh (Recommended)

**🚀 One-line installation using pre-built binaries from GitHub releases.**

#### Quick Install
```bash
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh | bash
```

#### What it does

- ✅ **Auto-detects your platform** (Linux, macOS x64/ARM64, Windows)
- ✅ **Downloads appropriate binary** from GitHub releases
- ✅ **Verifies checksums** for security
- ✅ **Installs to ~/.local/bin** (or custom directory)
- ✅ **Updates MCP configuration** automatically
- ✅ **No Rust toolchain required** - just download and run!

#### Usage Options

```bash
# Quick install (recommended)
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh | bash

# Install specific version
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh | bash -s -- --version v1.2.3

# System-wide install
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh | bash -s -- --dir /usr/local/bin

# Download script first for review
curl -fsSL https://raw.githubusercontent.com/5dlabs/agent-platform/main/scripts/install-mcp-server.sh -o install.sh
chmod +x install.sh
./install.sh --help
```

#### Available Options

```
OPTIONS:
    -h, --help              Show help message
    -v, --version VERSION   Install specific version (default: latest)
    -d, --dir DIR          Installation directory (default: ~/.local/bin)
    -f, --force            Force reinstall even if already installed
    --no-config            Skip MCP configuration update
    --config FILE          Custom MCP config file path
```

### deploy-mcp-server.sh

Quick deployment script with common scenarios - **now defaults to downloading pre-built binaries**.

#### Usage

```bash
# Download pre-built binary (default, fastest)
./scripts/deploy-mcp-server.sh

# Alternative scenarios
./scripts/deploy-mcp-server.sh download   # Download from GitHub releases (same as default)
./scripts/deploy-mcp-server.sh dev        # Build from source for development
./scripts/deploy-mcp-server.sh system     # System-wide install
./scripts/deploy-mcp-server.sh build      # Build only
```

### setup-mcp-server.sh (Build from Source)

Comprehensive setup script for building, installing, and configuring the enhanced Orchestrator MCP Server from source code.

#### Features

- ✅ Enhanced parameter validation with clear error messages
- ✅ Comprehensive tool documentation with examples
- ✅ Connectivity testing with ping tool
- ✅ Automatic environment detection
- ✅ Improved error handling and logging

#### What it does

1. **Prerequisites Check** - Validates Rust/Cargo installation and directory structure
2. **Build MCP Server** - Compiles the enhanced MCP server binary with release optimizations
3. **Install Binary** - Copies to specified location (default: `~/.local/bin`)
4. **Update Configuration** - Automatically updates MCP configuration in Cursor
5. **Environment Setup** - Configures TASKMASTER_ROOT and other environment variables

#### Prerequisites

- Rust and Cargo installed ([rustup.rs](https://rustup.rs/))
- Write access to installation directory
- For system install: sudo privileges

#### Usage

```bash
# Development install with auto-detection
./scripts/setup-mcp-server.sh

# Custom installation directory
./scripts/setup-mcp-server.sh -d ~/bin

# System-wide installation
./scripts/setup-mcp-server.sh --system

# Specify custom TASKMASTER_ROOT
./scripts/setup-mcp-server.sh -t /path/to/project

# Build only (no installation)
./scripts/setup-mcp-server.sh --build-only

# Custom MCP config file
./scripts/setup-mcp-server.sh -c ~/.cursor/custom-mcp.json

# Force overwrite existing installation
./scripts/setup-mcp-server.sh --force
```

#### Available Options

```
OPTIONS:
    -h, --help                  Show help message
    -d, --install-dir DIR       Installation directory (default: ~/.local/bin)
    -c, --config FILE           MCP config file to update (default: ~/.cursor/mcp.json)
    -t, --taskmaster-root DIR   Set TASKMASTER_ROOT environment variable
    -b, --build-only            Only build, don't install or configure
    -f, --force                 Force overwrite existing installation
    --system                    Install system-wide to /usr/local/bin (requires sudo)
```

#### Enhanced MCP Tools

After installation, the enhanced MCP server provides:

**init_docs tool with improved features:**
- Parameter validation with specific error messages
- Comprehensive documentation with usage examples
- Enhanced logging and error context
- Automatic environment detection

**New ping tool:**
- Test MCP server connectivity
- Validate environment configuration
- Check CLI availability
- Show current directory context

#### Usage Examples in Cursor

```javascript
// Generate docs for all tasks
init_docs({})

// Use specific model
init_docs({model: 'opus'})

// Generate docs for specific task
init_docs({task_id: 5})

// Force overwrite existing docs
init_docs({force: true})

// Test connectivity and configuration
ping()
```

#### Example Output

```
🔍 Checking prerequisites...
✅ Prerequisites check passed!

🔨 Building MCP server...
Running cargo build for mcp-server...
✅ Build completed successfully!
Binary size: 8.2M

📦 Installing MCP server...
✅ Binary installed to ~/.local/bin/orchestrator-mcp-server

⚙️ Updating MCP configuration...
Auto-detected TASKMASTER_ROOT: /path/to/platform/example
✅ MCP configuration updated

🎉 Installation completed!

📋 Installation Summary:
  Binary: ~/.local/bin/orchestrator-mcp-server
  Size: 8.2M
  Config: ~/.cursor/mcp.json
  TASKMASTER_ROOT: /path/to/platform/example

🚀 Enhanced Features Available:
  ✅ Parameter validation with clear error messages
  ✅ Comprehensive documentation with examples
  ✅ Connectivity testing with ping() tool
  ✅ Automatic environment detection
  ✅ Enhanced logging and error context

🔄 Next Steps:
  1. Restart Cursor to load the new MCP server
  2. Test connectivity: ping()
  3. Generate docs: init_docs({})
  4. Try examples: init_docs({model: 'opus', task_id: 5})
```

## clean-workspace-and-test.sh

A comprehensive script for testing the orchestrator with a completely clean environment.

### What it does

1. **Cleans the PVC completely** - Removes all data from the Claude workspace PVC
2. **Creates a fresh test repository** - Deletes and recreates a repository from the agent-template
3. **Waits for GitHub Actions** - Ensures any build workflows complete before proceeding
4. **Restarts the orchestrator** - Ensures the orchestrator has the latest configuration
5. **Submits a test task** - Creates task 9999 with test markdown files

### Prerequisites

- `kubectl` configured with access to the cluster
- `gh` (GitHub CLI) authenticated
- `orchestrator` CLI installed and in PATH
- Write access to the 5dlabs GitHub organization

### Usage

```bash
./scripts/clean-workspace-and-test.sh
```

The script will prompt for confirmation before proceeding with destructive operations.

### Configuration

Edit these variables at the top of the script to customize:

- `NAMESPACE`: Kubernetes namespace (default: "orchestrator")
- `PVC_NAME`: Name of the PVC to clean (default: "claude-workspace-pvc")
- `WORKER_NODE`: Node where PVC is mounted (default: "telemetry-worker-1")
- `TEST_REPO_NAME`: Name for the test repository (default: "todo-api-test")
- `GITHUB_ORG`: GitHub organization (default: "5dlabs")
- `GITHUB_USER`: GitHub user for authentication (default: "swe-1-5dlabs")
- `TASK_ID`: Task ID to submit (default: "9999")

### Safety Features

- Prerequisites check before running
- Confirmation prompt showing what will be deleted
- Job cleanup after PVC cleaning
- Error handling for each step
- Colored output for clarity

### Example Output

```
=== Claude Workspace Clean Test Script ===
Checking prerequisites...
Prerequisites check passed!

WARNING: This will:
 - Delete ALL data in PVC claude-workspace-pvc
 - Delete and recreate repository todo-api-test
 - Restart the orchestrator deployment

Are you sure you want to continue? (yes/no): yes

Step 1: Cleaning PVC claude-workspace-pvc...
job.batch/clean-pvc-1234567890 created
Waiting for cleaning job to complete...
job.batch/clean-pvc-1234567890 condition met
Cleaning job output:
Cleaning workspace...
Current contents:
...
Workspace cleaned

Step 2: Creating fresh repository from template...
✓ Deleted repository 5dlabs/todo-api-test
✓ Created repository 5dlabs/todo-api-test

Step 3: Checking for GitHub Actions...
Build completed successfully!

Step 4: Restarting orchestrator...
deployment.apps/orchestrator restarted
deployment "orchestrator" successfully rolled out

Step 5: Submitting test task...
Task submitted successfully
Task ID: abc123-def456-...

=== Test setup complete! ===

Summary:
 - PVC cleaned: claude-workspace-pvc
 - Repository created: 5dlabs/todo-api-test
 - Task submitted: 9999
 - TaskRun ID: abc123-def456-...

Next steps:
1. Monitor the task with: kubectl logs -n orchestrator -l task-id=9999 -f
2. Check task status with: orchestrator task status abc123-def456-...
3. View the agent workspace: kubectl exec -it -n orchestrator <pod-name> -- bash
4. Watch the job: kubectl get jobs -n orchestrator -w

Repository URL: https://github.com/5dlabs/todo-api-test
```