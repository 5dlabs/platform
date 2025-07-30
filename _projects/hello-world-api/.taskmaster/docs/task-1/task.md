# Task 1: Setup Project Structure and Dependencies

## Overview
Initialize the Node.js project with Express.js framework and establish the foundational project structure with all necessary dependencies for building a robust REST API service.

## Technical Context
This task creates the foundation for the Hello World API, a lightweight Node.js REST service designed for testing the 5D Labs orchestrator workflow. The setup follows modern Node.js best practices with a three-layer architecture (API, Business Logic, Infrastructure).

## Implementation Guide

### 1. Project Initialization
```bash
# Create and navigate to project directory
mkdir hello-world-api
cd hello-world-api

# Initialize Node.js project with default configuration
npm init -y
```

### 2. Core Dependencies Installation
Install production dependencies for the Express.js application:

```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

**Dependency Rationale:**
- `express@4.18.2`: Web framework for REST API
- `cors@2.8.5`: Cross-origin resource sharing support
- `helmet@7.0.0`: Security headers middleware
- `pino@8.15.0`: High-performance structured logging
- `pino-http@8.5.0`: HTTP request logging middleware
- `dotenv@16.3.1`: Environment variable management

### 3. Development Dependencies Installation
Install development tools for testing, linting, and documentation:

```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

**Development Dependencies:**
- `jest@29.6.4`: Testing framework
- `supertest@6.3.3`: HTTP API testing
- `nodemon@3.0.1`: Development server with auto-restart
- `eslint@8.48.0`: Code linting and style enforcement
- `swagger-jsdoc@6.2.8`: OpenAPI specification generation
- `swagger-ui-express@5.0.0`: API documentation UI

### 4. Project Structure Creation
Create the recommended directory structure for scalability:

```bash
# Source code directories
mkdir -p src/middleware src/routes src/utils

# Test directories
mkdir -p tests/unit tests/integration

# Documentation directory
mkdir docs

# Create core application files
touch src/app.js src/server.js
touch docs/openapi.yaml
touch .env .dockerignore Dockerfile kubernetes.yaml README.md
```

**Directory Structure:**
```
hello-world-api/
├── src/
│   ├── middleware/       # Express middleware components
│   ├── routes/          # API route handlers
│   ├── utils/           # Utility functions and configuration
│   ├── app.js           # Express application setup
│   └── server.js        # Server startup and configuration
├── tests/
│   ├── unit/           # Unit tests for individual components
│   └── integration/    # API endpoint integration tests
├── docs/
│   └── openapi.yaml    # API specification
├── .env                # Environment variables
├── .dockerignore       # Docker ignore patterns
├── Dockerfile          # Container definition
├── kubernetes.yaml     # Kubernetes deployment manifest
└── README.md          # Project documentation
```

### 5. Package.json Scripts Configuration
Update package.json with essential scripts:

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

### 6. Environment Configuration
Create `.env` file with configurable settings:

```env
# Server Configuration
PORT=3000
NODE_ENV=development

# Logging Configuration
LOG_LEVEL=info

# API Configuration
API_VERSION=v1
```

### 7. Git and Docker Ignore Files
Create `.gitignore`:
```
node_modules/
.env
*.log
coverage/
.nyc_output/
```

Create `.dockerignore`:
```
node_modules/
.git/
.env
*.log
coverage/
tests/
README.md
```

## Architecture Integration
This setup aligns with the three-layer architecture defined in the system design:
- **API Layer**: Routes directory for Express.js controllers
- **Business Logic**: Utils directory for service components
- **Infrastructure**: Middleware directory for cross-cutting concerns

## Quality Assurance
- All dependencies use specific versions for reproducible builds
- Development tools support >90% test coverage requirement
- Project structure follows Node.js community best practices
- Environment-based configuration supports deployment flexibility

## Next Steps
After completing this task:
1. Verify installation with `npm install`
2. Test development server with `npm run dev`
3. Confirm project structure matches specification
4. Proceed to implement core application files (app.js, server.js)

## Dependencies and Integration
- **Downstream Tasks**: All subsequent tasks depend on this foundation
- **Architecture Compliance**: Follows patterns defined in architecture.md
- **PRD Alignment**: Meets technical stack requirements from prd.txt