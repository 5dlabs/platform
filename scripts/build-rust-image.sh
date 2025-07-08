#!/bin/bash
set -euo pipefail

# Build and push pre-staged Rust builder image
# This image contains all tools and dependencies for fast Rust builds

REGISTRY="ghcr.io"
REPO="5dlabs/agent-platform"
IMAGE_NAME="rust-builder"
TAG="${1:-latest}"

echo "🏗️ Building pre-staged Rust builder image..."
echo "Registry: $REGISTRY"
echo "Repository: $REPO"
echo "Image: $IMAGE_NAME"
echo "Tag: $TAG"

# Build the image
docker build \
  -t "$REGISTRY/$REPO/$IMAGE_NAME:$TAG" \
  -f infra/images/rust-builder/Dockerfile \
  .

echo "✅ Image built successfully!"

# Push the image
echo "📤 Pushing image to registry..."
docker push "$REGISTRY/$REPO/$IMAGE_NAME:$TAG"

# Also tag as latest if not already
if [ "$TAG" != "latest" ]; then
  docker tag "$REGISTRY/$REPO/$IMAGE_NAME:$TAG" "$REGISTRY/$REPO/$IMAGE_NAME:latest"
  docker push "$REGISTRY/$REPO/$IMAGE_NAME:latest"
fi

echo "✅ Image pushed successfully!"
echo ""
echo "📋 Image details:"
echo "  Full name: $REGISTRY/$REPO/$IMAGE_NAME:$TAG"
echo "  Size: $(docker images --format 'table {{.Size}}' "$REGISTRY/$REPO/$IMAGE_NAME:$TAG" | tail -1)"
echo ""
echo "🚀 Ready to use in Arc runners!"