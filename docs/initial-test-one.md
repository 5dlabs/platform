# Initial Test One: Repository Cloning, File Locations, and GitHub Permissions

## Test Objectives

1. **Verify Repository Cloning Location**: Ensure repositories are cloned to the correct workspace directory
2. **Verify Markdown File Locations**: Confirm task files are properly placed and accessible via @imports
3. **Verify GitHub Permissions**: Validate that the simplified auth system correctly resolves secrets and provides access

## Pre-Test Cleanup Process

### Option 1: Delete and Recreate the PVC (Clean Slate)

```bash
# 1. Delete all TaskRuns to ensure no jobs are using the PVC
kubectl delete taskruns --all -n orchestrator

# 2. Delete any running jobs
kubectl delete jobs --all -n orchestrator

# 3. Delete the PVC to clean workspace
kubectl delete pvc shared-workspace -n orchestrator

# 4. Recreate the PVC
kubectl apply -f - <<EOF
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: shared-workspace
  namespace: orchestrator
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: local-path
EOF

# 5. Verify PVC is bound
kubectl get pvc -n orchestrator
```

### Option 2: Run a Cleanup Job

```bash
# Create a cleanup job to wipe the workspace
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: workspace-cleanup-$(date +%s)
  namespace: orchestrator
spec:
  template:
    spec:
      containers:
      - name: cleanup
        image: alpine:latest
        command: ["/bin/sh", "-c"]
        args:
          - |
            echo "Cleaning workspace..."
            rm -rf /workspace/*
            rm -rf /workspace/.*
            echo "Workspace cleaned"
            ls -la /workspace/
        volumeMounts:
        - name: workspace
          mountPath: /workspace
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: shared-workspace
      restartPolicy: Never
EOF

# Wait for cleanup to complete
kubectl wait --for=condition=complete job/workspace-cleanup-* -n orchestrator --timeout=60s

# Delete the cleanup job
kubectl delete jobs -l job-name=workspace-cleanup -n orchestrator
```

## Test Configuration

### 1. Create Test Task JSON

Create a file named `test-file-locations.json`:

```json
{
  "id": 9999,
  "title": "Test File Locations and Permissions",
  "description": "Verify repository cloning, file locations, and GitHub authentication",
  "details": "This is a test task to verify our simplified authentication system",
  "test_strategy": "Manual verification of file system state",
  "priority": "high",
  "dependencies": [],
  "status": "pending",
  "subtasks": [],
  "service_name": "test-service",
  "agent_name": "claude-agent-1",
  "model": "sonnet",
  "markdown_files": [
    {
      "filename": "task.md",
      "content": "# Test Task: Verify File Locations\n\n## Instructions\n\n1. Print current working directory\n2. List all files in /workspace/\n3. List all files in service directory\n4. Show contents of .task directory\n5. Verify git repository status\n6. Test GitHub authentication by fetching repo info\n\n## Commands to Run\n\n```bash\n# Show current directory\npwd\n\n# List workspace root\nls -la /workspace/\n\n# List service directory\nls -la /workspace/test-service/\n\n# Show task files\nls -la /workspace/test-service/.task/9999/\n\n# Check git status\ncd /workspace/test-service && git status\n\n# Test GitHub access\ncd /workspace/test-service && git remote -v\n```\n\n## Expected Results\n\n- Working directory should be `/workspace/test-service`\n- Repository should be cloned to `/workspace/test-service/`\n- Task files should be in `/workspace/test-service/.task/9999/`\n- Git should show clean working tree\n- GitHub remote should be accessible",
      "file_type": "task"
    },
    {
      "filename": "verification.md",
      "content": "# Verification Checklist\n\n- [ ] Repository cloned to correct location\n- [ ] No subpath - repo root is workspace root\n- [ ] Task files accessible via @imports\n- [ ] Git credentials configured\n- [ ] GitHub token available in environment\n- [ ] Can fetch from remote repository",
      "file_type": "context"
    }
  ],
  "agent_tools": [
    {
      "name": "bash",
      "enabled": true,
      "config": {},
      "restrictions": []
    },
    {
      "name": "read",
      "enabled": true,
      "config": {},
      "restrictions": []
    }
  ],
  "repository": {
    "url": "https://github.com/5dlabs/agent-sandbox",
    "branch": "main",
    "github_user": "swe-1-5dlabs"
  }
}
```

### 2. Submit the Test Task

```bash
# Submit via API
curl -X POST http://orchestrator.orchestrator.svc.cluster.local/api/v1/pm/taskruns \
  -H "Content-Type: application/json" \
  -d @test-file-locations.json

# Or via CLI (when available)
orchestrator task submit 9999 --service test-service \
  --repo https://github.com/5dlabs/agent-sandbox \
  --github-user swe-1-5dlabs
```

### 3. Monitor Task Execution

```bash
# Watch the TaskRun status
kubectl get taskruns -n orchestrator -w

# Check job logs
kubectl logs -n orchestrator job/claude-agent-1-test-service-task9999-attempt1 -c init

# Check main container logs
kubectl logs -n orchestrator job/claude-agent-1-test-service-task9999-attempt1 -c main
```

## Expected Outcomes

### 1. Repository Location
- Repository should be cloned directly to `/workspace/test-service/`
- NOT to `/workspace/test-service/agent-test/` or any subdirectory

### 2. Task File Locations
```
/workspace/test-service/
├── .git/                    # Git repository
├── .task/
│   └── 9999/
│       ├── task.md          # Task description
│       └── verification.md  # Verification checklist
├── .claude/                 # Claude configuration
│   └── settings.json
├── .gitignore              # Prevent internal file commits
├── CLAUDE.md               # Main task file with @imports
└── [repository files]      # Files from the cloned repo
```

### 3. GitHub Authentication
- Secret name should auto-resolve to `github-pat-swe-1-5dlabs`
- Git should be configured with credentials
- `gh` CLI should be authenticated
- `$GITHUB_TOKEN` environment variable should be set

## Verification Steps

1. **Check Init Container Logs**:
   - Should show "GitHub token loaded from secret: github-pat-swe-1-5dlabs"
   - Should show successful repository clone
   - Should show task files being copied

2. **Check Main Container Logs**:
   - Working directory should be `/workspace/test-service`
   - Should have access to all task files
   - Git commands should work without authentication prompts

3. **Check for Common Issues**:
   - No "path" subdirectory created
   - No authentication failures
   - No missing task files
   - No permission errors

## Troubleshooting

### If repository is in wrong location:
- Check init script for any path manipulation
- Verify no subpath is being used

### If authentication fails:
- Verify secret exists: `kubectl get secret github-pat-swe-1-5dlabs -n orchestrator`
- Check secret contains token: `kubectl get secret github-pat-swe-1-5dlabs -n orchestrator -o yaml`
- Verify token has correct permissions

### If task files are missing:
- Check ConfigMap was created: `kubectl get configmap -n orchestrator | grep task9999`
- Verify init container completed successfully
- Check file permissions in workspace

## Post-Test Cleanup

After test completion:

```bash
# Delete the test TaskRun
kubectl delete taskrun task-9999 -n orchestrator

# Optional: Clean workspace for next test
# Use Option 1 or Option 2 from Pre-Test Cleanup
```

## Notes for Future API Enhancement

Consider adding an API endpoint for workspace management:
- `POST /api/v1/workspace/clean` - Clean all workspaces
- `POST /api/v1/workspace/{service}/clean` - Clean specific service workspace
- `GET /api/v1/workspace/{service}/status` - Check workspace state

This would eliminate the need for manual cleanup jobs.