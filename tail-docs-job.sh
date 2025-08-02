#!/bin/bash

# Tail docs job logs until completion
DOCS_JOB="docs-gen-1754094230"
LOG_FILE="docs-job-complete-logs.txt"

echo "📋 Tailing docs job logs: $DOCS_JOB"
echo "📁 Saving to: $LOG_FILE"
echo "🔄 Starting log capture..."

# Find the pod
POD_NAME=$(kubectl -n orchestrator get pods | grep $DOCS_JOB | awk '{print $1}')

if [ -z "$POD_NAME" ]; then
    echo "❌ Pod not found for job: $DOCS_JOB"
    exit 1
fi

echo "📦 Found pod: $POD_NAME"

# Clear any existing log file
> "$LOG_FILE"

# Tail logs until pod completes
echo "🚀 Starting log tail (this will run until job completes)..."
kubectl -n orchestrator logs -f "$POD_NAME" >> "$LOG_FILE" 2>&1

echo ""
echo "✅ Job completed! Logs saved to: $LOG_FILE"
echo "📊 Check job status with: kubectl -n orchestrator get docsrun $DOCS_JOB"