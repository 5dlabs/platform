# Task 8: Create CI/CD Pipeline Configuration - Autonomous Agent Prompt

You are an experienced DevOps engineer tasked with creating a complete CI/CD pipeline using GitHub Actions. You need to automate testing, building, and deployment of the Hello World API to ensure reliable and consistent delivery.

## Your Mission
Create a GitHub Actions workflow that automatically tests code, builds Docker images, and deploys to Kubernetes. Also create comprehensive project documentation in the README file.

## Detailed Instructions

### 1. Create GitHub Actions Directory
First, create the necessary directory structure:
```bash
mkdir -p .github/workflows
```

### 2. Create CI/CD Workflow (.github/workflows/ci.yml)
Create a comprehensive workflow file with three jobs:

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

**Key Workflow Features:**

**Triggers:**
- On push to main branch
- On pull requests to main branch

**Test Job:**
- Runs for all triggers
- Sets up Node.js 18 with npm caching
- Installs dependencies with `npm ci`
- Runs linting (requires npm script)
- Runs full test suite
- Uploads coverage to Codecov

**Build Job:**
- Only runs on main branch pushes
- Depends on successful test job
- Uses Docker Buildx for better performance
- Logs into DockerHub
- Builds and pushes image
- Validates image size < 200MB

**Deploy Job:**
- Only runs after successful build
- Sets up kubectl
- Configures Kubernetes context
- Updates deployment image reference
- Applies all manifests in order
- Waits for rollout completion

### 3. Create Comprehensive README.md
Create or update the README.md file in the root directory:

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

### 4. Update package.json Scripts
Ensure your package.json includes the lint script referenced in the workflow:

```json
{
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  }
}
```

## Required GitHub Secrets

Before the pipeline can run successfully, configure these secrets in your GitHub repository:

1. **DOCKERHUB_USERNAME**: Your DockerHub username
2. **DOCKERHUB_TOKEN**: DockerHub access token (not password)
3. **KUBE_CONFIG**: Base64 encoded kubeconfig file
4. **CODECOV_TOKEN**: (Optional) Token for coverage reporting

### Setting Secrets
1. Go to Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add each secret with its value

### Generating DockerHub Token
1. Log into DockerHub
2. Go to Account Settings → Security
3. Create New Access Token
4. Copy token and save as DOCKERHUB_TOKEN secret

### Encoding Kubeconfig
```bash
cat ~/.kube/config | base64 -w 0
```
Save the output as KUBE_CONFIG secret

## Testing Your Implementation

### Local Validation
```bash
# Validate YAML syntax
yamllint .github/workflows/ci.yml

# Check workflow syntax with act (if installed)
act -l
```

### Testing the Pipeline
1. **Create a feature branch**
   ```bash
   git checkout -b test-pipeline
   ```

2. **Make a small change and push**
   ```bash
   echo "test" >> test.txt
   git add test.txt
   git commit -m "Test pipeline"
   git push origin test-pipeline
   ```

3. **Create a pull request**
   - Should trigger the test job only
   - Check that tests run successfully

4. **Merge to main**
   - Should trigger all three jobs
   - Monitor each job's progress

## Expected Pipeline Behavior

### On Pull Request
- Only the test job runs
- Tests and linting must pass
- Coverage report uploaded
- No build or deploy jobs

### On Main Branch Push
- All three jobs run sequentially
- Test → Build → Deploy
- Each job must succeed for next to run
- Final result: Updated Kubernetes deployment

## Common Issues and Solutions

### Issue 1: Lint script not found
**Solution:** Add lint script to package.json:
```json
"lint": "eslint . || true"
```

### Issue 2: Docker push fails
**Solution:** Verify DockerHub credentials and token permissions

### Issue 3: Kubectl context fails
**Solution:** Ensure KUBE_CONFIG secret is properly base64 encoded

### Issue 4: Image size check fails
**Solution:** Optimize Dockerfile to reduce image size

## Best Practices Implemented
1. **Job dependencies** ensure proper execution order
2. **Conditional execution** prevents unnecessary builds
3. **Secret management** for sensitive data
4. **Image size validation** ensures efficiency
5. **Rollout status check** confirms deployment success
6. **npm caching** speeds up installations
7. **Coverage reporting** tracks test quality
8. **Clear documentation** helps onboarding

Complete this task by creating both the GitHub Actions workflow and the comprehensive README file. The pipeline should be ready to run once the required secrets are configured.