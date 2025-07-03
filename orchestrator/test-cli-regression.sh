#!/bin/bash
# CLI Regression Test Suite

set -e

# Configuration
CLI_PATH="/Users/jonathonfritz/platform/orchestrator/target/release/orchestrator"
API_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
TEST_DIR="/Users/jonathonfritz/platform/example/todo-api"

echo "========================================="
echo "CLI Regression Test Suite"
echo "========================================="
echo "API URL: $API_URL"
echo "Test Directory: $TEST_DIR"
echo ""

cd "$TEST_DIR"

# Test 1: Health Check
echo "=== Test 1: Health Check ==="
$CLI_PATH --api-url "$API_URL" health
echo ""

# Test 2: Task Submission
echo "=== Test 2: Task Submission ==="
echo "Submitting task 1 (Project Setup and Configuration)..."
TASK_OUTPUT=$(timeout 10 $CLI_PATH --api-url "$API_URL" task submit 1 --service todo-api 2>&1)
echo "$TASK_OUTPUT"

# Extract task ID from output
TASK_ID=$(echo "$TASK_OUTPUT" | grep -oE 'Task ID: [a-f0-9-]+' | cut -d' ' -f3 || echo "")
echo "Submitted Task ID: $TASK_ID"
echo ""

# Test 3: Task Status
if [ -n "$TASK_ID" ]; then
    echo "=== Test 3: Task Status ==="
    $CLI_PATH --api-url "$API_URL" task status "$TASK_ID" || echo "Status check failed"
    echo ""
fi

# Test 4: List Tasks
echo "=== Test 4: List Tasks ==="
$CLI_PATH --api-url "$API_URL" task list || echo "List tasks failed (not implemented yet)"
echo ""

# Test 5: Task with Tools Specification
echo "=== Test 5: Task with Tools Specification ==="
echo "Submitting task 2 with limited tools..."
$CLI_PATH --api-url "$API_URL" task submit 2 --service todo-api --tools "bash:true,read:true,edit:false" || echo "Task with tools failed"
echo ""

# Test 6: Advanced Task Submission
echo "=== Test 6: Advanced Task Submission ==="
echo "Testing advanced submission with explicit paths..."
$CLI_PATH --api-url "$API_URL" task submit-advanced \
    --task "Test advanced task" \
    --service todo-api \
    --design-spec "$TEST_DIR/.taskmaster/docs/design-spec.md" \
    --prompt "$TEST_DIR/.taskmaster/docs/prompt.md" \
    --acceptance-criteria "$TEST_DIR/.taskmaster/docs/acceptance-criteria.md" || echo "Advanced submission not available"
echo ""

# Test 7: Output Formats
echo "=== Test 7: Output Formats ==="
echo "Testing JSON output..."
$CLI_PATH --api-url "$API_URL" --output json health | jq . || echo "JSON output failed"
echo ""

# Test 8: Error Handling
echo "=== Test 8: Error Handling ==="
echo "Testing invalid task ID..."
$CLI_PATH --api-url "$API_URL" task submit 9999 --service todo-api 2>&1 | head -5
echo ""

echo "========================================="
echo "Regression Test Suite Complete"
echo "========================================="