# Task 1: Setup Project Structure and Dependencies

## Overview
This task establishes the foundational structure for the Hello World API project, setting up a Node.js/Express.js environment with all necessary dependencies and configurations for a production-ready REST API service.

## Objectives
- Initialize a new Node.js project with proper package.json configuration
- Install all required core and development dependencies
- Create a scalable and maintainable project structure
- Configure development scripts and environment settings
- Prepare the project for containerization and deployment

## Technical Approach

### Technology Stack
- **Runtime**: Node.js (LTS version)
- **Framework**: Express.js 4.18.2
- **Security**: Helmet 7.0.0 for security headers, CORS 2.8.5 for cross-origin support
- **Logging**: Pino 8.15.0 with pino-http 8.5.0 for structured logging
- **Configuration**: dotenv 16.3.1 for environment management
- **Testing**: Jest 29.6.4 with Supertest 6.3.3
- **Development**: Nodemon 3.0.1, ESLint 8.48.0
- **Documentation**: Swagger-jsdoc 6.2.8, swagger-ui-express 5.0.0

### Project Structure Design
```
hello-world-api/
├── src/                      # Source code
│   ├── middleware/           # Express middleware components
│   ├── routes/              # API route definitions
│   ├── utils/               # Utility functions and helpers
│   ├── app.js               # Express app configuration
│   └── server.js            # Server entry point
├── tests/                   # Test suites
│   ├── unit/               # Unit tests
│   └── integration/        # Integration tests
├── docs/                   # Documentation
│   └── openapi.yaml        # OpenAPI specification
├── .env                    # Environment variables
├── .dockerignore          # Docker ignore patterns
├── .gitignore             # Git ignore patterns
├── Dockerfile             # Container definition
├── kubernetes.yaml        # K8s deployment manifest
├── package.json           # Project metadata and scripts
└── README.md              # Project documentation
```

## Implementation Details

### Step 1: Project Initialization
Create the project directory and initialize npm:
```bash
mkdir hello-world-api
cd hello-world-api
npm init -y
```

### Step 2: Dependency Installation

#### Core Dependencies
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

#### Development Dependencies
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### Step 3: Directory Structure Creation
```bash
# Create source directories
mkdir -p src/{middleware,routes,utils}
mkdir -p tests/{unit,integration}
mkdir docs

# Create initial files
touch src/{app.js,server.js}
touch docs/openapi.yaml
touch {.env,.dockerignore,Dockerfile,kubernetes.yaml,README.md}
```

### Step 4: Package.json Configuration
Update package.json with the following scripts:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple REST API for testing 5D Labs orchestrator workflow",
  "main": "src/server.js",
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "test:unit": "jest tests/unit",
    "test:integration": "jest tests/integration",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix"
  },
  "engines": {
    "node": ">=16.0.0"
  },
  "jest": {
    "testEnvironment": "node",
    "coverageDirectory": "coverage",
    "collectCoverageFrom": [
      "src/**/*.js",
      "!src/server.js"
    ]
  }
}
```

### Step 5: Environment Configuration
Create .env file with default values:
```env
# Server Configuration
PORT=3000
NODE_ENV=development

# Logging
LOG_LEVEL=info

# API Configuration
API_VERSION=1.0.0
API_PREFIX=/api/v1
```

### Step 6: Ignore Files Configuration

**.gitignore**:
```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Logs
logs/
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Testing
coverage/
.nyc_output/

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Build outputs
dist/
build/
```

**.dockerignore**:
```
node_modules/
npm-debug.log
.git/
.gitignore
.env
.env.*
coverage/
.nyc_output/
tests/
.eslintrc.js
.prettierrc
README.md
.vscode/
.idea/
```

## Dependencies and Requirements

### Runtime Requirements
- Node.js 16.x or higher
- npm 7.x or higher

### Development Environment
- Git for version control
- Docker for containerization (optional for local development)
- Code editor with JavaScript/Node.js support

### Dependency Rationale
- **Express.js**: Industry-standard web framework for Node.js
- **Helmet**: Adds security headers to protect against common vulnerabilities
- **CORS**: Enables cross-origin resource sharing for API access
- **Pino**: Fast, low-overhead structured logging
- **dotenv**: Manages environment variables from .env files
- **Jest**: Popular testing framework with good Express.js integration
- **Supertest**: HTTP assertion library for testing Express apps
- **Nodemon**: Auto-restarts server during development
- **ESLint**: Enforces code quality and consistency
- **Swagger**: API documentation and testing interface

## Success Criteria
- All dependencies install without errors
- Project structure is created as specified
- npm scripts execute successfully
- Development server starts with `npm run dev`
- Test runner executes with `npm test`
- Linter runs with `npm run lint`
- Environment variables are loaded from .env file

## Next Steps
After completing this task, the project will be ready for:
- Task 2: Basic Express Server Setup and Configuration
- Task 3: Health Check Endpoint Implementation
- Task 4: Hello World Endpoints
- Task 5: Echo Service Endpoint
- Task 6: Info Endpoint for Service Metadata
- Task 7: OpenAPI Documentation and Swagger UI
- Task 8: Unit and Integration Tests