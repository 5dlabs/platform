#!/bin/bash
set -euo pipefail

# Deploy optimized Arc runners (assumes image is already built)

echo "🚀 Deploying optimized Arc runners..."

# Copy GitHub token from existing namespace
echo "📋 Copying GitHub token secret..."
kubectl get secret arc-github-token -n arc-systems -o yaml | \
  sed 's/namespace: arc-systems/namespace: arc-systems-optimized/' | \
  kubectl apply -f -

# Deploy the optimized runners
echo "📦 Deploying optimized Arc runners configuration..."
kubectl apply -f infra/arc/arc-org-runners-optimized.yaml

echo "⏳ Waiting for persistent volumes..."
kubectl wait --for=condition=Bound pvc/rust-cache-shared -n arc-systems-optimized --timeout=60s || true
kubectl wait --for=condition=Bound pvc/cargo-registry-shared -n arc-systems-optimized --timeout=60s || true
kubectl wait --for=condition=Bound pvc/sccache-shared -n arc-systems-optimized --timeout=60s || true

echo "⏳ Waiting for runners to be ready..."
sleep 30  # Give runners time to start

echo "📊 Current status:"
kubectl get pods -n arc-systems-optimized
echo ""
kubectl get pvc -n arc-systems-optimized

echo ""
echo "✅ Optimized runners deployed!"
echo "🎯 Features enabled:"
echo "  - Pre-staged Rust image: ghcr.io/5dlabs/agent-platform/rust-builder:latest"
echo "  - Persistent NVMe cache: 100Gi total"
echo "  - Enhanced resources: 8 CPU, 16Gi RAM"
echo "  - Runner labels: [self-hosted, rust-optimized, nvme-cache]"
echo ""
echo "🚀 Ready to test with optimized workflow!"