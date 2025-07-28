# Task 1: Setup Project Structure and Dependencies

## Overview
This task establishes the foundational structure for the Hello World API project, including Node.js initialization, dependency installation, and creation of the project directory hierarchy. This is the critical first step that enables all subsequent development.

## Objectives
- Initialize a Node.js project with proper configuration
- Install all required core and development dependencies
- Create a scalable project directory structure
- Configure essential project scripts and environment settings
- Verify successful project setup

## Technical Approach

### Project Initialization
The project will be initialized using Node.js with npm as the package manager. The `npm init -y` command creates a default package.json file that will be customized with project-specific metadata and scripts.

### Dependencies Overview

#### Core Dependencies
- **express@4.18.2**: Web framework for building REST APIs
- **cors@2.8.5**: Cross-Origin Resource Sharing middleware
- **helmet@7.0.0**: Security headers middleware
- **pino@8.15.0**: High-performance JSON logger
- **pino-http@8.5.0**: HTTP request logging middleware
- **dotenv@16.3.1**: Environment variable management

#### Development Dependencies
- **jest@29.6.4**: Testing framework
- **supertest@6.3.3**: HTTP API testing library
- **nodemon@3.0.1**: Development server with auto-reload
- **eslint@8.48.0**: Code linting and style enforcement
- **swagger-jsdoc@6.2.8**: OpenAPI documentation generation
- **swagger-ui-express@5.0.0**: Interactive API documentation UI

### Directory Structure
```
hello-world-api/
├── src/                    # Source code
│   ├── middleware/         # Express middleware
│   ├── routes/            # API route definitions
│   ├── utils/             # Utility functions
│   ├── app.js             # Express app configuration
│   └── server.js          # Server startup file
├── tests/                 # Test suites
│   ├── unit/              # Unit tests
│   └── integration/       # API integration tests
├── docs/                  # Documentation
│   └── openapi.yaml       # OpenAPI specification
├── .env                   # Environment variables
├── .dockerignore          # Docker ignore patterns
├── .gitignore             # Git ignore patterns
├── Dockerfile             # Container definition
├── kubernetes.yaml        # K8s deployment manifest
├── package.json           # Project metadata
└── README.md              # Project documentation
```

## Implementation Details

### Step 1: Initialize Node.js Project
```bash
mkdir hello-world-api
cd hello-world-api
npm init -y
```

### Step 2: Install Dependencies
```bash
# Core dependencies
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1

# Development dependencies
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### Step 3: Create Directory Structure
```bash
# Create source directories
mkdir -p src/middleware src/routes src/utils

# Create test directories
mkdir -p tests/unit tests/integration

# Create documentation directory
mkdir docs

# Create essential files
touch src/app.js src/server.js
touch docs/openapi.yaml
touch .env .dockerignore Dockerfile kubernetes.yaml README.md
```

### Step 4: Configure package.json
Update package.json with the following scripts:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple REST API for testing the 5D Labs orchestrator workflow",
  "main": "src/server.js",
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  },
  "keywords": ["api", "rest", "express", "hello-world"],
  "author": "5D Labs",
  "license": "MIT"
}
```

### Step 5: Configure Environment Files

**.env file:**
```env
# Server Configuration
PORT=3000
NODE_ENV=development

# API Configuration
API_VERSION=1.0.0
LOG_LEVEL=info
```

**.gitignore file:**
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

# Test coverage
coverage/
.nyc_output/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Build outputs
dist/
build/
```

**.dockerignore file:**
```
node_modules/
npm-debug.log
.git/
.gitignore
.env
.env.*
coverage/
.nyc_output/
.vscode/
.idea/
*.md
tests/
docs/
```

## Dependencies and Requirements
- Node.js version 16.x or higher
- npm version 8.x or higher
- Write access to the project directory
- Internet connection for downloading npm packages

## Success Criteria
- All dependencies install without errors
- Project structure matches the specification
- Development server starts successfully with `npm run dev`
- No lint errors when running `npm run lint`
- Test framework initializes properly with `npm test`

## Related Tasks
- Task 2: Create Core API Structure (depends on this task)
- Task 3: Implement API Endpoints (depends on this task)
- All subsequent tasks depend on successful completion of this project setup