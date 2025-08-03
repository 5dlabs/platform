#!/bin/bash
set -euo pipefail

# Setup Argo CD Access for CI/CD
# This script configures access for GitHub Actions to interact with Argo CD

echo "🔐 Setting up Argo CD access for CI/CD..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is required but not found"
    exit 1
fi

# Check if argocd namespace exists
if ! kubectl get namespace argocd > /dev/null 2>&1; then
    echo "❌ argocd namespace not found. Please install Argo CD first:"
    echo "   ./infra/scripts/install-argocd.sh"
    exit 1
fi

# Create CI/CD service account for GitHub Actions
echo "👤 Creating CI/CD service account..."
kubectl apply -f - <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cicd-service-account
  namespace: argocd
  labels:
    app.kubernetes.io/name: cicd-service-account
    app.kubernetes.io/part-of: platform
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: cicd-argocd-role
rules:
  # Argo CD application management
  - apiGroups: ["argoproj.io"]
    resources: ["applications", "appprojects"]
    verbs: ["get", "list", "watch", "update", "patch", "sync"]
  
  # Pod and deployment status checking
  - apiGroups: [""]
    resources: ["pods", "services"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch"]
  
  # TaskRun monitoring
  - apiGroups: ["agents.platform"]
    resources: ["coderuns", "docsruns"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: cicd-argocd-binding
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: cicd-argocd-role
subjects:
  - kind: ServiceAccount
    name: cicd-service-account
    namespace: argocd
EOF

# Create long-lived token for CI/CD
echo "🎫 Creating long-lived token for CI/CD..."
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: cicd-service-account-token
  namespace: argocd
  annotations:
    kubernetes.io/service-account.name: cicd-service-account
type: kubernetes.io/service-account-token
EOF

# Wait for token to be created
echo "⏳ Waiting for token to be generated..."
sleep 5

# Get the token
TOKEN=$(kubectl -n argocd get secret cicd-service-account-token -o jsonpath='{.data.token}' | base64 -d)
CLUSTER_NAME=$(kubectl config current-context)
SERVER_URL=$(kubectl config view --minify -o jsonpath='{.clusters[0].cluster.server}')

echo ""
echo "✅ CI/CD access setup complete!"
echo ""
echo "🔧 Configuration for GitHub Actions:"
echo "Add these as GitHub repository secrets:"
echo ""
echo "ARGOCD_SERVER: ${SERVER_URL}"
echo "ARGOCD_TOKEN: ${TOKEN}"
echo ""
echo "🔄 Alternative: Use Argo CD CLI login in GitHub Actions:"
echo "The deploy-gitops.yml workflow includes login via admin credentials"
echo ""
echo "📋 Next Steps:"
echo "1. Add the secrets to your GitHub repository"
echo "2. Update your GitHub Actions workflow to use the new GitOps deployment"
echo "3. Test the deployment pipeline"

# Optionally create a kubeconfig for CI/CD use
echo ""
echo "📁 Creating kubeconfig for CI/CD (optional)..."
kubectl config view --minify --flatten > /tmp/cicd-kubeconfig
echo "Kubeconfig saved to /tmp/cicd-kubeconfig"
echo "You can add this as KUBECONFIG secret in GitHub Actions if needed"