#!/bin/bash
set -euo pipefail

# Install Argo Workflows in the Kubernetes cluster
# This script sets up Argo Workflows with proper configuration for the platform

echo "🚀 Installing Argo Workflows..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is required but not found"
    exit 1
fi

# Check if helm is available
if ! command -v helm &> /dev/null; then
    echo "❌ helm is required but not found"
    exit 1
fi

# Create namespace
echo "📁 Creating argo namespace..."
kubectl apply -f infra/charts/argo-workflows/namespace.yaml

# Add Argo Helm repository if not already added
echo "📦 Adding Argo Helm repository..."
helm repo add argo https://argoproj.github.io/argo-helm 2>/dev/null || true
helm repo update

# Apply RBAC and artifact configuration
echo "🔐 Setting up RBAC and artifact storage..."
kubectl apply -f infra/charts/argo-workflows/rbac.yaml
kubectl apply -f infra/charts/argo-workflows/artifact-config.yaml

# Install Argo Workflows
echo "🎯 Installing Argo Workflows..."
helm install argo-workflows argo/argo-workflows \
  --namespace argo \
  --create-namespace \
  --values infra/charts/argo-workflows/install-argo-workflows.yaml \
  --timeout 10m \
  --wait

# Wait for deployment to be ready
echo "⏳ Waiting for Argo Workflows to be ready..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=argo-workflows-server -n argo --timeout=300s

# Get access information
echo ""
echo "✅ Argo Workflows installation complete!"
echo ""
echo "🌐 Access Information:"
echo "   NodePort: http://localhost:30081"
echo ""
echo "🔄 Port Forward (alternative access):"
echo "   kubectl port-forward svc/argo-workflows-server -n argo 2746:2746"
echo "   Then access: https://localhost:2746"

echo ""
echo "📋 Next Steps:"
echo "1. Configure artifact storage (update secrets if using S3/MinIO)"
echo "2. Create your first WorkflowTemplate"
echo "3. Test workflow execution"

echo ""
echo "🔍 Verify Installation:"
echo "   kubectl get pods -n argo"
echo "   kubectl get svc -n argo"
echo ""
echo "📖 Create a test workflow:"
echo "   kubectl apply -f - <<EOF"
echo "apiVersion: argoproj.io/v1alpha1"
echo "kind: Workflow"
echo "metadata:"
echo "  generateName: hello-world-"
echo "  namespace: argo"
echo "spec:"
echo "  entrypoint: hello"
echo "  templates:"
echo "  - name: hello"
echo "    container:"
echo "      image: alpine:latest"
echo "      command: [sh, -c]"
echo "      args: ['echo \"Hello World\"']"
echo "EOF"