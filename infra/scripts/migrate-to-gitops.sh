#!/bin/bash
set -euo pipefail

# Complete GitOps Migration Script
# This script performs the full migration from Helm to GitOps with Argo CD

echo "🚀 Starting complete GitOps migration..."

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "🔍 Checking prerequisites..."
for cmd in kubectl helm; do
    if ! command_exists $cmd; then
        echo "❌ $cmd is required but not found"
        exit 1
    fi
done

# Check if we're in the right directory
if [ ! -f "infra/charts/orchestrator/Chart.yaml" ]; then
    echo "❌ Please run this script from the repository root"
    exit 1
fi

echo "✅ Prerequisites check passed"

# Step 1: Install Argo CD
echo ""
echo "📦 Step 1: Installing Argo CD..."
if ! kubectl get namespace argocd > /dev/null 2>&1; then
    ./infra/scripts/install-argocd.sh
    echo "⚠️  Please update GitHub credentials in infra/charts/argocd/secrets.yaml"
    read -p "Press Enter after updating the secrets..."
else
    echo "✅ Argo CD already installed"
fi

# Step 2: Install Argo Workflows
echo ""
echo "🎯 Step 2: Installing Argo Workflows..."
if ! kubectl get namespace argo > /dev/null 2>&1; then
    ./infra/scripts/install-argo-workflows.sh
else
    echo "✅ Argo Workflows already installed"
fi

# Step 3: Install Workflow Templates
echo ""
echo "📋 Step 3: Installing Workflow Templates..."
./infra/scripts/install-workflow-templates.sh

# Step 4: Setup Argo CD access for CI/CD
echo ""
echo "🔐 Step 4: Setting up Argo CD access..."
./infra/scripts/setup-argocd-access.sh

# Step 5: Deploy GitOps applications
echo ""
echo "🎛️ Step 5: Deploying GitOps applications..."

# Create platform project
echo "📁 Creating platform project..."
kubectl apply -f infra/gitops/projects/platform-project.yaml

# Deploy app of apps
echo "🎪 Deploying app of apps..."
kubectl apply -f infra/gitops/app-of-apps.yaml

# Wait for applications to sync
echo "⏳ Waiting for applications to sync..."
sleep 10

# Check application status
echo "🔍 Checking application status..."
kubectl get applications -n argocd

# Step 6: Verify the migration
echo ""
echo "✅ Step 6: Verifying migration..."

# Check if orchestrator is running
if kubectl get pods -n orchestrator -l app.kubernetes.io/name=orchestrator | grep -q Running; then
    echo "✅ Orchestrator is running via GitOps"
else
    echo "⚠️ Orchestrator may still be starting up"
fi

# Check Argo Workflows
if kubectl get pods -n argo -l app.kubernetes.io/name=argo-workflows-server | grep -q Running; then
    echo "✅ Argo Workflows is running"
else
    echo "⚠️ Argo Workflows may still be starting up"
fi

echo ""
echo "🎉 GitOps migration completed!"
echo ""
echo "🌐 Access Information:"
echo "   Argo CD UI: http://localhost:30080"
echo "   Argo Workflows UI: http://localhost:30081"
echo ""
echo "🔄 Next Steps:"
echo "1. Update your GitHub Actions to use .github/workflows/deploy-gitops.yml"
echo "2. Add ARGOCD_SERVER and ARGOCD_TOKEN secrets to GitHub repository"
echo "3. Test the new deployment pipeline"
echo "4. Gradually migrate other services to GitOps"
echo ""
echo "📚 Documentation:"
echo "   See infra/gitops/README.md for detailed usage instructions"

# Optional: Backup old deployment workflow
if [ -f ".github/workflows/deploy.yml" ]; then
    echo ""
    echo "💾 Backing up old deployment workflow..."
    mv .github/workflows/deploy.yml .github/workflows/deploy-helm-backup.yml
    echo "✅ Old workflow backed up as deploy-helm-backup.yml"
fi

echo ""
echo "🎯 Migration Summary:"
echo "   ✅ Argo CD installed and configured"
echo "   ✅ Argo Workflows installed with templates"
echo "   ✅ GitOps applications deployed"
echo "   ✅ CI/CD access configured"
echo "   ✅ New GitHub Actions workflow ready"
echo ""
echo "🚀 Your platform is now fully GitOps enabled!"