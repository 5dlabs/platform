#!/bin/bash

# Test script to verify ConfigMap creation for intake workflow

set -e

echo "Testing intake ConfigMap creation..."

# Create test files
TEST_DIR=$(mktemp -d)
echo "Using test directory: $TEST_DIR"

# Create simple test content
cat > "$TEST_DIR/prd.txt" << 'EOF'
# Test PRD
This is a test PRD with some content.
## Features
- Feature 1
- Feature 2
EOF

cat > "$TEST_DIR/architecture.md" << 'EOF'
# Test Architecture
Simple architecture document.
EOF

# Test values
PROJECT_NAME="test-project"
CONFIGMAP_NAME="intake-test-$(date +%s)"
GITHUB_APP="5DLabs-Morgan"
MODEL="claude-opus-4-20250514"

echo "Creating ConfigMap: $CONFIGMAP_NAME"

# Create the ConfigMap
kubectl create configmap "$CONFIGMAP_NAME" \
  -n argo \
  --from-file=prd.txt="$TEST_DIR/prd.txt" \
  --from-file=architecture.md="$TEST_DIR/architecture.md" \
  --from-literal="config.json={\"project_name\":\"$PROJECT_NAME\",\"repository_url\":\"https://github.com/5dlabs/test\",\"github_app\":\"$GITHUB_APP\",\"model\":\"$MODEL\",\"num_tasks\":50,\"expand_tasks\":true,\"analyze_complexity\":true}"

echo "ConfigMap created successfully!"

# Verify it exists
echo "Verifying ConfigMap..."
kubectl get configmap "$CONFIGMAP_NAME" -n argo -o yaml | head -20

# Clean up
echo "Cleaning up..."
kubectl delete configmap "$CONFIGMAP_NAME" -n argo
rm -rf "$TEST_DIR"

echo "Test completed successfully!"
