# Example: Todo API Service

This example demonstrates how to use the agent platform to build a simple Todo API service using AI agents for development.

## Overview

This example project shows:
- How to structure tasks for AI agents using markdown files
- How to use the orchestrator CLI to submit tasks
- How the TaskRun CRD manages agent deployments
- How agents work autonomously on your codebase

## Project Structure

```
example/
├── README.md                    # This file
├── todo-api/                    # The service we're building
│   ├── .taskmaster/            # Task management for this service
│   │   └── tasks.json          # Task definitions
│   ├── CLAUDE.md               # Agent instructions
│   ├── package.json            # Node.js project file
│   └── src/                    # Source code (to be created by agents)
└── tasks/                      # Task markdown files for submission
    ├── 01-setup-project.md     # Initialize the project
    ├── 02-create-api.md        # Create REST API
    ├── 03-add-database.md      # Add database support
    └── 04-add-tests.md         # Add test suite
```

## Getting Started

### 1. Initialize the Example Project

```bash
cd example/todo-api
npm init -y
```

### 2. Submit Tasks to the Platform

Use the orchestrator CLI to submit tasks:

```bash
# Submit the first task
orchestrator-cli task submit \
  --service todo-api \
  --task-file ../tasks/01-setup-project.md

# Check task status
orchestrator-cli task status --service todo-api
```

### 3. Watch the Agent Work

The platform will:
1. Create a TaskRun resource in Kubernetes
2. Deploy a Claude agent as a Job
3. Mount the task files and service code
4. Agent works autonomously to complete the task
5. Updates are reflected in the TaskRun status

### 4. Submit Follow-up Tasks

After each task completes, submit the next one:

```bash
orchestrator-cli task submit \
  --service todo-api \
  --task-file ../tasks/02-create-api.md
```

## Task Examples

Each task file follows a specific format that helps the AI agent understand what to build. See the `tasks/` directory for examples of well-structured task definitions.

## Monitoring Progress

You can monitor task progress in several ways:

1. **CLI Status Command**: `orchestrator-cli task status --service todo-api`
2. **Kubernetes Resources**: `kubectl get taskruns -n orchestrator`
3. **Agent Logs**: `kubectl logs -n orchestrator -l service=todo-api`

## Expected Outcome

After running all tasks, you'll have:
- A working Node.js Express API for managing todos
- SQLite database integration
- Full test suite with Jest
- Proper error handling and validation
- RESTful endpoints for CRUD operations

This demonstrates how AI agents can build production-ready services based on high-level task descriptions.