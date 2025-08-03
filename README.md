# 5D Labs Agent Platform

An AI-powered development platform that helps you generate documentation and implement code using Claude agents through simple MCP (Model Context Protocol) tools.

## What It Does

The platform provides two main capabilities:
- **📝 Documentation Generation**: Automatically creates comprehensive documentation for your Task Master projects
- **⚡ Code Implementation**: Deploys autonomous Claude agents to implement specific tasks from your project

Both operations run as Kubernetes jobs with enhanced reliability through TTL-safe reconciliation, preventing infinite loops and ensuring proper resource cleanup. All results are automatically submitted via GitHub PRs.

## Getting Started

### Prerequisites
- Access to a Cursor/Claude environment with MCP support
- A project with Task Master initialized (`.taskmaster/` directory)
- GitHub repository for your project

## Installation

This is an integrated platform with a clear data flow:

**Component Architecture:**
- **MCP Server (`fdl-mcp`)**: Handles MCP protocol calls from Cursor/Claude
- **CLI (`fdl`)**: Makes REST API calls to the agent-platform service
- **Orchestrator Service**: Kubernetes REST API that creates CodeRun/DocsRun CRDs
- **Kubernetes Controllers**: Separate controllers for CodeRun and DocsRun resources with TTL-safe reconciliation
- **Agent Workspaces**: Isolated persistent volumes for each service with session continuity

**Data Flow:**
1. Cursor calls `docs()` or `task()` via MCP protocol
2. MCP server receives call and internally executes CLI
3. CLI makes HTTP requests to agent-platform REST API (`/pm/tasks`)
4. Orchestrator creates CodeRun/DocsRun custom resources
5. Dedicated Kubernetes controllers reconcile CRDs with idempotent job management
6. Controllers deploy Claude agents as Jobs with workspace isolation
7. Agents complete work and submit GitHub PRs with automatic cleanup

### Deploy the Complete Platform

```bash
# Add the 5dlabs Helm repository
helm repo add 5dlabs https://5dlabs.github.io/platform
helm repo update

# Install Custom Resource Definitions (CRDs) first
kubectl apply -f https://raw.githubusercontent.com/5dlabs/platform/main/infra/charts/agent-platform/crds/platform-crds.yaml

# Install the agent-platform
helm install agent-platform 5dlabs/agent-platform --namespace agent-platform --create-namespace

# Setup agent secrets (interactive)
wget https://raw.githubusercontent.com/5dlabs/platform/main/infra/scripts/setup-agent-secrets.sh
chmod +x setup-agent-secrets.sh
./setup-agent-secrets.sh --help
```

**Requirements:**
- Kubernetes 1.19+
- Helm 3.2.0+
- GitHub Personal Access Token
- Anthropic API Key

**What you get:**
- Complete agent-platform platform deployed to Kubernetes
- REST API for task management
- Separate Kubernetes controllers for CodeRun/DocsRun resources with TTL-safe reconciliation
- Agent workspace management and isolation with persistent volumes
- Automatic resource cleanup and job lifecycle management
- MCP tools that connect to your deployment

### Optional: Remote Cluster Access with TwinGate

To access your Kubernetes cluster from anywhere (not just local network), install TwinGate connector:

```bash
# Add TwinGate Helm repository
helm repo add twingate https://twingate.github.io/helm-charts
helm repo update

# Install TwinGate connector (replace tokens with your actual values)
helm upgrade --install twingate-weightless-hummingbird twingate/connector \
  -n default \
  --set connector.network="maroonsnake" \
  --set connector.accessToken="your-access-token" \
  --set connector.refreshToken="your-refresh-token"
```

**Important**: After installation, add your Kubernetes service CIDR as resources in TwinGate admin panel. This enables the MCP tools to reach the agent-platform service using internal Kubernetes service URLs (e.g., `http://agent-platform.agent-platform.svc.cluster.local`) from anywhere.

### Install CLI Tools

For the MCP tools and CLI utilities, you can install pre-built binaries:

```bash
# One-liner installer (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/5dlabs/platform/releases/download/v0.2.0/tools-installer.sh | sh

# Verify installation
fdl --help       # CLI tool for direct API calls
```

**What you get:**
- `fdl` - Command-line tool for direct agent-platform API calls
- `fdl-mcp` - MCP server that integrates with Cursor/Claude
- Multi-platform support (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)
- Automatic installation to system PATH

### Configure Cursor MCP Integration

After installing the CLI tools, configure Cursor to use the MCP server by creating a `.cursor/mcp.json` file in your project directory:

```json
{
  "mcpServers": {
    "fdl-mcp": {
      "command": "fdl-mcp",
      "args": [],
      "env": {
        "FDL_DEFAULT_DOCS_USER": "your-github-username",
        "FDL_DEFAULT_CODE_USER": "your-github-username"
      }
    }
  }
}
```

**Configuration options:**
- `FDL_DEFAULT_DOCS_USER` - Default GitHub username for documentation generation (optional)
- `FDL_DEFAULT_CODE_USER` - Default GitHub username for code implementation (optional)

**Usage:**
1. Create the `.cursor/mcp.json` file in your project root
2. Replace `"your-github-username"` with your actual GitHub username
3. Restart Cursor to load the MCP server
4. The `docs()` and `task()` functions will be available in Claude conversations

**Important**: The MCP server connects to your deployed agent-platform service. Ensure your agent-platform is accessible from your development environment (either locally or via TwinGate for remote clusters).

### Building from Source (Development)

```bash
# Build from source
git clone https://github.com/5dlabs/platform.git
cd platform/agent-platform

# Build both CLI and MCP server
cargo build --release --bin fdl --bin fdl-mcp

# Verify the builds
./target/release/fdl --help       # CLI tool

# Install to your system (optional)
cp target/release/fdl /usr/local/bin/
cp target/release/fdl-mcp /usr/local/bin/
```

### MCP Tools Available

The platform exposes two primary MCP tools:

#### 1. `docs` - Generate Documentation
Analyzes your Task Master project and creates comprehensive documentation.

```javascript
docs({
  working_directory: "_projects/my-app",
  github_user: "your-github-username"  // optional if FDL_DEFAULT_DOCS_USER is set
});
```

**What happens:**
✅ Creates a Claude agent with your project context
✅ Analyzes all tasks in your Task Master project
✅ Generates comprehensive documentation
✅ Submits a GitHub PR with the docs

**Generated Documents:**
```
.taskmaster/docs/
├── task-1/
│   ├── task.md           # Comprehensive task documentation
│   ├── acceptance-criteria.md  # Clear success criteria
│   └── prompt.md         # Implementation guidance for agents
├── task-2/
│   ├── task.md
│   ├── acceptance-criteria.md
│   └── prompt.md
└── ...
```

#### 2. `task` - Implement Code
Deploys an autonomous Claude agent to implement a specific task from your Task Master project.

```javascript
// Implement a specific task (initial implementation)
task({
  task_id: 5,
  service: "api-server",
  repository: "myorg/my-project",
  docs_repository: "myorg/my-docs",
  docs_project_directory: "_projects/my-project",
  github_user: "myusername",
  working_directory: "services/api-server"  // optional, defaults to repo root
});

// Continue working on a partially completed or failed task
task({
  task_id: 5,
  service: "api-server", 
  repository: "myorg/my-project",
  docs_repository: "myorg/my-docs",
  docs_project_directory: "_projects/my-project",
  github_user: "myusername",
  continue_session: true
});
```

**What happens:**
✅ Creates a Claude agent with the generated docs as context
✅ Loads the specific task details from Task Master
✅ Implements the code autonomously
✅ Runs tests and validation
✅ Submits a GitHub PR with the implementation

## MCP Tool Reference

Complete parameter reference for both MCP tools.

### `docs` Tool Parameters

**Required:**
- `working_directory` - Working directory containing .taskmaster folder (e.g., `"_projects/simple-api"`)

**Optional:**
- `github_user` - GitHub username for authentication (uses `FDL_DEFAULT_DOCS_USER` env var if not specified)
- `model` - Claude model to use (defaults to server-side configuration)

### `task` Tool Parameters

**Required:**
- `task_id` - Task ID to implement from task files (integer, minimum 1)
- `service` - Target service name, creates workspace-{service} PVC (pattern: `^[a-z0-9-]+$`)
- `repository` - Target repository in format 'org/repo' or 'user/repo' (e.g., `"5dlabs/platform"`)
- `docs_repository` - Documentation repository in format 'org/repo' or 'user/repo' where Task Master definitions are stored
- `docs_project_directory` - Project directory within docs repository (e.g., `"_projects/simple-api"`, use `"."` for repo root)
- `github_user` - GitHub username for authentication and task assignment

**Optional:**

- `working_directory` - Working directory within target repository (defaults to `"."` for repo root)
- `model` - Claude model to use (defaults to server-side configuration)
- `continue_session` - Whether to continue a previous session (boolean, default: false)
- `env` - Environment variables to set in the container (object with key-value pairs)
- `env_from_secrets` - Environment variables from secrets (array of objects with `name`, `secretName`, `secretKey`)

## Template Customization

The platform uses a template system to customize Claude agent behavior, settings, and prompts. Templates are Handlebars (`.hbs`) files that get rendered with task-specific data.

**Model Defaults**: The agent-platform provides server-side model defaults (`claude-opus-4-20250514` for docs, `claude-sonnet-4-20250514` for code tasks) that can be overridden via MCP parameters or CLI arguments.

### Template Architecture

**Docs Tasks**: Generate documentation for Task Master projects

- **Prompts**: Rendered from `docs/prompt.md.hbs` template into ConfigMap
- **Settings**: `docs/settings.json.hbs` controls model, permissions, tools
- **Container Script**: `docs/container.sh.hbs` handles Git workflow and Claude execution

**Code Tasks**: Implement specific Task Master task IDs

- **Prompts**: Read from docs repository at `{docs_project_directory}/.taskmaster/docs/task-{id}/prompt.md` (or `_projects/{service}/.taskmaster/docs/task-{id}/prompt.md`)
- **Settings**: `code/settings.json.hbs` controls model, permissions, MCP tools
- **Container Script**: `code/container.sh.hbs` handles dual-repo workflow and Claude execution

### How to Customize

#### 1. Changing Agent Settings

Edit the settings template files directly:

```bash
# For docs generation agents
vim infra/charts/agent-platform/claude-templates/docs/settings.json.hbs

# For code implementation agents
vim infra/charts/agent-platform/claude-templates/code/settings.json.hbs
```

Settings control:

- Model selection (`claude-opus-4`, `claude-sonnet-4`, etc.)
- Tool permissions and access
- MCP tool configuration
- Enterprise managed settings

See [Claude Code Settings](https://docs.anthropic.com/en/docs/claude-code/settings) for complete configuration options.

#### 2. Updating Prompts

**For docs tasks** (affects all documentation generation):

```bash
# Edit the docs prompt template
vim infra/charts/agent-platform/claude-templates/docs/prompt.md.hbs
```

**For code tasks** (affects specific task implementation):

```bash
# Edit task-specific files in your docs repository
vim {docs_project_directory}/.taskmaster/docs/task-{id}/prompt.md
vim {docs_project_directory}/.taskmaster/docs/task-{id}/task.md
vim {docs_project_directory}/.taskmaster/docs/task-{id}/acceptance-criteria.md
```

#### 3. Adding Custom Hooks

Hooks are shell scripts that run during agent execution. Add new hook files to the `claude-templates` directory:

```bash
# Create new hook script (docs example)
vim infra/charts/agent-platform/claude-templates/docs/hooks/my-custom-hook.sh.hbs

# Create new hook script (code example)
vim infra/charts/agent-platform/claude-templates/code/hooks/my-custom-hook.sh.hbs
```

Hook files are automatically discovered and rendered. Ensure the hook name matches any references in your settings templates.

See [Claude Code Hooks Guide](https://docs.anthropic.com/en/docs/claude-code/hooks-guide) for detailed hook configuration and examples.

#### 4. Deploying Template Changes

After editing any template files, redeploy the agent-platform:

```bash
# Deploy template changes
helm upgrade agent-platform . -n agent-platform

# Verify ConfigMap was updated
kubectl get configmap claude-templates-configmap -n agent-platform -o yaml
```

**Important**: Template changes only affect new agent jobs. Running jobs continue with their original templates.

### Template Variables

Common variables available in templates:

- `{{task_id}}` - Task ID for code tasks
- `{{service_name}}` - Target service name
- `{{github_user}}` - GitHub username
- `{{repository_url}}` - Target repository URL
- `{{working_directory}}` - Working directory path
- `{{model}}` - Claude model name
- `{{docs_repository_url}}` - Documentation repository URL

## Best Practices

1. **Always generate docs first** to establish baseline documentation
2. **Implement tasks sequentially** based on dependencies
3. **Use `continue_session: true`** for retries on the same task
4. **Review GitHub PRs promptly** - agents provide detailed logs and explanations
5. **Check PR descriptions** for detailed agent logs when troubleshooting

## Support

- Check GitHub PRs for detailed agent logs and explanations
- Review Task Master project structure in `.taskmaster/` directory
- Verify repository access and GitHub authentication setup

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). This means:

- ✅ You can use, modify, and distribute this software freely
- ✅ You can use it for commercial purposes
- ⚠️ If you deploy a modified version on a network server, you must provide source code access to users
- ⚠️ Any derivative works must also be licensed under AGPL-3.0

The AGPL license is specifically designed for server-side software to ensure that improvements to the codebase remain open source, even when deployed as a service. This protects the open source nature of the project while allowing commercial use.

**Source Code Access**: Since this platform operates as a network service, users interacting with it have the right to access the source code under AGPL-3.0. The complete source code is available at this repository, ensuring full compliance with AGPL-3.0's network clause.

For more details, see the [LICENSE](LICENSE) file.

## Related Projects

- **[Task Master AI](https://github.com/eyaltoledano/claude-task-master)** - The AI-powered task management system that works perfectly with this agent-platform platform. Task Master AI helps you break down complex projects into manageable tasks, which can then be implemented using this platform's `task()` MCP tool.

## Roadmap

See our [ROADMAP.md](ROADMAP.md) for upcoming features and planned enhancements to the platform.

---

*The platform runs on Kubernetes and automatically manages Claude agent deployments, workspace isolation, and GitHub integration. All you need to do is call the MCP tools and review the resulting PRs.*


