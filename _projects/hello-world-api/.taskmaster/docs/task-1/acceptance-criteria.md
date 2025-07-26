# Acceptance Criteria: Initialize Node.js Project

## Overview
This document defines the acceptance criteria for Task 1: Initialize Node.js Project. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Project Directory Structure
- [ ] A directory named `hello-world-api` exists
- [ ] The directory contains a `src` subdirectory
- [ ] The directory contains a `node_modules` subdirectory (after npm install)
- [ ] All project files are contained within the `hello-world-api` directory

### 2. Package Configuration
- [ ] A valid `package.json` file exists in the project root
- [ ] The package name is set to "hello-world-api"
- [ ] The version is set to "1.0.0"
- [ ] The description is set to "A simple Hello World API"
- [ ] The main entry point is set to "src/index.js"
- [ ] The scripts section contains a "start" script with value "node src/index.js"

### 3. Dependencies
- [ ] Express.js is listed in the dependencies section of package.json
- [ ] Express.js version is ^4.18.2 or compatible 4.x version
- [ ] A `package-lock.json` file exists (created by npm)
- [ ] Running `npm list express` shows Express is installed

### 4. Source Files
- [ ] File `src/index.js` exists
- [ ] The file contains valid JavaScript code that:
  - [ ] Imports Express.js using `require('express')`
  - [ ] Creates an Express application instance
  - [ ] Sets PORT to use environment variable or default to 3000
  - [ ] Includes request logging middleware
  - [ ] Includes server startup code with console log

### 5. Version Control Setup
- [ ] A `.gitignore` file exists in the project root
- [ ] The `.gitignore` file contains an entry for `node_modules/`
- [ ] The `.gitignore` file contains an entry for `.env`

### 6. Functional Requirements
- [ ] Running `npm start` executes without errors
- [ ] The server starts and displays "Server running on http://localhost:3000"
- [ ] The server listens on port 3000 (or PORT environment variable)
- [ ] HTTP requests to the server are logged to console with timestamp

## Test Cases

### Test Case 1: Project Structure Validation
```bash
# Navigate to project directory
cd hello-world-api

# Verify directory structure
ls -la
# Expected: src/, node_modules/, package.json, package-lock.json, .gitignore

ls src/
# Expected: index.js
```

### Test Case 2: Package.json Validation
```bash
# Check package.json content
cat package.json | grep '"name"'
# Expected: "name": "hello-world-api"

cat package.json | grep '"start"'
# Expected: "start": "node src/index.js"
```

### Test Case 3: Dependency Validation
```bash
# Check Express installation
npm list express
# Expected: express@4.x.x

# Verify in package.json
cat package.json | grep '"express"'
# Expected: "express": "^4.18.2" (or similar 4.x version)
```

### Test Case 4: Server Startup Test
```bash
# Start the server
npm start

# Expected output:
# > hello-world-api@1.0.0 start
# > node src/index.js
# 
# Server running on http://localhost:3000
```

### Test Case 5: Request Logging Test
```bash
# With server running, in another terminal:
curl http://localhost:3000

# Expected in server console:
# 2024-01-15T10:30:45.123Z - GET /
```

### Test Case 6: Environment Variable Test
```bash
# Start server with custom port
PORT=4000 npm start

# Expected output:
# Server running on http://localhost:4000
```

## Definition of Done

The task is complete when:
1. All acceptance criteria checkboxes above can be marked as complete
2. All test cases pass successfully
3. The server can be started and stopped without errors
4. The project structure follows Node.js best practices
5. No npm audit vulnerabilities at high or critical level

## Non-Functional Requirements

- **Performance**: Server should start within 2 seconds
- **Compatibility**: Must work with Node.js version 20 or higher
- **Code Quality**: No ESLint errors (if linter is configured)
- **Documentation**: Code includes appropriate comments
- **Security**: No sensitive information in committed files