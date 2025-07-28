# Task 6: Create Dockerfile with Multi-Stage Build

## Overview
This task implements containerization for the Hello World API using Docker best practices. It creates a production-ready Docker image with multi-stage builds, security hardening, health checks, and automated build/test scripts. The goal is to produce a minimal, secure container image under 200MB.

## Objectives
- Create optimized Docker image using multi-stage builds
- Implement security best practices (non-root user)
- Configure health checks for container orchestration
- Minimize image size while maintaining functionality
- Create automated build and test scripts
- Ensure graceful shutdown handling in containers

## Technical Approach

### Containerization Strategy
The implementation uses a two-stage build process:
1. **Build Stage**: Install dependencies in a clean environment
2. **Runtime Stage**: Copy only production artifacts for minimal image size

### Security Measures
- Run as non-root user (nodejs:1001)
- Minimal base image (node:18-alpine)
- No development dependencies in final image
- Proper file ownership settings

### Health Monitoring
- Built-in HEALTHCHECK directive
- Configurable intervals and retries
- Integration with orchestration platforms

## Implementation Details

### Step 1: Create .dockerignore File
```
node_modules
npm-debug.log
.git
.gitignore
.env
.vscode
coverage
.DS_Store
```

### Step 2: Create Multi-Stage Dockerfile
```dockerfile
# Build stage
FROM node:18-alpine AS build

WORKDIR /app

# Copy package files and install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Runtime stage
FROM node:18-alpine

# Set environment variables
ENV NODE_ENV=production
ENV PORT=3000

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001 -G nodejs

WORKDIR /app

# Copy from build stage
COPY --from=build --chown=nodejs:nodejs /app/node_modules ./node_modules

# Copy application code
COPY --chown=nodejs:nodejs . .

# Switch to non-root user
USER nodejs

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Start application
CMD ["node", "src/server.js"]
```

### Step 3: Create Build Script (scripts/docker-build.sh)
```bash
#!/bin/bash
set -e

IMAGE_NAME="hello-world-api"
IMAGE_TAG="latest"

echo "Building Docker image: ${IMAGE_NAME}:${IMAGE_TAG}"
docker build -t ${IMAGE_NAME}:${IMAGE_TAG} .

echo "Image size:"
docker images ${IMAGE_NAME}:${IMAGE_TAG} --format "{{.Size}}"

echo "Running container for testing..."
docker run -d --name hello-world-api-test -p 3000:3000 ${IMAGE_NAME}:${IMAGE_TAG}

echo "Waiting for container to start..."
sleep 3

echo "Testing health endpoint..."
RESPONSE=$(curl -s http://localhost:3000/health)
echo $RESPONSE

echo "Stopping and removing test container..."
docker stop hello-world-api-test
docker rm hello-world-api-test

echo "Docker build and test complete!"
```

### Step 4: Set Script Permissions
```bash
chmod +x scripts/docker-build.sh
```

### Key Implementation Details

#### Multi-Stage Build Benefits
- **Smaller Image Size**: Only production dependencies included
- **Build Cache Optimization**: Dependencies cached separately
- **Security**: Build tools not included in final image
- **Reproducibility**: Clean build environment

#### Alpine Linux Choice
- Minimal base image (~5MB)
- Security-focused distribution
- Includes only essential packages
- Regular security updates

#### Health Check Configuration
- **Interval**: 30s between checks
- **Timeout**: 5s per check attempt
- **Start Period**: 5s grace period on startup
- **Retries**: 3 attempts before unhealthy

#### Non-Root User Setup
- User ID 1001 (common convention)
- Dedicated nodejs group
- Proper file ownership with --chown
- Prevents privilege escalation

## Dependencies and Requirements
- Docker installed on build system
- Task 3 completed (API endpoints for health check)
- All source files present
- Package.json with correct dependencies
- No local .env file required (uses defaults)

## Container Runtime Behavior

### Startup Sequence
1. Container starts with nodejs user
2. Environment variables set (NODE_ENV=production)
3. Application starts on port 3000
4. Health check begins after 5s start period
5. Container marked healthy after successful check

### Shutdown Sequence
1. Container receives SIGTERM
2. Application handles graceful shutdown
3. Existing connections closed
4. Process exits cleanly
5. Container stops

### Resource Limits
- Memory: Typically < 128MB
- CPU: Minimal usage (< 0.1 cores)
- Storage: Image size < 200MB

## Testing Strategy

### Build Verification
```bash
# Build the image
docker build -t hello-world-api:latest .

# Check image size
docker images hello-world-api:latest

# Inspect image layers
docker history hello-world-api:latest
```

### Runtime Testing
```bash
# Run container
docker run -d --name test-api -p 3000:3000 hello-world-api:latest

# Check logs
docker logs test-api

# Test endpoints
curl http://localhost:3000/health
curl http://localhost:3000/hello

# Check health status
docker inspect test-api --format='{{.State.Health.Status}}'

# Test graceful shutdown
docker stop test-api
```

### Security Verification
```bash
# Check running user
docker exec test-api whoami
# Expected: nodejs

# Check file permissions
docker exec test-api ls -la /app
# Expected: files owned by nodejs:nodejs

# Scan for vulnerabilities
docker scan hello-world-api:latest
```

## Success Criteria
- Docker image builds successfully
- Image size < 200MB
- Container starts without errors
- Health endpoint responds correctly
- Runs as non-root user (nodejs)
- Graceful shutdown works properly
- Health check passes after startup
- No security vulnerabilities in base image

## Common Docker Patterns

### Environment Variable Override
```bash
# Override default port
docker run -e PORT=8080 -p 8080:8080 hello-world-api:latest
```

### Volume Mounting for Development
```bash
# Mount source code for development
docker run -v $(pwd)/src:/app/src hello-world-api:latest
```

### Docker Compose Example
```yaml
version: '3.8'
services:
  api:
    image: hello-world-api:latest
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
      interval: 30s
      timeout: 5s
      retries: 3
```

## Troubleshooting Guide

### Issue: Container exits immediately
**Solution:** Check logs with `docker logs <container>` for startup errors

### Issue: Health check failing
**Solution:** Verify /health endpoint works and wget is available in image

### Issue: Permission denied errors
**Solution:** Ensure COPY uses --chown=nodejs:nodejs flag

### Issue: Image size too large
**Solution:** Check .dockerignore is working, verify multi-stage build

## Related Tasks
- Task 3: API Endpoints (provides health check endpoint)
- Task 5: Testing Suite (tests should work in container)
- Task 7: ESLint Configuration (ensures code quality before containerization)
- Task 8: Kubernetes Deployment (uses this Docker image)