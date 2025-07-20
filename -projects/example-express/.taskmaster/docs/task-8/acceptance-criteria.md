# Task 8: Prepare Deployment Configuration - Acceptance Criteria

## Overview
This document defines acceptance criteria for preparing the Express.js application for production deployment. The deployment configuration should include containerization, environment management, CI/CD pipeline, and comprehensive documentation.

## Docker Configuration Criteria

### ✓ Dockerfile Created
- **Requirement**: Multi-stage Dockerfile exists
- **Verification**:
  ```bash
  test -f Dockerfile && echo "Dockerfile exists"
  ```
- **Expected Features**:
  - Multi-stage build
  - Node 20 Alpine base
  - Non-root user
  - Dumb-init for signals
  - Optimized layers

### ✓ Docker Build Success
- **Requirement**: Image builds without errors
- **Verification**:
  ```bash
  docker build -t express-app .
  ```
- **Expected**: Build completes successfully

### ✓ Container Runs
- **Requirement**: Container starts and runs
- **Verification**:
  ```bash
  docker run -p 3000:3000 express-app
  ```
- **Expected**: Application accessible on port 3000

### ✓ .dockerignore Configuration
- **Requirement**: Excludes unnecessary files
- **Verification**:
  ```bash
  test -f .dockerignore && cat .dockerignore
  ```
- **Expected Exclusions**:
  - node_modules
  - .env files
  - test directories
  - coverage reports
  - .git directory

### ✓ Image Size Optimization
- **Requirement**: Production image is optimized
- **Verification**:
  ```bash
  docker images express-app --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
  ```
- **Expected**: Image size < 200MB

## Docker Compose Configuration Criteria

### ✓ Development Compose File
- **Requirement**: docker-compose.yml for development
- **Verification**:
  ```bash
  docker-compose config
  ```
- **Expected Features**:
  - Volume mounts for hot reload
  - Development command
  - Port mapping

### ✓ Production Compose File
- **Requirement**: docker-compose.prod.yml for production
- **Verification**:
  ```bash
  docker-compose -f docker-compose.prod.yml config
  ```
- **Expected Features**:
  - Restart policy
  - Health check
  - Environment file support
  - Data volume

### ✓ Compose Services Start
- **Requirement**: Services start successfully
- **Test Both**:
  ```bash
  docker-compose up -d
  docker-compose -f docker-compose.prod.yml up -d
  ```
- **Expected**: Containers running without errors

## Environment Configuration Criteria

### ✓ Environment Template
- **Requirement**: .env.example file exists
- **Verification**:
  ```bash
  test -f .env.example && echo "Template exists"
  ```
- **Expected Variables**:
  - NODE_ENV
  - PORT
  - DATABASE_URL
  - JWT_SECRET
  - JWT_REFRESH_SECRET
  - All other required vars

### ✓ Configuration Module
- **Requirement**: Centralized config management
- **Verification**:
  ```bash
  test -f src/config/index.js && echo "Config module exists"
  ```
- **Expected Features**:
  - Environment parsing
  - Default values
  - Validation
  - Type conversion

### ✓ Required Variable Validation
- **Requirement**: Production validates required vars
- **Test**:
  ```bash
  NODE_ENV=production node -e "require('./src/config')"
  ```
- **Expected**: Error if required vars missing

### ✓ Environment-Specific Config
- **Requirement**: Different configs per environment
- **Test**: Check config values for each environment
- **Expected**:
  - Development: Relaxed security
  - Production: Strict security
  - Test: In-memory database

## Health Check Criteria

### ✓ Health Endpoint
- **Requirement**: Basic health check endpoint
- **Test**:
  ```bash
  curl http://localhost:3000/health
  ```
- **Expected Response**:
  ```json
  {
    "status": "healthy",
    "timestamp": "...",
    "uptime": 123,
    "environment": "production",
    "version": "1.0.0",
    "database": "connected"
  }
  ```

### ✓ Readiness Endpoint
- **Requirement**: Readiness check endpoint
- **Test**:
  ```bash
  curl http://localhost:3000/health/ready
  ```
- **Expected Response**:
  ```json
  {
    "ready": true,
    "timestamp": "..."
  }
  ```

### ✓ Database Check
- **Requirement**: Health includes database status
- **Test**: Stop database and check health
- **Expected**: Status 503 with database error

### ✓ Docker Health Check
- **Requirement**: Container health monitoring
- **Verification**:
  ```bash
  docker inspect express-app | grep -A 10 Healthcheck
  ```
- **Expected**: Health check configured

## Production Optimization Criteria

### ✓ Compression Enabled
- **Requirement**: Response compression in production
- **Test**: Check response headers
  ```bash
  curl -H "Accept-Encoding: gzip" -I http://localhost:3000/api/tasks
  ```
- **Expected**: Content-Encoding: gzip

### ✓ Security Headers
- **Requirement**: Helmet security headers
- **Test**: Check response headers
- **Expected Headers**:
  - X-Content-Type-Options
  - X-Frame-Options
  - X-XSS-Protection
  - Strict-Transport-Security (if HTTPS)

### ✓ Static File Caching
- **Requirement**: Cache headers for static files
- **Test**:
  ```bash
  curl -I http://localhost:3000/styles.css
  ```
- **Expected**: Cache-Control header with max-age

### ✓ Trust Proxy Setting
- **Requirement**: Configured for reverse proxy
- **Verification**: Check app.get('trust proxy')
- **Expected**: Set to 1 in production

### ✓ Graceful Shutdown
- **Requirement**: Handles SIGTERM/SIGINT
- **Test**:
  ```bash
  # Start app
  node src/index.js &
  PID=$!
  # Send signal
  kill -TERM $PID
  ```
- **Expected**: "Shutting down gracefully" message

## CI/CD Pipeline Criteria

### ✓ GitHub Actions Workflow
- **Requirement**: Deploy workflow exists
- **Verification**:
  ```bash
  test -f .github/workflows/deploy.yml && echo "Workflow exists"
  ```

### ✓ Workflow Jobs
- **Requirement**: Complete CI/CD pipeline
- **Expected Jobs**:
  - Test job (runs tests)
  - Build job (builds Docker image)
  - Deploy job (deployment logic)

### ✓ Test Job Success
- **Requirement**: Tests run in CI
- **Expected Steps**:
  - Checkout code
  - Setup Node.js
  - Install dependencies
  - Run tests
  - Run linter

### ✓ Docker Build in CI
- **Requirement**: Builds and pushes image
- **Expected Steps**:
  - Login to registry
  - Extract metadata
  - Build image
  - Push to registry

### ✓ Conditional Deployment
- **Requirement**: Deploy only from main/production
- **Verification**: Check workflow conditions
- **Expected**: Deploy job has branch conditions

## NPM Scripts Criteria

### ✓ Deployment Scripts
- **Requirement**: Scripts for deployment tasks
- **Verification**:
  ```bash
  npm run | grep docker
  ```
- **Expected Scripts**:
  - docker:build
  - docker:run
  - docker:dev
  - docker:prod
  - docker:down

### ✓ Scripts Function
- **Test Each**:
  ```bash
  npm run docker:build
  npm run lint
  npm test
  ```
- **Expected**: All scripts execute successfully

## Documentation Criteria

### ✓ Deployment Guide
- **Requirement**: DEPLOYMENT.md exists
- **Verification**:
  ```bash
  test -f DEPLOYMENT.md && echo "Guide exists"
  ```
- **Expected Sections**:
  - Prerequisites
  - Local deployment
  - Cloud deployment options
  - Environment variables
  - Security checklist

### ✓ Platform Examples
- **Requirement**: Multiple deployment options
- **Expected Examples**:
  - Heroku
  - AWS ECS
  - DigitalOcean
  - Generic VPS

### ✓ Environment Documentation
- **Requirement**: All variables documented
- **Verification**: Check .env.example comments
- **Expected**: Each variable has description

## Security Criteria

### ✓ Non-Root Container User
- **Requirement**: Container runs as non-root
- **Verification**:
  ```bash
  docker run express-app whoami
  ```
- **Expected**: nodejs (not root)

### ✓ No Secrets in Image
- **Requirement**: No hardcoded secrets
- **Test**: Inspect image layers
  ```bash
  docker history express-app
  ```
- **Expected**: No JWT_SECRET or passwords

### ✓ Production Error Handling
- **Requirement**: No stack traces in production
- **Test**: Trigger 500 error in production mode
- **Expected**: Generic error message

### ✓ Environment Validation
- **Requirement**: Required vars validated
- **Test**: Start without required vars
- **Expected**: Clear error message

## Performance Criteria

### ✓ Multi-Stage Build
- **Requirement**: Optimized build process
- **Verification**: Check Dockerfile stages
- **Expected**: Separate build and runtime stages

### ✓ Production Dependencies Only
- **Requirement**: No dev dependencies in image
- **Verification**:
  ```bash
  docker run express-app npm list --production
  ```
- **Expected**: Only production packages

### ✓ Layer Caching
- **Requirement**: Efficient layer caching
- **Test**: Rebuild after code change
- **Expected**: Package layers cached

## Monitoring Criteria

### ✓ Structured Logging
- **Requirement**: JSON logs in production
- **Test**: Check production logs
- **Expected**: JSON formatted output

### ✓ Health Metrics
- **Requirement**: Exposes useful metrics
- **Verification**: Check /health response
- **Expected**: Uptime, version, status

## Test Summary Checklist

- [ ] Dockerfile created with multi-stage build
- [ ] Docker image builds successfully
- [ ] Container runs without errors
- [ ] .dockerignore excludes unnecessary files
- [ ] docker-compose.yml for development
- [ ] docker-compose.prod.yml for production
- [ ] .env.example with all variables
- [ ] src/config/index.js for configuration
- [ ] Environment validation works
- [ ] Health endpoints functioning
- [ ] Readiness check implemented
- [ ] Compression enabled in production
- [ ] Security headers configured
- [ ] Graceful shutdown works
- [ ] GitHub Actions workflow created
- [ ] CI/CD pipeline runs tests
- [ ] Docker image pushed to registry
- [ ] NPM scripts for deployment
- [ ] DEPLOYMENT.md documentation
- [ ] Non-root container user
- [ ] No secrets in image
- [ ] Production error handling
- [ ] Multi-stage build optimization

## Definition of Done

Task 8 is complete when:
1. Application is fully containerized with Docker
2. Environment configuration is centralized and validated
3. Health check endpoints are implemented
4. Production optimizations are applied
5. CI/CD pipeline is configured and working
6. Comprehensive deployment documentation exists
7. Security best practices are followed
8. All acceptance criteria are met

## Notes

- Consider using secrets management service in production
- Monitor container resource usage
- Set up log aggregation for production
- Implement backup strategy for data
- Consider using Kubernetes for orchestration
- Add monitoring and alerting tools