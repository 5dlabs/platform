#!/bin/bash
set -e

# Configuration
IMAGE_NAME="grok-agent"
IMAGE_TAG="latest"
REGISTRY="${REGISTRY:-}"

echo "🔨 Building Grok Agent Docker image..."

# Build the image
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

# Tag for registry if specified
if [ -n "$REGISTRY" ]; then
    echo "🏷️  Tagging for registry: $REGISTRY"
    docker tag "${IMAGE_NAME}:${IMAGE_TAG}" "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"

    if [ "$1" = "--push" ]; then
        echo "📤 Pushing to registry..."
        docker push "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"
    fi
fi

echo "✅ Grok Agent image built successfully"
echo "📦 Image: ${IMAGE_NAME}:${IMAGE_TAG}"

if [ -n "$REGISTRY" ]; then
    echo "📦 Registry image: ${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"
fi