# Orchestrator CLI Complete Command Reference

## Overview

The orchestrator CLI is a command-line tool for interacting with the Orchestrator service to manage Claude agent tasks. This document provides a comprehensive reference of all commands, arguments, and their defaults.

## Binary Information

- **Name**: `orchestrator`
- **Version**: Available via `orchestrator --version`
- **Description**: CLI for Unified Orchestration Service

## Global Options

These options are available for all commands:

| Option | Short | Type | Default | Environment Variable | Description |
|--------|-------|------|---------|---------------------|-------------|
| `--api-url` | - | String | `http://orchestrator.local/api/v1` | `ORCHESTRATOR_API_URL` | API endpoint URL |
| `--output` | `-o` | Enum | `table` | - | Output format (table, json, yaml) |
| `--no-color` | - | Flag | `false` | - | Disable colored output |
| `--verbose` | `-v` | Flag | `false` | - | Enable verbose logging |
| `--help` | `-h` | Flag | - | - | Print help information |
| `--version` | `-V` | Flag | - | - | Print version information |

## Commands

### 1. Task Commands (`orchestrator task`)

#### 1.1 Submit Task (Simplified Workflow)
**Command**: `orchestrator task submit <TASK_ID> [OPTIONS]`

Submit a new task using Task Master directory structure.

| Argument/Option | Short | Type | Default | Required | Description |
|-----------------|-------|------|---------|----------|-------------|
| `<TASK_ID>` | - | u32 | - | Yes | Task ID to submit from Task Master |
| `--service` | `-s` | String | - | Yes | Target service name (e.g., auth-service, api-gateway) |
| `--agent` | `-a` | String | `claude-agent-1` | No | Agent name |
| `--taskmaster-dir` | `-d` | String | `.taskmaster` | No | Path to Task Master directory |
| `--context` | `-c` | Vec<String> | `[]` | No | Path to additional context files (can specify multiple) |
| `--tools` | `-t` | Vec<String> | `[]` | No | Agent tools (format: tool_name:enabled) |
| `--repo` | `-r` | String | None | No | Repository URL to clone |
| `--branch` | `-b` | String | `main` | No | Repository branch or tag to checkout |
| `--github-user` | - | String | None | No | GitHub username for authentication |
| `--retry` | - | Flag | `false` | No | Indicates this is a retry of a previous attempt |
| `--model` | `-m` | String | `sonnet` | No | Claude model to use (sonnet, opus) |

#### 1.2 Submit Task (Advanced Workflow)
**Command**: `orchestrator task submit-advanced [OPTIONS]`

Submit a new task with explicit file paths.

| Option | Short | Type | Default | Required | Description |
|--------|-------|------|---------|----------|-------------|
| `--task-json` | - | String | - | Yes | Path to tasks.json file containing all tasks |
| `--task-id` | - | u32 | - | Yes | Task ID to submit from the tasks.json file |
| `--design-spec` | - | String | - | Yes | Path to design specification markdown file |
| `--prompt` | - | String | None | No | Path to autonomous prompt markdown file |
| `--service-name` | - | String | - | Yes | Target service name |
| `--agent-name` | - | String | - | Yes | Agent name (e.g., claude-agent-1) |
| `--retry` | - | Flag | `false` | No | Indicates this is a retry |
| `--model` | `-m` | String | `sonnet` | No | Claude model to use (sonnet, opus) |

#### 1.3 Get Task Status
**Command**: `orchestrator task status <TASK_ID>`

Get the status of a specific task.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<TASK_ID>` | u32 | Yes | Task ID |

#### 1.4 Add Context to Task
**Command**: `orchestrator task add-context <TASK_ID> <CONTEXT> [OPTIONS]`

Add context to a running task.

| Argument/Option | Short | Type | Default | Required | Description |
|-----------------|-------|------|---------|----------|-------------|
| `<TASK_ID>` | - | u32 | - | Yes | Task ID |
| `<CONTEXT>` | - | String | - | Yes | Context to add (can be text or file path) |
| `--file` | `-f` | Flag | `false` | No | Treat context as file path |

#### 1.5 List Tasks
**Command**: `orchestrator task list [OPTIONS]`

List all tasks with optional filtering.

| Option | Short | Type | Default | Required | Description |
|--------|-------|------|---------|----------|-------------|
| `--service` | `-s` | String | None | No | Filter by service |
| `--status` | - | String | None | No | Filter by status |

#### 1.6 Initialize Documentation
**Command**: `orchestrator task init-docs [OPTIONS]`

Initialize documentation for Task Master tasks using Claude.

| Option | Short | Type | Default | Required | Description |
|--------|-------|------|---------|----------|-------------|
| `--taskmaster-dir` | `-d` | String | `.taskmaster` | No | Path to Task Master directory |
| `--model` | `-m` | String | `sonnet` | No | Claude model to use (sonnet, opus) |
| `--repo` | `-r` | String | None | No | Repository URL (auto-detected if not specified) |
| `--source-branch` | - | String | None | No | Source branch to base documentation branch from |
| `--target-branch` | - | String | None | No | Target branch for PR (defaults to source branch) |
| `--working-dir` | `-w` | String | None | No | Working directory within repo |
| `--force` | `-f` | Flag | `false` | No | Overwrite existing documentation |
| `--task-id` | `-t` | u32 | None | No | Generate docs for specific task only |
| `--update` | `-u` | Flag | `false` | No | Update existing docs from current tasks.json |
| `--update-all` | - | Flag | `false` | No | Force update all docs regardless of changes |
| `--dry-run` | - | Flag | `false` | No | Preview what would be generated |
| `--verbose` | `-v` | Flag | `false` | No | Show detailed generation progress |

### 2. Job Commands (`orchestrator job`)

#### 2.1 List Jobs
**Command**: `orchestrator job list [OPTIONS]`

List all jobs with optional filtering.

| Option | Type | Default | Required | Description |
|--------|------|---------|----------|-------------|
| `--microservice` | String | None | No | Filter by microservice |
| `--status` | String | None | No | Filter by status |

#### 2.2 Get Job Details
**Command**: `orchestrator job get --id <JOB_ID>`

Get details of a specific job.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--id` | String | Yes | Job ID |

#### 2.3 Stream Job Logs
**Command**: `orchestrator job logs --id <JOB_ID> [OPTIONS]`

Stream logs from a job.

| Option | Short | Type | Default | Required | Description |
|--------|-------|------|---------|----------|-------------|
| `--id` | - | String | - | Yes | Job ID |
| `--follow` | `-f` | Flag | `false` | No | Follow log stream |

### 3. Config Commands (`orchestrator config`)

#### 3.1 Create ConfigMap
**Command**: `orchestrator config create --name <NAME> --files <FILES>`

Create a new ConfigMap.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | String | Yes | ConfigMap name |
| `--files` | Vec<String> | Yes | Files to include (can specify multiple) |

#### 3.2 Get ConfigMap
**Command**: `orchestrator config get --name <NAME>`

Get ConfigMap contents.

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | String | Yes | ConfigMap name |

### 4. Health Check
**Command**: `orchestrator health`

Check service health. No additional arguments or options.

## Output Formats

The CLI supports three output formats, selectable via the `--output` or `-o` flag:

1. **table** (default): Human-readable table format with colored output (unless `--no-color` is specified)
2. **json**: JSON format for machine processing
3. **yaml**: YAML format for human and machine readability

## Status Values

### Task Status Values
- `pending`: Task is ready to work on
- `in-progress`: Task is currently being worked on
- `completed`: Task has been completed successfully
- `failed`: Task execution failed
- `cancelled`: Task was cancelled
- `blocked`: Task is blocked

### Job Status Values
- `Pending`: Job is waiting to start
- `Running`: Job is currently executing
- `Succeeded`: Job completed successfully
- `Failed`: Job execution failed
- `Unknown`: Job status cannot be determined

## Examples

```bash
# Basic task submission
orchestrator task submit 1001 --service auth-service

# Task submission with all options
orchestrator task submit 1002 \
  --service api-gateway \
  --agent claude-agent-2 \
  --taskmaster-dir ./my-tasks \
  --context ./additional-context.md \
  --context ./more-context.txt \
  --tools "bash:true,edit:true,webfetch:false" \
  --repo https://github.com/org/repo \
  --branch feature/new-feature \
  --github-user my-github-user \
  --model opus \
  --retry

# Get task status with JSON output
orchestrator --output json task status 1001

# List all jobs with filtering
orchestrator job list --microservice auth-service --status Running

# Follow job logs
orchestrator job logs --id job-abc123 --follow

# Initialize docs with verbose output
orchestrator task init-docs --verbose --model opus --force
```

## Environment Variables

- `ORCHESTRATOR_API_URL`: Sets the default API endpoint URL
  - Inside Kubernetes cluster: `http://orchestrator.orchestrator.svc.cluster.local/api/v1`
  - Local with port-forward: `http://localhost:8080/api/v1`
  - Via ingress: `http://orchestrator.local/api/v1`

## Tool Format Specification

When using the `--tools` option, specify tools in the format:
```
tool_name:enabled
```

Example:
```bash
--tools "bash:true,edit:true,read:true,webfetch:false"
```

## Notes

1. The CLI uses colored output by default when connected to a terminal. Use `--no-color` to disable.
2. Verbose logging includes timestamps and additional debug information.
3. Multiple `--context` and `--files` options can be specified by repeating the flag.
4. The `retry` flag indicates to the system that this is a retry of a previously failed attempt.
5. Model options are currently limited to "sonnet" and "opus".