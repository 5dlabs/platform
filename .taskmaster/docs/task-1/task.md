# Task 1: Setup Project Structure and Dependencies

## Overview
This task establishes the foundation for the Hello World API by initializing a Node.js project with Express.js and creating the necessary project structure. This is the critical first step that enables all subsequent development work.

## Technical Requirements

### Dependencies to Install

**Core Dependencies:**
- `express@4.18.2` - Web framework for routing and middleware
- `cors@2.8.5` - Cross-Origin Resource Sharing middleware
- `helmet@7.0.0` - Security headers middleware
- `pino@8.15.0` - Fast structured JSON logger
- `pino-http@8.5.0` - HTTP request logger for Pino
- `dotenv@16.3.1` - Environment variable management

**Development Dependencies:**
- `jest@29.6.4` - Testing framework
- `supertest@6.3.3` - HTTP assertion library for testing
- `nodemon@3.0.1` - Auto-restart on file changes during development
- `eslint@8.48.0` - Code linting
- `swagger-jsdoc@6.2.8` - Generate OpenAPI spec from JSDoc
- `swagger-ui-express@5.0.0` - Serve API documentation UI

### Project Structure
```
hello-world-api/
├── src/
│   ├── middleware/      # Express middleware modules
│   ├── routes/          # API route definitions
│   ├── utils/           # Utility functions
│   ├── app.js          # Express app configuration
│   └── server.js       # Server startup file
├── tests/
│   ├── unit/           # Unit test files
│   └── integration/    # Integration test files
├── docs/
│   └── openapi.yaml    # OpenAPI specification
├── .env                # Environment variables
├── .dockerignore       # Docker ignore patterns
├── .gitignore          # Git ignore patterns
├── Dockerfile          # Container definition
├── kubernetes.yaml     # K8s deployment manifest
├── package.json        # Project metadata and scripts
└── README.md           # Project documentation
```

## Implementation Steps

### Step 1: Initialize Project
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

# Create docs directory
mkdir docs

# Create placeholder files
touch src/app.js src/server.js
touch docs/openapi.yaml
touch .env .dockerignore Dockerfile kubernetes.yaml README.md
```

### Step 4: Configure package.json Scripts
Update `package.json` with the following scripts section:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "Simple REST API for testing 5D Labs orchestrator workflow",
  "main": "src/server.js",
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  },
  "engines": {
    "node": ">=18.0.0"
  }
}
```

### Step 5: Create Environment Configuration
Create `.env` file:
```
PORT=3000
NODE_ENV=development
LOG_LEVEL=info
API_VERSION=1.0.0
```

### Step 6: Configure .gitignore
```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Test coverage
coverage/
*.lcov

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Log files
*.log
npm-debug.log*

# Build outputs
dist/
build/
```

### Step 7: Configure .dockerignore
```
node_modules
npm-debug.log
.git
.gitignore
.env
coverage
.nyc_output
.vscode
.idea
*.md
tests
docs
```

## Configuration Details

### Jest Configuration
Add to `package.json`:
```json
"jest": {
  "testEnvironment": "node",
  "coverageDirectory": "coverage",
  "collectCoverageFrom": [
    "src/**/*.js",
    "!src/server.js"
  ],
  "testMatch": [
    "**/tests/**/*.test.js"
  ],
  "verbose": true
}
```

### ESLint Configuration
Create `.eslintrc.json`:
```json
{
  "env": {
    "node": true,
    "es2021": true,
    "jest": true
  },
  "extends": "eslint:recommended",
  "parserOptions": {
    "ecmaVersion": "latest"
  },
  "rules": {
    "indent": ["error", 2],
    "quotes": ["error", "single"],
    "semi": ["error", "always"]
  }
}
```

## Verification Steps

1. **Verify Installation**: Run `npm install` and ensure no errors
2. **Check Dependencies**: Run `npm list` to verify all packages installed
3. **Test Scripts**: Run each npm script to ensure they execute:
   - `npm run dev` - Should attempt to start server
   - `npm run lint` - Should run ESLint
   - `npm test` - Should run Jest (will report no tests)
4. **Verify Structure**: Use `tree` or `ls -la` to confirm directory structure

## Common Issues and Solutions

- **Port Already in Use**: Change PORT in .env file
- **Permission Errors**: Run with appropriate permissions or change npm prefix
- **Missing Dependencies**: Delete node_modules and package-lock.json, then reinstall
- **Version Conflicts**: Use exact versions specified to ensure compatibility

## Next Steps
After completing this task, the project will be ready for:
- Task 2: Basic Express server setup
- Task 3: Health check endpoint implementation
- Task 4: Core API endpoints development