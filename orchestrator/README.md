# Orchestrator Service

A unified orchestration service for managing AI-powered development tasks.

## Task Types Overview

The orchestrator handles two distinct types of tasks, each with different workflows and requirements:

### 📚 Docs Tasks (`docs`)

**Purpose**: Generate comprehensive documentation for Task Master projects

**Workflow**:
1. **Single Repository Setup** (Container Initialization):
   - Clone the repository containing `.taskmaster/tasks/tasks.json`
   - Source branch = the branch the command was executed from
   - Checkout to a randomized feature branch for documentation work

2. **Template Rendering** (Server-side):
   - Controller loads Handlebars templates from ConfigMap (`/claude-templates/docs/`)
   - Renders templates with task-specific data (repository URLs, branches, task IDs)
   - Generated files include `CLAUDE.md`, `container.sh`, `settings.json`, and prompt content

3. **Remote Execution** (Kubernetes):
   - Creates a `DocsRun` CRD in the orchestrator namespace
   - Spins up a documentation generation job with rendered templates
   - Claude agent reads task definitions and generates comprehensive documentation
   - Stop hook commits changes and creates PR back to the source branch

**CLI Command**: `orchestrator task docs --working-directory _projects/trader --model opus`

**Requirements**:
- Repository with `.taskmaster/` directory structure
- Valid `tasks.json` file with task definitions
- Git repository with proper authentication

---

### 🔧 Code Tasks (`code`)

**Purpose**: Execute actual code implementation for specific Task Master tasks

**Workflow**:
1. **Two Repository Setup** (Container Initialization):
   - **Platform Repository**: Cloned to access task specifications from `.taskmaster/docs/task-N/`
   - **Destination Repository**: Where code implementation happens and gets committed
   - Task files (`task.md`, `acceptance-criteria.md`, `architecture.md`) copied from platform to destination
   - Creates or resumes feature branch (`feature/task-N-implementation`)

2. **Template Rendering** (Server-side):
   - Controller loads Handlebars templates from ConfigMap (`/claude-templates/code/`)
   - Renders templates with task-specific data and tool configurations
   - Generated files include `CLAUDE.md`, `mcp.json`, `client-config.json`, `coding-guidelines.md`, `github-guidelines.md`
   - MCP configuration enables additional tools beyond Claude's built-in capabilities

3. **Remote Execution** (Kubernetes):
   - Creates a `CodeRun` CRD in the orchestrator namespace
   - Spins up an implementation job with Claude Code + MCP tools
   - Claude agent works in destination repository working directory
   - Implements functionality following copied task specifications
   - Uses `--resume` flag for subsequent runs to continue existing work
   - Commits implementation directly to feature branch

**CLI Command**: `orchestrator task code 1 --service trader --repo git@github.com:5dlabs/platform.git --github-user swe-1-5dlabs`

**Requirements**:
- Task ID and service specification
- Platform repository URL (for task documentation)
- Destination repository URL with proper SSH/HTTPS authentication
- Target branch and working directory specification

---

## Key Differences

| Aspect | Docs (`docs`) | Code (`code`) |
|--------|-----------|-----------|
| **Repository Setup** | 🔄 Single repository | ✅ Platform + Destination repos |
| **Template Rendering** | 📋 `docs/` templates via Handlebars | 📋 `code/` templates via Handlebars |
| **Task File Access** | ✅ Reads from same repo `.taskmaster/` | ✅ Copies task docs from platform to destination |
| **Tools Available** | 🔧 Claude Code tools only | 🔧 Claude Code + MCP tools |
| **MCP Configuration** | ❌ No MCP setup needed | ✅ `mcp.json` + `client-config.json` |
| **CRD Type** | `DocsRun` | `CodeRun` |
| **Purpose** | Generate documentation | Implement functionality |
| **Branch Strategy** | Randomized feature branch → PR to source | Feature branch (create/resume) |
| **Git Workflow** | Clone → checkout random branch → generate → PR | Clone both repos → work on feature branch |
| **Subsequent Runs** | Start fresh | Handled in CRD (retry/continuation field) |

---

## Authentication Patterns

Both task types support SSH and HTTPS authentication:

**SSH Pattern** (Preferred):
- Repository URL: `git@github.com:5dlabs/platform.git`
- Kubernetes Secret: `github-ssh-{username}` (e.g., `github-ssh-swe-1-5dlabs`)
- Used for: Private repositories, secure access

**HTTPS Pattern**:
- Repository URL: `https://github.com/5dlabs/platform.git`
- Kubernetes Secret: `github-pat-{username}` (e.g., `github-pat-swe-1-5dlabs`)
- Used for: Public repositories, token-based access

The orchestrator automatically detects the authentication method based on the repository URL format.

---

## Template Rendering Architecture

The orchestrator uses a sophisticated template system to generate agent-specific configurations:

### **Server-Side Template Processing**

```
📂 ConfigMap 1: Raw Templates
├── claude-templates/
│   ├── docs/CLAUDE.md.hbs
│   ├── code/CLAUDE.md.hbs
│   ├── code/mcp.json.hbs
│   ├── code/client-config.json.hbs
│   └── ...

🔄 Controller Processing
├── Load .hbs templates from ConfigMap
├── Apply Handlebars rendering with task data
└── Generate fully rendered files

📂 ConfigMap 2: Rendered Files
├── CLAUDE.md (task-specific memory)
├── settings.json (Claude tool permissions)
├── mcp.json (MCP server configuration)
├── client-config.json (dynamic tool selection)
└── container.sh (startup script)
```

### **Template Data Context**

```rust
// Example template data passed to Handlebars
{
  "task_id": 1,
  "service_name": "trader",
  "repository_url": "git@github.com:org/repo.git",
  "platform_repository_url": "git@github.com:5dlabs/platform.git",
  "github_user": "swe-1-5dlabs",
  "working_directory": "_projects/trader",
  "tool_config": "default", // "minimal", "default", "advanced"
  "local_tools": ["tool1", "tool2"], // MCP tools (not Claude tools)
  "remote_tools": ["rustdocs_query_rust_docs", "brave-search_brave_web_search"]
}
```

### **Agent Tool Configuration**

**Important Distinction**:
- **Claude Code Tools**: Built-in file operations (`read`, `write`, `edit`, `bash`, etc.)
- **MCP Tools**: External tools via MCP servers (`rustdocs_query_rust_docs`, `memory_create_entities`, etc.)

**Configuration Files**:
- `settings.json`: Claude tool permissions and model settings
- `mcp.json`: MCP server configuration and endpoints
- `client-config.json`: Dynamic MCP tool selection based on `tool_config` preset

**Tool Presets**:
```yaml
minimal: # Basic tools only
  remote_tools: []

default: # Standard development tools
  remote_tools: ["brave-search_brave_web_search", "memory_create_entities", "rustdocs_query_rust_docs"]

advanced: # Full toolset + filesystem server
  remote_tools: ["brave-search_brave_web_search", "memory_create_entities", "rustdocs_query_rust_docs", "github_create_issue"]
  local_servers: {"filesystem": {...}}
```

---

## Container Initialization Process

### **Docs Tasks (Single Repository)**

```bash
# 1. Clone the repository containing task definitions
git clone "$REPO_URL" repo
cd repo && git checkout "$SOURCE_BRANCH"

# 2. Checkout to randomized feature branch
DOCS_BRANCH="docs-generation-$(date +%Y%m%d-%H%M%S)"
git checkout -b "$DOCS_BRANCH"

# 3. Deploy rendered configuration files
cp /config/CLAUDE.md $WORK_DIR/
cp /config/settings.json $WORK_DIR/

# 4. Execute Claude (reads .taskmaster/tasks/tasks.json directly)
claude -p --output-format stream-json --verbose "$PROMPT"

# 5. Stop hook creates PR back to source branch
```

### **Code Tasks (Dual Repository)**

```bash
# 1. Platform Repository Clone (for task documentation)
git clone "$PLATFORM_URL" platform-repo
cd platform-repo && git checkout "$PLATFORM_BRANCH"

# 2. Copy task documentation
mkdir -p task
cp platform-repo/.taskmaster/docs/task-1/task.md task/
cp platform-repo/.taskmaster/docs/task-1/acceptance-criteria.md task/
cp platform-repo/.taskmaster/docs/task-1/architecture.md task/

# 3. Destination Repository Setup
git clone "$REPO_URL" target-repo
cd target-repo

# Create or resume feature branch
if [ "$CURRENT_BRANCH" = "feature/task-1-implementation" ]; then
  # Resume existing work
  git reset --hard HEAD && git clean -fd
else
  # Start new task
  git checkout -b "feature/task-1-implementation"
fi

# 4. Deploy configuration files (including MCP setup)
cp /config/CLAUDE.md $CLAUDE_WORK_DIR/
cp /config/mcp.json $CLAUDE_WORK_DIR/.mcp.json
cp /config/client-config.json $CLAUDE_WORK_DIR/
cp /config/coding-guidelines.md $CLAUDE_WORK_DIR/

# 5. Execute Claude with MCP tools
# The CRD determines if this is a fresh task or continuation
# based on retry/continuation field in the CodeRun spec
claude -p --output-format stream-json --verbose "$PROMPT"
```

**Key Architectural Differences**:
- **Docs**: Single repo, randomized branch, hook-driven PR creation
- **Code**: Dual repo, predictable branch naming, CRD-controlled retry/continuation

# Orchestrator

A Rust-based unified orchestration service that processes requests from multiple sources (GitHub, PM Agent, Grafana, CLI) and orchestrates Kubernetes Jobs for AI agent task execution.

## Features

- **GitHub Webhook Processing**: Handles GitHub webhook events (issues, pull requests, etc.)
- **Kubernetes Integration**: Creates and manages Kubernetes Jobs for code processing
- **Health Monitoring**: Built-in health check endpoint
- **Graceful Shutdown**: Proper signal handling for clean container shutdown
- **Security**: Runs as non-root user with minimal privileges
- **Observability**: Structured logging with configurable levels

## Architecture

The orchestrator follows a modular architecture:

1. **Webhook Receiver**: Axum-based HTTP server receives GitHub webhooks
2. **Event Parser**: Parses GitHub webhook payloads and extracts relevant information
3. **Job Orchestrator**: Creates Kubernetes Jobs with appropriate configurations
4. **Kubernetes Client**: Manages Job lifecycle and status monitoring

## Configuration

The controller uses environment variables and a default configuration:

### Server Configuration
- `SERVER_HOST`: Server bind address (default: `0.0.0.0`)
- `SERVER_PORT`: Server port (default: `8080`)

### Kubernetes Configuration
- `KUBERNETES_NAMESPACE`: Target namespace for Jobs (default: `default`)
- `JOB_IMAGE`: Container image for Jobs (default: `anthropic/claude-code:latest`)
- `JOB_TTL_SECONDS`: Job TTL after completion (default: `1800` - 30 minutes)
- `JOB_MEMORY_REQUEST`: Job memory request (default: `2Gi`)
- `JOB_MEMORY_LIMIT`: Job memory limit (default: `4Gi`)
- `JOB_CPU_REQUEST`: Job CPU request (default: `1`)
- `JOB_CPU_LIMIT`: Job CPU limit (default: `2`)

## Building

> **Note**: This orchestrator is a **pure Rust application** and does not require Node.js. The main project Dockerfile (`../Dockerfile`) is for the development environment that includes both Node.js and Rust.

### Local Development

#### Prerequisites
- Rust 1.75+
- Docker (for containerization)
- Kubernetes cluster (for testing)

#### Quick Build
```bash
# Build and test locally
cargo build --release
cargo test

# Run locally (requires Kubernetes access)
cargo run
```

#### Docker Build
```bash
# Simple build
./build.sh

# Build with specific tag
./build.sh -t v1.0.0

# Build and push to registry
./build.sh -r ghcr.io/yourusername -t v1.0.0 -p

# Build without running tests (faster)
./build.sh -b
```

### CI/CD

The project includes a GitHub Actions workflow that:

1. **Builds** multi-architecture Docker images (amd64, arm64)
2. **Tests** the code with cargo test, clippy, and fmt
3. **Scans** for security vulnerabilities with Trivy
4. **Publishes** to GitHub Container Registry (ghcr.io)

#### Triggering Builds

- **Push to main**: Builds and publishes with `latest` tag
- **Pull Requests**: Builds and tests without publishing
- **Releases**: Builds and publishes with semantic version tags

#### Image Tags

The CI system creates multiple tags:
- `latest`: Latest build from main branch
- `main-<sha>`: Specific commit from main
- `v1.2.3`: Semantic version tags from releases
- `pr-123`: Pull request builds (not published)

## Deployment

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orchestrator
spec:
  replicas: 2
  selector:
    matchLabels:
      app: orchestrator
  template:
    metadata:
      labels:
        app: orchestrator
    spec:
      containers:
      - name: orchestrator
        image: ghcr.io/yourusername/platform/orchestrator:latest
        ports:
        - containerPort: 8080
        env:
        - name: KUBERNETES_NAMESPACE
          value: "claude-jobs"
        - name: JOB_IMAGE
          value: "anthropic/claude-code:latest"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: orchestrator-service
spec:
  selector:
    app: orchestrator
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Docker Compose (Development)

```yaml
version: '3.8'
services:
  orchestrator:
    image: orchestrator:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=orchestrator=debug
      - KUBERNETES_NAMESPACE=default
    # Note: Requires proper Kubernetes configuration
```

## API Endpoints

### Health Check
```
GET /health
```
Returns `200 OK` if the service is healthy.

### GitHub Webhook
```
POST /webhook/github
```
Processes GitHub webhook events. Expects GitHub webhook payload in request body.

## Development

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With logging
RUST_LOG=debug cargo test
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### Local Testing with ngrok

For testing GitHub webhooks locally:

```bash
# Install ngrok
# Start the controller
cargo run

# In another terminal, expose local port
ngrok http 8080

# Configure GitHub webhook URL to point to ngrok URL + /webhook/github
```

## Security Considerations

- **Container Security**: Runs as non-root user (UID 1000)
- **Resource Limits**: Configured memory and CPU limits
- **Network Security**: Only exposes necessary ports
- **Image Scanning**: Trivy security scanning in CI
- **Minimal Dependencies**: Uses distroless-style base image

## Monitoring and Observability

### Logging
- Structured logging with `tracing`
- Configurable log levels via `RUST_LOG`
- Request/response logging for HTTP endpoints

### Metrics
- Built-in health check endpoint
- Container metrics via Docker/Kubernetes
- Custom metrics can be added via prometheus crate

### Health Checks
- HTTP health endpoint at `/health`
- Kubernetes liveness and readiness probes
- Docker health check built into image

## Troubleshooting

### Common Issues

1. **Kubernetes Connection Failed**
   ```
   Error: Failed to initialize Kubernetes client
   ```
   - Ensure proper KUBECONFIG or in-cluster credentials
   - Check RBAC permissions for Job creation

2. **Port Already in Use**
   ```
   Error: Address already in use (os error 48)
   ```
   - Change port via `SERVER_PORT` environment variable
   - Check for conflicting services

3. **Job Creation Failed**
   ```
   Error: Failed to create Kubernetes Job
   ```
   - Verify namespace exists and is accessible
   - Check resource quotas and limits
   - Ensure job image is accessible

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=orchestrator=debug cargo run

# Or in Docker
docker run -e RUST_LOG=debug orchestrator:latest
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

The CI will automatically:
- Run tests and linting
- Build Docker images
- Scan for security issues
- Provide feedback on the PR

## License

[Add your license information here]