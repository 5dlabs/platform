# Test Design Specification

## Overview
This is a comprehensive test to verify the orchestrator system works end-to-end, including:
- CLI task submission
- Repository cloning with simplified auth
- File system structure with subPath mounting
- GitHub authentication and permissions
- Claude agent execution

## Architecture
- **CLI**: Reads markdown files from disk and submits to API
- **API**: Creates TaskRun CRD with embedded markdown content
- **Controller**: Creates ConfigMap and Job with proper volume mounts
- **Init Container**: Clones repository and prepares workspace
- **Main Container**: Runs Claude with task context

## Key Test Points

### 1. CLI Functionality
- Can read Task Master directory structure
- Properly formats markdown files into JSON payload
- Submits to correct API endpoint
- Handles authentication parameters

### 2. Workspace Structure
With subPath mounting, Claude sees:
```
/workspace/                    # This is actually /workspace/test-service on host
├── .task/9999/               # Task files
│   ├── task.md
│   ├── design-spec.md
│   └── prompt.md
├── .claude/                  # Claude configuration
│   └── settings.local.json
├── .claude.json             # Project settings
├── .gitignore              # Prevent committing internal files
└── [repository files]      # Cloned from agent-sandbox
```

### 3. Authentication Flow
- CLI provides `--github-user swe-1-5dlabs`
- Controller auto-resolves to secret `github-pat-swe-1-5dlabs`
- Secret mounted in init container at `/secrets/github-pat-swe-1-5dlabs/token`
- Token used for git clone and gh CLI authentication

## Success Criteria
1. CLI successfully submits the task
2. Repository clones without errors
3. Claude can read all task files
4. Claude can commit and push changes
5. Logs show proper workspace structure