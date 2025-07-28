# Task 8: Create CI/CD Pipeline Configuration

## Overview
This task implements a complete CI/CD pipeline using GitHub Actions to automate testing, building, and deployment of the Hello World API. The pipeline ensures code quality, automates Docker image creation, and deploys to Kubernetes, providing a production-ready continuous delivery workflow.

## Objectives
- Automate testing and code quality checks
- Build and publish Docker images on successful tests
- Deploy to Kubernetes automatically on main branch
- Ensure image size constraints are met
- Upload test coverage reports
- Create comprehensive project documentation

## Technical Approach

### Pipeline Architecture
The CI/CD pipeline consists of three sequential jobs:
1. **Test Job**: Runs on all PRs and pushes
2. **Build Job**: Runs only on main branch after tests pass
3. **Deploy Job**: Runs after successful build to update Kubernetes

### Pipeline Flow
```
[Code Push/PR] → [Test & Lint] → [Build Docker] → [Deploy to K8s]
                       ↓                ↓                ↓
                 [Coverage Report] [Image Size Check] [Rollout Status]
```

## Implementation Details

### Step 1: Create GitHub Actions Workflow (.github/workflows/ci.yml)
```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Use Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    - name: Install dependencies
      run: npm ci
    - name: Run linting
      run: npm run lint
    - name: Run tests
      run: npm test
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      with:
        token: ${{ secrets.CODECOV_TOKEN }}

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2
    - name: Login to DockerHub
      uses: docker/login-action@v2
      with:
        username: ${{ secrets.DOCKERHUB_USERNAME }}
        password: ${{ secrets.DOCKERHUB_TOKEN }}
    - name: Build and push
      uses: docker/build-push-action@v4
      with:
        context: .
        push: true
        tags: ${{ secrets.DOCKERHUB_USERNAME }}/hello-world-api:latest
    - name: Image size check
      run: |
        IMAGE_SIZE=$(docker image inspect ${{ secrets.DOCKERHUB_USERNAME }}/hello-world-api:latest --format='{{.Size}}')
        IMAGE_SIZE_MB=$(echo "scale=2; $IMAGE_SIZE/1024/1024" | bc)
        echo "Image size: ${IMAGE_SIZE_MB}MB"
        if (( $(echo "$IMAGE_SIZE_MB > 200" | bc -l) )); then
          echo "Image size exceeds 200MB limit"
          exit 1
        fi

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    - name: Set up kubectl
      uses: azure/setup-kubectl@v3
    - name: Set Kubernetes context
      uses: azure/k8s-set-context@v3
      with:
        kubeconfig: ${{ secrets.KUBE_CONFIG }}
    - name: Deploy to Kubernetes
      run: |
        # Update image tag in deployment.yaml
        sed -i 's|image: hello-world-api:latest|image: ${{ secrets.DOCKERHUB_USERNAME }}/hello-world-api:latest|' kubernetes/deployment.yaml
        # Apply Kubernetes manifests
        kubectl apply -f kubernetes/configmap.yaml
        kubectl apply -f kubernetes/deployment.yaml
        kubectl apply -f kubernetes/service.yaml
        kubectl apply -f kubernetes/ingress.yaml
        # Wait for deployment to complete
        kubectl rollout status deployment/hello-world-api
```

### Step 2: Create Comprehensive README.md
```markdown
# Hello World API

A simple REST API that serves as a "Hello World" example for testing the 5D Labs orchestrator workflow. This API demonstrates basic HTTP endpoints, JSON responses, and containerized deployment.

## Features

- Health check endpoint
- Basic and personalized greeting endpoints
- Echo service for posted JSON data
- Service information endpoint
- OpenAPI/Swagger documentation
- Containerized with Docker
- Kubernetes deployment ready
- Comprehensive test suite

## Technical Stack

- **Language**: Node.js with Express.js framework
- **Testing**: Jest for unit tests, Supertest for API testing
- **Documentation**: OpenAPI/Swagger specification
- **Containerization**: Docker with multi-stage builds
- **Deployment**: Kubernetes deployment ready

## API Endpoints

- **GET /health** - Health check endpoint
- **GET /hello** - Basic hello world endpoint
- **GET /hello/{name}** - Personalized greeting
- **POST /echo** - Echo service for JSON data
- **GET /info** - Service information
- **GET /docs** - API documentation

## Getting Started

### Prerequisites

- Node.js 18 or higher
- npm or yarn
- Docker (for containerization)
- Kubernetes (for deployment)

### Installation

1. Clone the repository
   ```
   git clone https://github.com/yourusername/hello-world-api.git
   cd hello-world-api
   ```

2. Install dependencies
   ```
   npm install
   ```

3. Start the development server
   ```
   npm run dev
   ```

4. Access the API at http://localhost:3000

### Running Tests

```
npm test
```

### Building and Running with Docker

1. Build the Docker image
   ```
   docker build -t hello-world-api .
   ```

2. Run the container
   ```
   docker run -p 3000:3000 hello-world-api
   ```

### Deploying to Kubernetes

1. Apply the Kubernetes manifests
   ```
   kubectl apply -f kubernetes/
   ```

2. Check the deployment status
   ```
   kubectl get pods -l app=hello-world-api
   ```

## CI/CD Pipeline

This project includes a GitHub Actions workflow for continuous integration and deployment:

1. Run tests and linting on pull requests
2. Build and push Docker image on merge to main
3. Deploy to Kubernetes on successful build

## License

MIT
```

### Key Pipeline Features

#### Test Job
- Runs on every push and pull request
- Uses Node.js 18 with npm caching
- Executes linting and full test suite
- Uploads coverage to Codecov
- Blocks further jobs if tests fail

#### Build Job
- Only runs on main branch pushes
- Uses Docker Buildx for efficient builds
- Authenticates with DockerHub
- Pushes image with latest tag
- Validates image size < 200MB

#### Deploy Job
- Triggered after successful build
- Configures kubectl with cluster credentials
- Updates deployment with new image
- Applies all Kubernetes manifests
- Waits for successful rollout

## Dependencies and Requirements
- Tasks 5, 6, and 7 must be completed
- GitHub repository with Actions enabled
- DockerHub account for image storage
- Kubernetes cluster with kubectl access
- Required GitHub Secrets:
  - DOCKERHUB_USERNAME
  - DOCKERHUB_TOKEN
  - CODECOV_TOKEN (optional)
  - KUBE_CONFIG

## GitHub Secrets Configuration

### Setting Up Secrets
1. **DockerHub Credentials**
   ```
   DOCKERHUB_USERNAME: your-dockerhub-username
   DOCKERHUB_TOKEN: your-dockerhub-access-token
   ```

2. **Kubernetes Config**
   ```
   KUBE_CONFIG: base64 encoded kubeconfig file
   ```
   Generate with: `cat ~/.kube/config | base64`

3. **Codecov Token** (optional)
   ```
   CODECOV_TOKEN: your-codecov-token
   ```

## Testing Strategy

### Local Workflow Testing
```bash
# Install act for local GitHub Actions testing
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux

# Test the workflow locally
act -j test
```

### Pipeline Verification
1. **Test Job Verification**
   - Create PR with test changes
   - Verify tests run and pass
   - Check coverage upload

2. **Build Job Verification**
   - Merge PR to main
   - Verify Docker image builds
   - Check image size validation

3. **Deploy Job Verification**
   - Monitor deployment rollout
   - Verify new pods are running
   - Test deployed application

## Success Criteria
- All jobs complete successfully on main branch
- Tests achieve >90% coverage
- Docker image size < 200MB
- Image successfully pushed to registry
- Kubernetes deployment updates without downtime
- README provides clear documentation
- Pipeline runs in < 10 minutes

## Monitoring and Debugging

### GitHub Actions Dashboard
- View workflow runs at: `https://github.com/[owner]/[repo]/actions`
- Check job logs for failures
- Review artifact uploads
- Monitor execution times

### Common Pipeline Issues

#### Test Failures
```yaml
# Add debugging to test job
- name: Run tests with verbose output
  run: npm test -- --verbose
```

#### Docker Build Issues
```yaml
# Add build caching
- name: Build and push
  uses: docker/build-push-action@v4
  with:
    cache-from: type=registry,ref=${{ secrets.DOCKERHUB_USERNAME }}/hello-world-api:buildcache
    cache-to: type=registry,ref=${{ secrets.DOCKERHUB_USERNAME }}/hello-world-api:buildcache,mode=max
```

#### Deployment Failures
```bash
# Debug deployment
kubectl describe deployment hello-world-api
kubectl logs -l app=hello-world-api
```

## Best Practices

### Security
- Use GitHub Secrets for sensitive data
- Implement least-privilege access
- Scan images for vulnerabilities
- Use specific action versions

### Performance
- Cache dependencies
- Use Docker layer caching
- Parallelize independent jobs
- Minimize build context

### Reliability
- Add retry logic for flaky steps
- Implement rollback mechanisms
- Monitor pipeline metrics
- Set up alerts for failures

## Related Tasks
- Task 5: Testing Suite (tests run in pipeline)
- Task 6: Docker Configuration (image built in pipeline)
- Task 7: Kubernetes Manifests (deployed by pipeline)

## Future Enhancements
- Add staging environment deployment
- Implement semantic versioning
- Add security scanning
- Enable branch protection rules
- Set up deployment approvals
- Add performance testing stage