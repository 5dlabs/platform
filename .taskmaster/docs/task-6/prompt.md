# Task 6: Create Dockerfile with Multi-Stage Build - Autonomous Agent Prompt

You are an experienced DevOps engineer tasked with containerizing the Hello World API using Docker best practices. You need to create a production-ready Docker image that is minimal, secure, and includes health checks.

## Your Mission
Create a multi-stage Dockerfile, configure Docker ignore patterns, implement security best practices with a non-root user, and create an automated build/test script.

## Detailed Instructions

### 1. Create .dockerignore File
Create a `.dockerignore` file in the root directory with the following content:

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

**Purpose:**
- Prevents unnecessary files from being included in the Docker build context
- Reduces build time and image size
- Avoids including sensitive files like .env

### 2. Create Multi-Stage Dockerfile
Create a `Dockerfile` in the root directory with this exact content:

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

**Key Features Explained:**

**Multi-Stage Build:**
- First stage (build): Installs dependencies in a clean environment
- Second stage (runtime): Copies only production dependencies
- Results in smaller final image

**Security Features:**
- Creates non-root user (nodejs) with UID 1001
- All files owned by nodejs user
- Application runs as nodejs user, not root

**Health Check:**
- Runs every 30 seconds
- 5 second timeout per check
- 5 second start period before first check
- Fails after 3 consecutive failures
- Uses wget (available in Alpine)

### 3. Create Scripts Directory and Build Script
First create the scripts directory:
```bash
mkdir -p scripts
```

Then create `scripts/docker-build.sh` with this content:

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

### 4. Make Script Executable
Run this command:
```bash
chmod +x scripts/docker-build.sh
```

## Testing Your Implementation

### Manual Build and Test
```bash
# Build the image
docker build -t hello-world-api:latest .

# Check image size (should be < 200MB)
docker images hello-world-api:latest

# Run container
docker run -d --name test-api -p 3000:3000 hello-world-api:latest

# Check if running
docker ps

# Test health endpoint
curl http://localhost:3000/health

# Check logs
docker logs test-api

# Verify running as non-root user
docker exec test-api whoami
# Should output: nodejs

# Check health status
docker inspect test-api --format='{{.State.Health.Status}}'
# Should show: healthy (after ~10 seconds)

# Clean up
docker stop test-api
docker rm test-api
```

### Automated Build and Test
```bash
# Run the build script
./scripts/docker-build.sh
```

## Expected Results

### Image Size
```bash
docker images hello-world-api:latest
# SIZE should be < 200MB (typically ~150MB)
```

### Container User
```bash
docker run --rm hello-world-api:latest whoami
# Output: nodejs
```

### Health Check Status
```bash
docker inspect <container-id> --format='{{json .State.Health}}'
# Should show Status: "healthy" after startup
```

## Common Issues and Solutions

### Issue 1: "wget: command not found"
**Solution:** The Alpine image should include wget. If not, add `RUN apk add --no-cache wget` before the HEALTHCHECK

### Issue 2: Permission denied when running
**Solution:** Ensure all COPY commands use `--chown=nodejs:nodejs`

### Issue 3: Build script permission denied
**Solution:** Run `chmod +x scripts/docker-build.sh`

### Issue 4: Health check always failing
**Solution:** Verify the /health endpoint works and the container can reach localhost:3000

### Issue 5: Image size too large
**Solution:** Check that .dockerignore is working and you're using `npm ci --only=production`

## Validation Checklist
- [ ] .dockerignore file exists and excludes node_modules
- [ ] Dockerfile uses multi-stage build
- [ ] Final stage uses Alpine Linux
- [ ] Non-root user (nodejs) is created
- [ ] Application runs as nodejs user
- [ ] HEALTHCHECK directive is present
- [ ] Image size is < 200MB
- [ ] Build script executes successfully
- [ ] Container starts and passes health check
- [ ] Graceful shutdown works (test with docker stop)

## Best Practices Implemented
1. **Multi-stage builds** for smaller images
2. **Alpine Linux** for minimal base image
3. **Non-root user** for security
4. **Health checks** for orchestration
5. **.dockerignore** to exclude unnecessary files
6. **Build caching** with separate COPY for package files
7. **Production dependencies only** with npm ci --only=production
8. **Explicit port exposure** with EXPOSE
9. **Clear startup command** with CMD
10. **File ownership** with --chown flag

Complete this task by creating all files exactly as specified. The resulting Docker image should be production-ready, secure, and efficient.