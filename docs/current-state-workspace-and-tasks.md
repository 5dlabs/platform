# Current State: Workspace and Task Management

## Date: 2025-07-05

This document captures the current implementation state before implementing subPath volume mounting changes.

## Current Workspace Structure

### Physical Storage
- Single shared PVC named `shared-workspace`
- Mounted at `/workspace` in all containers
- Uses local-path-provisioner (stored on NVMe drive)
- Shared across ALL services and tasks

### Container View
- Working directory: `/workspace/{service_name}`
- Repository cloned to: `/workspace/{service_name}/`
- Task files stored at: `/workspace/{service_name}/.task/{task_id}/`
- Claude sees the service name in its path

### Directory Layout Example
```
/workspace/
├── auth-service/
│   ├── .git/
│   ├── .task/
│   │   ├── 1001/
│   │   │   ├── task.md
│   │   │   └── design-spec.md
│   │   └── 1002/
│   │       └── task.md
│   ├── CLAUDE.md
│   └── [repository files]
├── api-gateway/
│   ├── .git/
│   ├── .task/
│   │   └── 2001/
│   │       └── task.md
│   └── [repository files]
└── test-service/
    └── ...
```

## Task File Management

### What Gets Stored
The `.task/{task_id}/` directory contains the markdown files from the API request:
- NOT the JSON request itself
- Each file from the `markdown_files` array
- Common files: task.md, design-spec.md, acceptance-criteria.md, prompt.md

### CLAUDE.md Generation
- **Regenerated for each task** - completely overwritten
- **Only references current task** - `@import .task/1002/task.md`
- **No automatic context carry-over** - previous tasks not referenced
- **Fresh conversation start** - no --continue by default

## Current Authentication Flow

### API Structure
```json
"repository": {
  "url": "https://github.com/org/repo",
  "branch": "main",
  "github_user": "swe-1-5dlabs"
}
```

### Secret Resolution
- Auto-resolves to: `github-pat-{github_user}`
- Expected secret name: `github-pat-swe-1-5dlabs`
- Secret key: `token` (hardcoded convention)

## Task Execution Flow

1. **API receives task** with repository and github_user
2. **ConfigMap created** with all markdown files
3. **Init container**:
   - Mounts ConfigMap at `/config`
   - Copies files to `/workspace/{service}/.task/{task_id}/`
   - Clones repository to `/workspace/{service}/` using `.` (no subdirectory)
   - Sets up git credentials using resolved secret
4. **Main container**:
   - Starts in `/workspace/{service}`
   - Reads CLAUDE.md with @imports to current task only
   - Has GITHUB_TOKEN environment variable set

## Limitations of Current Approach

1. **Path Visibility**: Claude sees `/workspace/auth-service/` not just `/workspace/`
2. **No Automatic Context**: Each task starts fresh unless manually using --continue
3. **Service Name in Path**: May confuse Claude about working directory
4. **Manual Session Tracking**: Need to track session IDs for --continue

## Planned Changes

### SubPath Volume Mounting
- Mount `/workspace/{service}` as `/workspace` in container
- Claude only sees `/workspace/` as root
- Service isolation maintained on disk
- Cleaner working directory for Claude

### Benefits
- Claude always works from repository root perspective
- Service workspaces remain isolated
- No changes needed to PVC structure
- Supports existing multi-task workflows

## Outstanding Questions for Future

1. **Automatic --continue**: Should we automatically use --continue after first task?
2. **CLAUDE.md Updates**: Should we append previous task references vs regenerate?
3. **Session Management**: How to track and reuse session IDs automatically?
4. **Task Context**: Should we include summaries of previous tasks in CLAUDE.md?

---

*This document represents the state before implementing subPath mounting changes.*