# Orchestrator CLI Reference

## Overview

The orchestrator CLI provides commands to interact with the Orchestrator service for managing Claude agent tasks.

## Installation

```bash
cd orchestrator
cargo build --release --bin orchestrator
# Binary will be at: orchestrator/target/release/orchestrator
```

## Global Options

All commands support these global options:

```bash
orchestrator [GLOBAL_OPTIONS] <COMMAND>

Global Options:
  --api-url <API_URL>     # API endpoint URL [env: ORCHESTRATOR_API_URL=]
                          # Default: http://orchestrator.local/api/v1
  -o, --output <OUTPUT>   # Output format: table, json, yaml [default: table]
  --no-color              # Disable colored output
  -v, --verbose           # Enable verbose logging
  -h, --help              # Print help
  -V, --version           # Print version
```

## Environment Configuration

The CLI respects the following environment variables:

- `ORCHESTRATOR_API_URL`: Sets the API endpoint URL

### Common API URLs

```bash
# From within Kubernetes cluster (e.g., from a pod)
export ORCHESTRATOR_API_URL=http://orchestrator.orchestrator.svc.cluster.local/api/v1

# From local machine with port-forward
kubectl port-forward -n orchestrator deployment/orchestrator 8080:8080
export ORCHESTRATOR_API_URL=http://localhost:8080/api/v1

# Via ingress (if configured)
export ORCHESTRATOR_API_URL=http://orchestrator.local/api/v1
```

## Commands

### Task Management

#### Submit Task

Submit a new task using Task Master directory structure:

```bash
orchestrator task submit [OPTIONS] --service <SERVICE> <TASK_ID>

Arguments:
  <TASK_ID>                           # Task ID from .taskmaster/tasks/tasks.json

Required Options:
  -s, --service <SERVICE>             # Target service name (e.g., todo-api)

Optional Options:
  -a, --agent <AGENT>                 # Agent name [default: claude-agent-1]
  -d, --taskmaster-dir <DIR>          # Path to Task Master directory [default: .taskmaster]
  -c, --context <CONTEXT>             # Path to additional context files
  -t, --tools <TOOLS>                 # Agent tools (format: tool_name:enabled)
  -r, --repo <REPO>                   # Repository URL to clone
  -b, --branch <BRANCH>               # Repository branch [default: main]
  --github-user <USER>                # GitHub username for authentication
  --retry                             # Indicates this is a retry
  -m, --model <MODEL>                 # Claude model: sonnet, opus [default: sonnet]

Examples:
  # Basic task submission
  orchestrator task submit --service todo-api 1001

  # With repository and GitHub auth
  orchestrator task submit --service todo-api \
    --repo https://github.com/org/todo-api \
    --github-user swe-1-5dlabs \
    1001

  # With custom tools
  orchestrator task submit --service auth-api \
    --tools "bash:true,edit:true,webfetch:false" \
    1002

  # Retry a task
  orchestrator task submit --service api-gateway --retry 1003
```

#### List Tasks

List all tasks:

```bash
orchestrator task list

# With different output format
orchestrator -o json task list
```

#### Get Task Status

Get status of a specific task:

```bash
orchestrator task status <TASK_RUN_ID>

Example:
  orchestrator task status abc123-def456-789
```

#### Add Context to Task

Add context to a running task:

```bash
orchestrator task add-context [OPTIONS] <TASK_RUN_ID>

Options:
  -c, --context <FILE>    # Path to context file
  -m, --message <MSG>     # Context message

Examples:
  # Add message context
  orchestrator task add-context abc123 -m "Focus on error handling"

  # Add file context
  orchestrator task add-context abc123 -c additional-requirements.md
```

### Job Management

#### List Jobs

```bash
orchestrator job list

Options:
  -n, --namespace <NS>    # Kubernetes namespace [default: orchestrator]
```

#### Get Job Logs

```bash
orchestrator job logs <JOB_NAME>

Options:
  -n, --namespace <NS>    # Kubernetes namespace [default: orchestrator]
  -f, --follow            # Follow log output
  --tail <LINES>          # Number of lines from end [default: 100]
```

### ConfigMap Management

#### List ConfigMaps

```bash
orchestrator config list

Options:
  -n, --namespace <NS>    # Kubernetes namespace [default: orchestrator]
```

#### Get ConfigMap

```bash
orchestrator config get <NAME>

Options:
  -n, --namespace <NS>    # Kubernetes namespace [default: orchestrator]
```

### Health Check

Check orchestrator service health:

```bash
orchestrator health
```

## Task Master Integration

The CLI expects the following directory structure when using `task submit`:

```
.taskmaster/
├── tasks/
│   └── tasks.json          # Task definitions
└── docs/
    ├── design-spec.md      # Comprehensive design
    ├── prompt.md           # Implementation instructions
    ├── acceptance-criteria.md  # Success criteria
    └── regression-testing.md   # Testing guidelines
```

## GitHub Authentication

For tasks that need to create pull requests, set up GitHub authentication:

```bash
# 1. Create a Kubernetes secret with the PAT
kubectl create secret generic github-pat-<username> \
  --from-literal=token=<github-pat> \
  -n orchestrator

# 2. Use the username when submitting tasks
orchestrator task submit --service my-service \
  --repo https://github.com/org/repo \
  --github-user <username> \
  1001
```

## Common Workflows

### Submit a Fresh Task

```bash
# Set API URL
export ORCHESTRATOR_API_URL=http://orchestrator.orchestrator.svc.cluster.local/api/v1

# Submit task
orchestrator task submit \
  --service todo-api \
  --repo https://github.com/5dlabs/todo-api \
  --github-user swe-1-5dlabs \
  1001
```

### Monitor Task Progress

```bash
# List all tasks
orchestrator task list

# Get specific task status
orchestrator task status <task-run-id>

# Follow job logs
orchestrator job logs <job-name> -f
```

### Retry Failed Task

```bash
# Submit with retry flag
orchestrator task submit \
  --service todo-api \
  --repo https://github.com/5dlabs/todo-api \
  --github-user swe-1-5dlabs \
  --retry \
  1001
```

## Troubleshooting

### Connection Issues

If you get connection errors:

1. Check if orchestrator is running:
   ```bash
   kubectl get pods -n orchestrator
   ```

2. For local testing, ensure port-forward is active:
   ```bash
   kubectl port-forward -n orchestrator deployment/orchestrator 8080:8080
   ```

3. Verify API URL is correct:
   ```bash
   echo $ORCHESTRATOR_API_URL
   ```

### Task Submission Hanging

If task submission hangs:

1. Check orchestrator logs:
   ```bash
   kubectl logs -n orchestrator deployment/orchestrator
   ```

2. Ensure you're using the correct API URL based on your location:
   - Inside cluster: `http://orchestrator.orchestrator.svc.cluster.local/api/v1`
   - Local with port-forward: `http://localhost:8080/api/v1`

3. Check if a task with the same ID already exists:
   ```bash
   kubectl get taskrun -n orchestrator
   ```