# 5D Labs Agent Platform

An AI-powered development platform that helps you generate documentation and implement code using Claude agents through simple MCP (Model Context Protocol) tools.

## What It Does

The platform provides two main capabilities:
- **📝 Documentation Generation**: Automatically creates comprehensive documentation for your Task Master projects
- **⚡ Code Implementation**: Deploys autonomous Claude agents to implement specific tasks from your project

Both operations run as Kubernetes jobs and automatically submit results via GitHub PRs.

## Getting Started

### Prerequisites
- Access to a Cursor/Claude environment with MCP support
- A project with Task Master initialized (`.taskmaster/` directory)
- GitHub repository for your project

## Installation

This is an integrated platform that consists of:
- **Orchestrator**: Kubernetes service that manages Claude agents
- **MCP Tools**: CLI interface for docs/task commands
- **Agent Management**: Automated deployment and workspace isolation

All components work together - the MCP tools communicate with the orchestrator service to deploy and manage Claude agents.

### Deploy the Complete Platform

```bash
# Add the 5dlabs Helm repository
helm repo add 5dlabs https://5dlabs.github.io/platform
helm repo update

# Install Custom Resource Definitions (CRDs) first
kubectl apply -f https://github.com/5dlabs/platform/releases/download/v0.0.2/platform-crds.yaml

# Install the orchestrator
helm install orchestrator 5dlabs/orchestrator --namespace orchestrator --create-namespace

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
- Complete orchestrator platform deployed to Kubernetes
- REST API for task management
- Custom Kubernetes operators for CodeRun/DocsRun resources
- Agent workspace management and isolation
- MCP tools that connect to your deployment

### Building from Source (Development)

```bash
# Build from source
git clone https://github.com/5dlabs/platform.git
cd platform/orchestrator

# Build the tools binary (contains both CLI and MCP server)
cargo build --release --bin fivedlabs-tools

# The binary includes both tools:
./target/release/fivedlabs-tools --help          # CLI tool (5d-cli)
./target/release/fivedlabs-tools --mcp --help    # MCP server (5d-mcp)

# Install to your system (optional)
cp target/release/fivedlabs-tools /usr/local/bin/
```

### MCP Tools Available

The platform exposes two primary MCP tools:

#### 1. `docs` - Generate Documentation
Analyzes your Task Master project and creates comprehensive documentation.

```javascript
docs({
  working_directory: "_projects/my-app",
  model: "claude-opus-4-20250514",
  repository_url: "https://github.com/your-org/your-repo",
  source_branch: "main",
  github_user: "your-github-username"
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
  repository_url: "https://github.com/myorg/my-api",
  github_user: "myusername",
  working_directory: "_projects/my-api"
});

// Continue working on a partially completed or failed task
task({
  task_id: 5,
  service: "api-server",
  repository_url: "https://github.com/myorg/my-api",
  github_user: "myusername",
  working_directory: "_projects/my-api",
  continue_session: true,
  context_version: 2
});
```

**What happens:**
✅ Creates a Claude agent with the generated docs as context
✅ Loads the specific task details from Task Master
✅ Implements the code autonomously
✅ Runs tests and validation
✅ Submits a GitHub PR with the implementation

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

---

*The platform runs on Kubernetes and automatically manages Claude agent deployments, workspace isolation, and GitHub integration. All you need to do is call the MCP tools and review the resulting PRs.*
