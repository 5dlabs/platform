#!/bin/bash

# Test script for TaskRun --continue functionality
# This script submits a task, kills it, then retries to test continuation

set -e

export ORCHESTRATOR_API_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
ORCHESTRATOR_CLI="./target/release/orchestrator"

echo "=== Testing TaskRun --continue Functionality ==="

# Submit initial task
echo "1. Submitting initial task..."
TASK_RESPONSE=$($ORCHESTRATOR_CLI --output json task submit 1 --service debug-api --model sonnet)
TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.task_id // .id // "unknown"')

if [ "$TASK_ID" = "unknown" ]; then
    echo "Failed to extract task ID from response: $TASK_RESPONSE"
    exit 1
fi

echo "   Task ID: $TASK_ID"

# Wait for job to start
echo "2. Waiting for job to start..."
sleep 10

# Check TaskRun status  
echo "3. Checking TaskRun status..."
$ORCHESTRATOR_CLI --output json task status "$TASK_ID"

# Get job name to kill it
echo "4. Finding and killing the job..."
JOB_NAME=$(kubectl get taskruns -n orchestrator -l task-id="$TASK_ID" -o jsonpath='{.items[0].status.jobName}')
echo "   Job name: $JOB_NAME"

if [ -n "$JOB_NAME" ]; then
    kubectl delete job "$JOB_NAME" -n orchestrator
    echo "   Job deleted"
else
    echo "   No job found to delete"
fi

# Wait a moment
sleep 5

# Check attempts count
echo "5. Checking attempts count after deletion..."
ATTEMPTS=$(kubectl get taskruns -n orchestrator -l task-id="$TASK_ID" -o jsonpath='{.items[0].status.attempts}')
echo "   Current attempts: $ATTEMPTS"

# Submit retry (should trigger --continue)
echo "6. Submitting retry (should use --continue flag)..."
$ORCHESTRATOR_CLI task submit "$TASK_ID" --service debug-api --model sonnet --retry

# Wait and check logs for --continue flag
echo "7. Waiting for retry job to start..."
sleep 15

# Get new job name
NEW_JOB_NAME=$(kubectl get taskruns -n orchestrator -l task-id="$TASK_ID" -o jsonpath='{.items[0].status.jobName}')
echo "   New job name: $NEW_JOB_NAME"

# Check job logs for --continue flag
echo "8. Checking logs for --continue flag usage..."
if [ -n "$NEW_JOB_NAME" ]; then
    kubectl logs "job/$NEW_JOB_NAME" -n orchestrator -c claude-agent | grep -E "(continue|Adding --continue|attempt)" || echo "No continue flag found in logs"
else
    echo "   No new job found"
fi

# Check final attempts count
echo "9. Final attempts count:"
FINAL_ATTEMPTS=$(kubectl get taskruns -n orchestrator -l task-id="$TASK_ID" -o jsonpath='{.items[0].status.attempts}')
echo "   Final attempts: $FINAL_ATTEMPTS"

# Cleanup
echo "10. Cleaning up..."
kubectl delete taskruns -n orchestrator -l task-id="$TASK_ID"

echo "=== Test Complete ==="
echo "Expected behavior:"
echo "- Initial attempt should have attempts=1"
echo "- Retry should have attempts=2 and use --continue flag"
echo "- Directory structure should be /workspace/debug-api/.task/$TASK_ID/ (no run-X subdirs)"