# CLI Regression Test Commands

## Prerequisites

1. **Ensure Twingate VPN is connected**
2. **Set the orchestrator API URL**:
   ```bash
   export ORCHESTRATOR_API_URL=http://orchestrator.orchestrator.svc.cluster.local/api/v1
   ```

## Step 1: Clean the Workspace

First, ensure we have a clean workspace for testing:

```bash
# Clean the test-service workspace using debug pod
kubectl exec -n orchestrator debug-pod -- sh -c 'rm -rf /workspace/test-service/* /workspace/test-service/.*' 2>/dev/null || true

# Verify it's clean
kubectl exec -n orchestrator debug-pod -- ls -la /workspace/test-service/
```

## Step 2: Submit the Test Task

Submit the test task using the CLI with the Task Master directory structure:

```bash
# Navigate to orchestrator directory
cd /Users/jonathonfritz/platform/orchestrator

# Submit the test task
./target/debug/orchestrator task submit 9999 \
  --service test-service \
  --agent claude-agent-1 \
  --taskmaster-dir ./test-taskmaster \
  --repo https://github.com/5dlabs/agent-sandbox \
  --github-user swe-1-5dlabs
```

Expected output:
```
INFO: Preparing task submission...
INFO: Found task: Test File Locations and CLI Integration
SUCCESS: Task 9999 submitted successfully!
INFO: Service: test-service
INFO: Agent: claude-agent-1
{
  "name": "task-9999",
  "namespace": "orchestrator",
  "service": "test-service",
  "task_id": 9999
}
```

## Step 3: Monitor the Task Execution

### Check Task Status
```bash
# Get the TaskRun status
kubectl get taskrun task-9999 -n orchestrator -o jsonpath='{.status}' | jq
```

### Find the Pod
```bash
# Find the pod for the task
kubectl get pods -n orchestrator | grep task9999
```

### Monitor Init Container Logs
```bash
# Replace POD_NAME with the actual pod name from above
kubectl logs POD_NAME -n orchestrator -c prepare-workspace | tail -50
```

Look for:
- ✅ "Git is available"
- ✅ "GitHub CLI is available"
- ✅ "Cloning repository: https://github.com/5dlabs/agent-sandbox"
- ✅ "GitHub token loaded from secret: github-pat-swe-1-5dlabs"
- ✅ "Workspace prepared successfully"

### Monitor Claude Agent Logs
```bash
# Follow the main container logs
kubectl logs -f POD_NAME -n orchestrator --tail=100
```

Watch for Claude to:
1. Print working directory
2. List files in workspace
3. Check git status
4. Create test-results-task-9999.md
5. Commit and push the file

## Step 4: Verify Results

### Check if Claude Created the Test Results File
```bash
# Check the workspace after Claude completes
kubectl exec -n orchestrator debug-pod -- ls -la /workspace/test-service/ | grep test-results
```

### Check the Repository
Visit https://github.com/5dlabs/agent-sandbox to see if:
- The test-results-task-9999.md file was committed
- The commit message is: "Add test results for task 9999 - workspace verification"

## Step 5: Cleanup (Optional)

If you need to rerun the test:

```bash
# Delete the TaskRun
kubectl delete taskrun task-9999 -n orchestrator

# Clean the workspace again
kubectl exec -n orchestrator debug-pod -- sh -c 'rm -rf /workspace/test-service/* /workspace/test-service/.*' 2>/dev/null || true
```

## Success Criteria

The regression test is successful if:
1. ✅ CLI reads markdown files from `./test-taskmaster/docs/`
2. ✅ CLI submits task successfully to the API
3. ✅ Init container clones repository without permission errors
4. ✅ Claude runs with proper tool permissions
5. ✅ Claude creates and commits test-results-task-9999.md
6. ✅ Push to repository succeeds

## Troubleshooting

### If Claude doesn't have tool permissions:
Check the `.claude.json` content - it should have `/workspace` (not `/workspace/test-service`) in the projects section.

### If git clone fails:
- Check init container logs for permission errors
- Verify the GitHub PAT secret exists: `kubectl get secret github-pat-swe-1-5dlabs -n orchestrator`

### If push fails:
- Verify the user swe-1-5dlabs has write access to the repository
- Check Claude's logs for authentication errors