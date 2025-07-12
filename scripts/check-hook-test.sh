#!/bin/bash

# Quick script to check if the early hook test is working
# Usage: ./scripts/check-hook-test.sh [namespace] [service-name]

NAMESPACE=${1:-default}
SERVICE_NAME=${2:-platform}

echo "🔍 Checking for hook test files in docs generation job..."
echo "Namespace: $NAMESPACE"
echo "Service: $SERVICE_NAME"

# Find the docs generation job (task ID 999999)
DOCS_JOB=$(kubectl get jobs -n "$NAMESPACE" -l "orchestrator.io/service-name=$SERVICE_NAME,orchestrator.io/task-id=999999" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

if [ -z "$DOCS_JOB" ]; then
    echo "❌ No docs generation job found for service '$SERVICE_NAME'"
    echo "Looking for jobs with task-id=999999..."
    kubectl get jobs -n "$NAMESPACE" -l "orchestrator.io/task-id=999999" 2>/dev/null || echo "No docs jobs found at all"
    exit 1
fi

echo "✅ Found docs job: $DOCS_JOB"

# Get the pod name
POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l "job-name=$DOCS_JOB" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

if [ -z "$POD_NAME" ]; then
    echo "❌ No pod found for job '$DOCS_JOB'"
    exit 1
fi

echo "✅ Found pod: $POD_NAME"

# Check pod status
POD_STATUS=$(kubectl get pod -n "$NAMESPACE" "$POD_NAME" -o jsonpath='{.status.phase}' 2>/dev/null)
echo "Pod status: $POD_STATUS"

# Try to check for hook test files
echo ""
echo "🧪 Checking for hook test files..."

# Check for hook test success file
echo "Checking for .hook-test-success file..."
if kubectl exec -n "$NAMESPACE" "$POD_NAME" -- test -f /workspace/.hook-test-success 2>/dev/null; then
    echo "✅ Hook test success file found!"
    echo "Contents:"
    kubectl exec -n "$NAMESPACE" "$POD_NAME" -- cat /workspace/.hook-test-success 2>/dev/null || echo "Could not read file contents"
else
    echo "❌ Hook test success file not found yet"
fi

echo ""

# Check for hook log file
echo "Checking for .hook-test.log file..."
if kubectl exec -n "$NAMESPACE" "$POD_NAME" -- test -f /workspace/.hook-test.log 2>/dev/null; then
    echo "✅ Hook log file found!"
    echo "Contents:"
    kubectl exec -n "$NAMESPACE" "$POD_NAME" -- cat /workspace/.hook-test.log 2>/dev/null || echo "Could not read file contents"
else
    echo "❌ Hook log file not found yet"
fi

echo ""

# Check workspace contents
echo "📁 Current workspace contents:"
kubectl exec -n "$NAMESPACE" "$POD_NAME" -- ls -la /workspace/ 2>/dev/null || echo "Could not list workspace contents"

echo ""
echo "🔍 Hook test check complete!"

# If neither file exists and pod is running, hooks may not be firing
if ! kubectl exec -n "$NAMESPACE" "$POD_NAME" -- test -f /workspace/.hook-test-success 2>/dev/null && \
   ! kubectl exec -n "$NAMESPACE" "$POD_NAME" -- test -f /workspace/.hook-test.log 2>/dev/null && \
   [ "$POD_STATUS" = "Running" ]; then
    echo ""
    echo "⚠️  WARNING: Pod is running but no hook test files found"
    echo "   This may indicate hooks are not firing properly"
    echo "   Check Claude settings and hook configuration"
fi