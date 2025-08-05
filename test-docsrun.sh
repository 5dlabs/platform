#!/bin/bash

# Standardized DocsRun workflow test
# This is the ONLY way to test DocsRun workflows

echo "📚 Testing DocsRun workflow with Morgan agent..."

# Clean up any existing jobs first
kubectl delete jobs -n agent-platform -l workflows.argoproj.io/workflow 2>/dev/null || true

# Submit DocsRun workflow using Argo
argo submit --from workflowtemplate/docsrun-template \
  -n agent-platform \
  -p working-directory=projects/market-research \
  -p repository-url=5dlabs/cto \
  -p github-app=5DLabs-Morgan \
  -p github-user="" \
  -p source-branch=argo \
  -p model=claude-opus-4-20250514 \
  --wait

echo "✅ DocsRun workflow test completed"