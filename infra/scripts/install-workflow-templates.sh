#!/bin/bash
set -euo pipefail

# Install Argo Workflow Templates for Platform Tasks
# This script deploys the workflow templates for CodeRun and DocsRun

echo "🎯 Installing Argo Workflow Templates..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is required but not found"
    exit 1
fi

# Check if argo namespace exists
if ! kubectl get namespace argo > /dev/null 2>&1; then
    echo "❌ argo namespace not found. Please install Argo Workflows first:"
    echo "   ./infra/scripts/install-argo-workflows.sh"
    exit 1
fi

# Apply RBAC for workflow templates
echo "🔐 Setting up workflow template RBAC..."
kubectl apply -f infra/workflow-templates/workflow-rbac.yaml

# Apply workflow templates
echo "📋 Installing CodeRun workflow template..."
kubectl apply -f infra/workflow-templates/coderun-template.yaml

echo "📚 Installing DocsRun workflow template..."
kubectl apply -f infra/workflow-templates/docsrun-template.yaml

# Verify installation
echo "🔍 Verifying workflow template installation..."
kubectl get workflowtemplates -n argo

echo ""
echo "✅ Workflow templates installation complete!"
echo ""
echo "📋 Available Templates:"
echo "   - coderun-template: For code implementation tasks"
echo "   - docsrun-template: For documentation generation tasks"
echo ""
echo "🔄 Next Steps:"
echo "1. Update orchestrator to use workflow submission instead of direct jobs"
echo "2. Test workflow execution with sample tasks"
echo "3. Monitor workflow execution in Argo UI"
echo ""
echo "📖 Test a CodeRun workflow:"
echo "   kubectl create -f - <<EOF"
echo "apiVersion: argoproj.io/v1alpha1"
echo "kind: Workflow"
echo "metadata:"
echo "  generateName: test-coderun-"
echo "  namespace: argo"
echo "spec:"
echo "  workflowTemplateRef:"
echo "    name: coderun-template"
echo "  arguments:"
echo "    parameters:"
echo "      - name: task-id"
echo "        value: \"test-123\""
echo "      - name: service-name"
echo "        value: \"test-service\""
echo "      - name: repository"
echo "        value: \"5dlabs/cto\""
echo "      - name: docs-repository"
echo "        value: \"5dlabs/cto\""
echo "      - name: github-app"
echo "        value: \"5DLabs-Rex\""
echo "      - name: anthropic-api-key"
echo "        value: \"your-api-key\""
echo "EOF"