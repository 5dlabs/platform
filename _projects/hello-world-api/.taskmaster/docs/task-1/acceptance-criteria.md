# Task 1: Setup Project Structure and Dependencies - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 1, which involves setting up the initial project structure and installing dependencies for the Hello World API project.

## Acceptance Criteria

### 1. Project Initialization
- [ ] A directory named `hello-world-api` exists
- [ ] The directory contains a valid `package.json` file
- [ ] The package.json has the correct project name: `"hello-world-api"`
- [ ] The package.json has a meaningful description
- [ ] The main entry point is set to `"src/server.js"`

### 2. Dependencies Installation

#### Core Dependencies (Exact Versions)
- [ ] express@4.18.2 is listed in dependencies
- [ ] cors@2.8.5 is listed in dependencies  
- [ ] helmet@7.0.0 is listed in dependencies
- [ ] pino@8.15.0 is listed in dependencies
- [ ] pino-http@8.5.0 is listed in dependencies
- [ ] dotenv@16.3.1 is listed in dependencies

#### Development Dependencies (Exact Versions)
- [ ] jest@29.6.4 is listed in devDependencies
- [ ] supertest@6.3.3 is listed in devDependencies
- [ ] nodemon@3.0.1 is listed in devDependencies
- [ ] eslint@8.48.0 is listed in devDependencies
- [ ] swagger-jsdoc@6.2.8 is listed in devDependencies
- [ ] swagger-ui-express@5.0.0 is listed in devDependencies

### 3. Project Structure

#### Directory Structure
- [ ] `/src` directory exists
- [ ] `/src/middleware` directory exists
- [ ] `/src/routes` directory exists
- [ ] `/src/utils` directory exists
- [ ] `/tests` directory exists
- [ ] `/tests/unit` directory exists
- [ ] `/tests/integration` directory exists
- [ ] `/docs` directory exists

#### Required Files
- [ ] `src/app.js` file exists
- [ ] `src/server.js` file exists
- [ ] `docs/openapi.yaml` file exists
- [ ] `.env` file exists
- [ ] `.dockerignore` file exists
- [ ] `.gitignore` file exists
- [ ] `Dockerfile` file exists
- [ ] `kubernetes.yaml` file exists
- [ ] `README.md` file exists

### 4. NPM Scripts Configuration

The package.json must contain the following scripts:
- [ ] `"start": "node src/server.js"`
- [ ] `"dev": "nodemon src/server.js"`
- [ ] `"test": "jest --coverage"`
- [ ] `"test:watch": "jest --watch"`
- [ ] `"test:unit": "jest tests/unit"`
- [ ] `"test:integration": "jest tests/integration"`
- [ ] `"lint": "eslint ."`
- [ ] `"lint:fix": "eslint . --fix"`

### 5. Jest Configuration

The package.json must contain Jest configuration:
- [ ] `"testEnvironment": "node"`
- [ ] `"coverageDirectory": "coverage"`
- [ ] `collectCoverageFrom` includes `"src/**/*.js"`
- [ ] `collectCoverageFrom` excludes `"!src/server.js"`

### 6. Engine Requirements
- [ ] package.json specifies `"engines": { "node": ">=16.0.0" }`

### 7. Environment Configuration

The `.env` file must contain:
- [ ] `PORT=3000`
- [ ] `NODE_ENV=development`
- [ ] `LOG_LEVEL=info`
- [ ] `API_VERSION=1.0.0`
- [ ] `API_PREFIX=/api/v1`

### 8. Git Ignore Configuration

The `.gitignore` file must include:
- [ ] `node_modules/`
- [ ] `.env` and `.env.*` patterns
- [ ] `logs/` and `*.log` patterns
- [ ] `coverage/` directory
- [ ] Common IDE directories (`.vscode/`, `.idea/`)
- [ ] OS-specific files (`.DS_Store`, `Thumbs.db`)

### 9. Docker Ignore Configuration

The `.dockerignore` file must include:
- [ ] `node_modules/`
- [ ] `.git/` directory
- [ ] `.env` and `.env.*` files
- [ ] `tests/` directory
- [ ] `coverage/` directory
- [ ] Development config files (`.eslintrc.js`, etc.)

## Test Scenarios

### Test 1: Dependency Installation
```bash
# Should complete without errors
npm install

# Verify node_modules exists
ls -la node_modules/

# Check specific packages
npm list express
npm list jest
```

### Test 2: Script Execution
```bash
# Development server (may fail to start but shouldn't have syntax errors)
npm run dev

# Linter should run without configuration errors
npm run lint

# Test runner should execute (no tests yet is okay)
npm test
```

### Test 3: Directory Structure Validation
```bash
# Check all required directories exist
find . -type d -name "middleware" | grep -q "src/middleware" && echo "✓ middleware dir exists"
find . -type d -name "routes" | grep -q "src/routes" && echo "✓ routes dir exists"
find . -type d -name "utils" | grep -q "src/utils" && echo "✓ utils dir exists"
find . -type d -name "unit" | grep -q "tests/unit" && echo "✓ unit tests dir exists"
find . -type d -name "integration" | grep -q "tests/integration" && echo "✓ integration tests dir exists"
```

### Test 4: File Existence
```bash
# Check all required files exist
[ -f "src/app.js" ] && echo "✓ app.js exists"
[ -f "src/server.js" ] && echo "✓ server.js exists"
[ -f ".env" ] && echo "✓ .env exists"
[ -f ".gitignore" ] && echo "✓ .gitignore exists"
[ -f ".dockerignore" ] && echo "✓ .dockerignore exists"
[ -f "Dockerfile" ] && echo "✓ Dockerfile exists"
```

### Test 5: Environment Variables
```bash
# Check .env file contains required variables
grep -q "PORT=3000" .env && echo "✓ PORT configured"
grep -q "NODE_ENV=development" .env && echo "✓ NODE_ENV configured"
grep -q "LOG_LEVEL=info" .env && echo "✓ LOG_LEVEL configured"
grep -q "API_VERSION=1.0.0" .env && echo "✓ API_VERSION configured"
grep -q "API_PREFIX=/api/v1" .env && echo "✓ API_PREFIX configured"
```

## Definition of Done

This task is considered complete when:
1. All acceptance criteria checkboxes above can be checked
2. All test scenarios pass successfully
3. The project structure matches the specification exactly
4. Dependencies are installed with correct versions
5. Configuration files are properly formatted
6. No errors occur when running `npm install`
7. The project is ready for API endpoint implementation

## Non-Functional Requirements

- Installation should complete in under 2 minutes on a standard development machine
- The project structure should be intuitive and follow Node.js conventions
- All configuration should be environment-agnostic
- The setup should work on Windows, macOS, and Linux
- Total project size (excluding node_modules) should be under 1MB

## Notes for Reviewers

When reviewing this task:
1. Verify exact dependency versions match requirements
2. Ensure no additional dependencies were added
3. Check that all directories exist, even if empty
4. Confirm environment variables use sensible defaults
5. Validate that ignore files follow best practices
6. Test that npm scripts are properly configured