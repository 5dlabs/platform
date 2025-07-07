# Orchestrator Scripts

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