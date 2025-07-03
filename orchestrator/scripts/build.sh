#!/bin/bash

# Build script for orchestrator
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
IMAGE_NAME="orchestrator"
TAG="latest"
REGISTRY=""
PUSH=false
BUILD_ONLY=false

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build and optionally push the orchestrator Docker image.

OPTIONS:
    -h, --help          Show this help message
    -t, --tag TAG       Set the image tag (default: latest)
    -r, --registry REG  Set the registry prefix (e.g., ghcr.io/username)
    -p, --push          Push the image after building
    -b, --build-only    Only build, don't run any tests
    --no-cache          Build without using cache

EXAMPLES:
    $0                                  # Build locally with tag 'latest'
    $0 -t v1.0.0                       # Build with tag 'v1.0.0'
    $0 -r ghcr.io/myuser -t v1.0.0 -p  # Build, tag, and push to registry
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        -b|--build-only)
            BUILD_ONLY=true
            shift
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Construct full image name
if [[ -n "$REGISTRY" ]]; then
    FULL_IMAGE_NAME="$REGISTRY/$IMAGE_NAME:$TAG"
else
    FULL_IMAGE_NAME="$IMAGE_NAME:$TAG"
fi

print_status "Building orchestrator Docker image..."
print_status "Image name: $FULL_IMAGE_NAME"

# Run tests first (unless build-only is specified)
if [[ "$BUILD_ONLY" != true ]]; then
    print_status "Running cargo tests..."
    cargo test --verbose || {
        print_error "Tests failed!"
        exit 1
    }

    print_status "Running cargo clippy..."
    cargo clippy --all-targets --all-features -- -D warnings || {
        print_error "Clippy checks failed!"
        exit 1
    }

    print_status "Checking code formatting..."
    cargo fmt --all -- --check || {
        print_error "Code formatting check failed! Run 'cargo fmt' to fix."
        exit 1
    }
fi

# Build Docker image
print_status "Building Docker image..."

# Determine build command based on available tools and push flag
if command -v docker buildx &> /dev/null; then
    if [[ "$PUSH" == true ]]; then
        if [[ -z "$REGISTRY" ]]; then
            print_error "Cannot push without registry specified. Use -r/--registry option."
            exit 1
        fi
        print_status "Using buildx for multi-platform build and push..."
        docker buildx build ${NO_CACHE:-} --platform linux/amd64,linux/arm64 -t "$FULL_IMAGE_NAME" --push . || {
            print_error "Docker buildx build/push failed!"
            exit 1
        }
        PUSHED=true
    else
        print_status "Using buildx for local build (current platform only)..."
        docker buildx build ${NO_CACHE:-} --load -t "$FULL_IMAGE_NAME" . || {
            print_error "Docker buildx build failed!"
            exit 1
        }
        PUSHED=false
    fi
else
    print_warning "Docker buildx not available, building for current platform only"
    docker build ${NO_CACHE:-} -t "$FULL_IMAGE_NAME" . || {
        print_error "Docker build failed!"
        exit 1
    }
    PUSHED=false
fi

print_status "Docker image built successfully: $FULL_IMAGE_NAME"

# Test the built image
print_status "Testing the built image..."
if docker run --rm --name orchestrator-test -d -p 8080:8080 "$FULL_IMAGE_NAME"; then
    sleep 3
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        print_status "Health check passed!"
        docker stop orchestrator-test > /dev/null 2>&1 || true
    else
        print_warning "Health check failed, but continuing..."
        docker stop orchestrator-test > /dev/null 2>&1 || true
    fi
else
    print_warning "Could not start container for testing, but image was built successfully"
fi

# Push if requested (and not already pushed by buildx)
if [[ "$PUSH" == true ]] && [[ "$PUSHED" != true ]]; then
    print_status "Pushing image to registry..."
    docker push "$FULL_IMAGE_NAME" || {
        print_error "Docker push failed!"
        exit 1
    }
    print_status "Image pushed successfully: $FULL_IMAGE_NAME"
elif [[ "$PUSHED" == true ]]; then
    print_status "Image already pushed during buildx build: $FULL_IMAGE_NAME"
fi

print_status "Build completed successfully!"