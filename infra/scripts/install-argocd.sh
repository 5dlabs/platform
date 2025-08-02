#!/bin/bash
set -euo pipefail

# Install Argo CD in the Kubernetes cluster
# This script sets up Argo CD with proper configuration for the platform

echo "🚀 Installing Argo CD..."

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
echo "📁 Creating argocd namespace..."
kubectl apply -f infra/charts/argocd/namespace.yaml

# Add Argo Helm repository
echo "📦 Adding Argo Helm repository..."
helm repo add argo https://argoproj.github.io/argo-helm
helm repo update

# Install secrets (with placeholders - user needs to update)
echo "🔐 Creating repository secrets..."
echo "⚠️  WARNING: Update the secrets in infra/charts/argocd/secrets.yaml with real GitHub credentials!"
kubectl apply -f infra/charts/argocd/secrets.yaml

# Install Argo CD
echo "🎯 Installing Argo CD..."
helm install argocd argo/argo-cd \
  --namespace argocd \
  --create-namespace \
  --values infra/charts/argocd/install-argocd.yaml \
  --timeout 10m \
  --wait

# Wait for deployment to be ready
echo "⏳ Waiting for Argo CD to be ready..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=argocd-server -n argocd --timeout=300s

# Get access information
echo ""
echo "✅ Argo CD installation complete!"
echo ""
echo "🌐 Access Information:"
echo "   NodePort: http://localhost:30080"
echo "   HTTPS NodePort: https://localhost:30443"
echo ""
echo "🔑 Login Credentials:"
echo "   Username: admin"

# Get the admin password
ADMIN_PASSWORD=$(kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" 2>/dev/null | base64 -d 2>/dev/null || echo "admin123")
echo "   Password: $ADMIN_PASSWORD"

echo ""
echo "🔄 Port Forward (alternative access):"
echo "   kubectl port-forward svc/argocd-server -n argocd 8080:443"
echo "   Then access: https://localhost:8080"

echo ""
echo "📋 Next Steps:"
echo "1. Update secrets in infra/charts/argocd/secrets.yaml with real GitHub credentials"
echo "2. Change the default admin password"
echo "3. Configure repository access"
echo "4. Create your first Application"

echo ""
echo "🔍 Verify Installation:"
echo "   kubectl get pods -n argocd"
echo "   kubectl get svc -n argocd"