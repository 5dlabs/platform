# Task 1: Project Setup and Configuration - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 1: Project Setup and Configuration. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Project Structure ✓

**Given** a new project directory
**When** the setup is complete
**Then** the following directory structure must exist:

```
simple-api/
├── src/
│   ├── controllers/      # Empty directory
│   ├── models/          # Empty directory
│   ├── routes/          # Empty directory
│   └── middleware/      # Empty directory
├── tests/
│   ├── unit/
│   │   ├── models/      # Empty directory
│   │   ├── controllers/ # Empty directory
│   │   └── middleware/  # Empty directory
│   └── integration/     # Empty directory
├── data/                # Empty directory
├── package.json
├── .env
├── .env.example
├── .gitignore
└── README.md
```

**Test**: Run `tree -d` or manually verify each directory exists

### 2. NPM Package Configuration ✓

**Given** the project is initialized
**When** checking package.json
**Then** it must contain:

- Name: "simple-api" (or appropriate project name)
- Version: "1.0.0"
- Main: "server.js"
- All required dependencies with appropriate versions
- All required scripts configured

**Test**: Run `cat package.json` and verify contents

### 3. Dependencies Installation ✓

**Given** package.json is configured
**When** running `npm list --depth=0`
**Then** the following packages must be installed:

Production dependencies:
- express
- better-sqlite3
- express-validator
- swagger-ui-express
- swagger-jsdoc

Development dependencies:
- jest
- supertest
- nodemon
- prettier

**Test**: Run `npm list --depth=0` and verify no missing dependencies

### 4. NPM Scripts Functionality ✓

**Given** npm scripts are configured
**When** running each script
**Then** they must execute without configuration errors:

| Script | Command | Expected Result |
|--------|---------|-----------------|
| start | `npm start` | Attempts to run server.js (error if file doesn't exist is OK) |
| dev | `npm run dev` | Nodemon starts and watches for changes |
| test | `npm test` | Jest runs (no tests found is OK) |
| format | `npm run format` | Prettier runs on all .js files |

**Test**: Run each command and verify behavior

### 5. Environment Configuration ✓

**Given** the project needs environment variables
**When** checking the root directory
**Then** the following files must exist:

`.env` file containing:
```
PORT=3000
NODE_ENV=development
```

`.env.example` file containing the same content

**Test**: Run `cat .env` and `cat .env.example`

### 6. Git Configuration ✓

**Given** the project will use version control
**When** checking .gitignore
**Then** it must contain entries for:

- node_modules/
- .env
- coverage/
- data/
- *.log
- .DS_Store (or OS-specific files)

**Test**: Run `cat .gitignore` and verify entries

### 7. Documentation ✓

**Given** the project needs basic documentation
**When** checking README.md
**Then** it must contain at minimum:

- Project title
- Brief description
- Installation instructions
- How to run the project
- How to run in development mode

**Test**: Run `cat README.md` and verify content

## Test Scenarios

### Scenario 1: Fresh Installation
```bash
# Clone/create project
cd simple-api
npm install
# Should complete without errors
```

### Scenario 2: Development Workflow
```bash
npm run dev
# Should start nodemon watching for changes
# Ctrl+C to stop
```

### Scenario 3: Code Formatting
```bash
echo "const test='unformatted'" > test.js
npm run format
cat test.js
# Should show formatted code
rm test.js
```

### Scenario 4: Test Runner
```bash
npm test
# Should run Jest (no tests is OK)
# Should attempt to generate coverage
```

## Definition of Done

- [ ] All directories exist as specified
- [ ] package.json contains all required dependencies and scripts
- [ ] All dependencies install without errors
- [ ] All npm scripts execute without configuration errors
- [ ] Environment files exist with correct content
- [ ] .gitignore properly configured
- [ ] README.md contains basic documentation
- [ ] No application code exists yet (only configuration)
- [ ] Another developer can clone and run `npm install` successfully

## Notes

- This is a setup task only - no application code should be written
- Empty directories are expected and correct
- Some npm scripts may fail due to missing application files, but should not have configuration errors
- The setup should work on Windows, macOS, and Linux