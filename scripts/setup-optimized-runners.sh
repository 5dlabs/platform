#!/bin/bash
set -euo pipefail

# Setup optimized Arc runners with persistent cache
# This script deploys the enhanced runner configuration

echo "🚀 Setting up optimized Arc runners..."

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check kubectl
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl not found. Please install kubectl first."
    exit 1
fi

# Check cluster connection
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster. Please check your kubeconfig."
    exit 1
fi

# Check if local-path storage class exists
if ! kubectl get storageclass local-path &> /dev/null; then
    echo "⚠️ local-path storage class not found. Please ensure your local storage provisioner is installed."
    echo "You can install it with: kubectl apply -f https://raw.githubusercontent.com/rancher/local-path-provisioner/master/deploy/local-path-storage.yaml"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if GitHub token secret exists
if ! kubectl get secret arc-github-token -n arc-systems-optimized &> /dev/null; then
    echo "⚠️ GitHub token secret not found."
    echo "Please create it with:"
    echo "  kubectl create secret generic arc-github-token --from-literal=github_token=YOUR_TOKEN -n arc-systems-optimized"
    echo ""
    echo "The token needs the following permissions:"
    echo "  - admin:org (for organization-level runners)"
    echo "  - repo (for repository access)"
    echo ""
    read -p "Do you want to create the namespace and continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kubectl create namespace arc-systems-optimized || true
        echo "Please create the secret manually and re-run this script."
        exit 1
    else
        exit 1
    fi
fi

echo "✅ Prerequisites check passed!"

# Deploy the optimized runners
echo "📦 Deploying optimized Arc runners..."
kubectl apply -f infra/arc/arc-org-runners-optimized.yaml

echo "⏳ Waiting for persistent volumes to be ready..."
kubectl wait --for=condition=Bound pvc/rust-cache-shared -n arc-systems-optimized --timeout=60s
kubectl wait --for=condition=Bound pvc/cargo-registry-shared -n arc-systems-optimized --timeout=60s
kubectl wait --for=condition=Bound pvc/sccache-shared -n arc-systems-optimized --timeout=60s

echo "⏳ Waiting for runners to be ready..."
kubectl wait --for=condition=Available deployment/org-runners-optimized -n arc-systems-optimized --timeout=300s

echo "✅ Optimized Arc runners deployed successfully!"

# Show status
echo ""
echo "📊 Runner Status:"
kubectl get pods -n arc-systems-optimized -o wide

echo ""
echo "💾 Storage Status:"
kubectl get pvc -n arc-systems-optimized

echo ""
echo "🎯 Optimization Features Enabled:"
echo "  ✅ Pre-staged Rust toolchain"
echo "  ✅ Persistent NVMe cache (100Gi total)"
echo "  ✅ Mold linker for fast linking"
echo "  ✅ Enhanced resources (8 CPU, 16Gi RAM)"
echo "  ✅ Multi-core parallel builds"
echo "  ✅ sccache compilation cache"
echo ""
echo "🚀 Ready for sub-30 second Rust builds!"
echo ""
echo "Next steps:"
echo "1. Build the pre-staged image: ./scripts/build-rust-image.sh"
echo "2. Update your workflow to use: runs-on: [self-hosted, rust-optimized, nvme-cache]"
echo "3. Monitor build times and cache hit rates"