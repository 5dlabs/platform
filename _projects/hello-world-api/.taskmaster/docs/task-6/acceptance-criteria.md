# Task 6: Create Dockerfile with Multi-Stage Build - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. Docker Ignore Configuration ✓
- [ ] File `.dockerignore` exists in root directory
- [ ] Contains `node_modules` entry
- [ ] Contains `npm-debug.log` entry
- [ ] Contains `.git` entry
- [ ] Contains `.gitignore` entry
- [ ] Contains `.env` entry
- [ ] Contains `.vscode` entry
- [ ] Contains `coverage` entry
- [ ] Contains `.DS_Store` entry

### 2. Dockerfile Structure ✓
- [ ] File `Dockerfile` exists in root directory
- [ ] Uses multi-stage build (at least 2 stages)
- [ ] Build stage uses `node:18-alpine`
- [ ] Runtime stage uses `node:18-alpine`
- [ ] Build stage named `AS build`
- [ ] Uses `npm ci --only=production`

### 3. Security Configuration ✓
- [ ] Creates non-root user named `nodejs`
- [ ] User has UID 1001
- [ ] Creates group with GID 1001
- [ ] All COPY commands use `--chown=nodejs:nodejs`
- [ ] Switches to nodejs user with `USER nodejs`
- [ ] No sudo or root operations after USER directive

### 4. Runtime Configuration ✓
- [ ] Sets `NODE_ENV=production`
- [ ] Sets `PORT=3000`
- [ ] Sets `WORKDIR /app`
- [ ] Exposes port 3000
- [ ] CMD runs `node src/server.js`

### 5. Health Check ✓
- [ ] HEALTHCHECK directive present
- [ ] Interval set to 30s
- [ ] Timeout set to 5s
- [ ] Start period set to 5s
- [ ] Retries set to 3
- [ ] Uses wget to check health endpoint

### 6. Build Script ✓
- [ ] Directory `scripts/` exists
- [ ] File `scripts/docker-build.sh` exists
- [ ] Script is executable (chmod +x)
- [ ] Script builds the image
- [ ] Script displays image size
- [ ] Script runs test container
- [ ] Script tests health endpoint
- [ ] Script cleans up test container

## Test Cases

### Test Case 1: Docker Build
```bash
docker build -t hello-world-api:test .
```
**Expected:**
- Build completes successfully
- No errors or warnings
- Both stages execute

### Test Case 2: Image Size Verification
```bash
docker images hello-world-api:test --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
```
**Expected:**
- Image size < 200MB
- Typically around 150MB for Node.js Alpine image

### Test Case 3: Container Runtime User
```bash
docker run --rm hello-world-api:test whoami
```
**Expected Output:** `nodejs`

### Test Case 4: Container Startup
```bash
docker run -d --name test-container -p 3000:3000 hello-world-api:test
docker logs test-container
```
**Expected:**
- Container starts without errors
- Logs show "Server running on port 3000"

### Test Case 5: Health Check Functionality
```bash
# Start container
docker run -d --name health-test -p 3000:3000 hello-world-api:test

# Wait for startup
sleep 10

# Check health status
docker inspect health-test --format='{{.State.Health.Status}}'
```
**Expected Output:** `healthy`

### Test Case 6: Health Endpoint Access
```bash
# With container running
curl http://localhost:3000/health
```
**Expected:**
- 200 status code
- JSON response with status "success"

### Test Case 7: File Ownership
```bash
docker run --rm hello-world-api:test ls -la /app
```
**Expected:**
- All files owned by `nodejs:nodejs`
- No files owned by root

### Test Case 8: Build Script Execution
```bash
./scripts/docker-build.sh
```
**Expected Output:**
```
Building Docker image: hello-world-api:latest
Image size: <200MB
Running container for testing...
Waiting for container to start...
Testing health endpoint...
{"status":"success","message":"Service is healthy","data":{"status":"up"},"timestamp":"..."}
Stopping and removing test container...
Docker build and test complete!
```

## Validation Commands

### Dockerfile Analysis
```bash
# Check for multi-stage build
grep -E "FROM.*AS build" Dockerfile
grep -c "FROM" Dockerfile  # Should be 2 or more

# Check security configuration
grep "USER nodejs" Dockerfile
grep "adduser.*nodejs.*1001" Dockerfile

# Check health check
grep "HEALTHCHECK" Dockerfile
```

### Container Security Validation
```bash
# Run security check
docker run --rm hello-world-api:test id
# Expected: uid=1001(nodejs) gid=1001(nodejs)

# Check for root files
docker run --rm hello-world-api:test find /app -user root | wc -l
# Expected: 0
```

### Build Context Validation
```bash
# Build with verbose to see context size
docker build --no-cache --progress=plain -t test:latest . 2>&1 | grep "transferring context"
# Context should be small due to .dockerignore
```

## Success Indicators
- ✅ Image builds without errors
- ✅ Final image size < 200MB
- ✅ Container runs as non-root user
- ✅ Health check passes after startup
- ✅ All endpoints accessible
- ✅ Graceful shutdown works
- ✅ Build script completes successfully
- ✅ No security warnings from Docker

## Common Issues and Solutions

### Issue 1: Health check failing
**Debug Steps:**
```bash
# Check if wget is available
docker run --rm hello-world-api:test which wget

# Test health check manually
docker exec test-container wget --spider http://localhost:3000/health
```

### Issue 2: Permission denied errors
**Solution:** Ensure all COPY commands include `--chown=nodejs:nodejs`:
```dockerfile
COPY --chown=nodejs:nodejs . .
```

### Issue 3: Large image size
**Debug Steps:**
```bash
# Analyze image layers
docker history hello-world-api:test

# Check what's in node_modules
docker run --rm hello-world-api:test du -sh node_modules
```

### Issue 4: Build script not executable
**Solution:**
```bash
chmod +x scripts/docker-build.sh
ls -la scripts/docker-build.sh  # Should show 'x' permissions
```

## Performance Benchmarks
- Build time: < 60 seconds (with cache)
- Image size: 120-180MB
- Container startup: < 5 seconds
- Memory usage: < 128MB
- Health check response: < 100ms

## Production Readiness Checklist
- [ ] No development dependencies in image
- [ ] No source maps or test files
- [ ] Environment variables configurable
- [ ] Logs to stdout/stderr
- [ ] Handles SIGTERM gracefully
- [ ] Health check responds quickly
- [ ] Runs as non-root user
- [ ] No hardcoded secrets

## Additional Validation
```bash
# Scan for vulnerabilities (requires Docker Desktop or Snyk)
docker scan hello-world-api:latest

# Check for exposed secrets
docker run --rm hello-world-api:test env | grep -E "SECRET|PASSWORD|KEY"
# Should return nothing

# Verify graceful shutdown
docker run -d --name shutdown-test hello-world-api:test
docker stop -t 10 shutdown-test
docker logs shutdown-test | grep "Shutting down gracefully"
```