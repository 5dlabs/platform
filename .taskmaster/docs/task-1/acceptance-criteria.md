# Task 1: Setup Project Structure and Dependencies - Acceptance Criteria

## Overview
This document defines the acceptance criteria for the project initialization task. All criteria must be met for the task to be considered complete.

## Core Acceptance Criteria

### 1. Project Initialization
- [ ] Directory `hello-world-api` exists
- [ ] `package.json` file exists with correct project metadata
- [ ] Project name is set to "hello-world-api"
- [ ] Main entry point is set to "src/server.js"
- [ ] Description mentions "REST API" and "5D Labs orchestrator"

### 2. Dependencies Installation

#### Core Dependencies (Exact Versions)
- [ ] express@4.18.2 installed
- [ ] cors@2.8.5 installed
- [ ] helmet@7.0.0 installed
- [ ] pino@8.15.0 installed
- [ ] pino-http@8.5.0 installed
- [ ] dotenv@16.3.1 installed

#### Development Dependencies (Exact Versions)
- [ ] jest@29.6.4 installed
- [ ] supertest@6.3.3 installed
- [ ] nodemon@3.0.1 installed
- [ ] eslint@8.48.0 installed
- [ ] swagger-jsdoc@6.2.8 installed
- [ ] swagger-ui-express@5.0.0 installed

### 3. Directory Structure
```
✓ hello-world-api/
  ✓ src/
    ✓ middleware/
    ✓ routes/
    ✓ utils/
    ✓ app.js (file exists)
    ✓ server.js (file exists)
  ✓ tests/
    ✓ unit/
    ✓ integration/
  ✓ docs/
    ✓ openapi.yaml (file exists)
  ✓ .env (file exists)
  ✓ .dockerignore (file exists)
  ✓ .gitignore (file exists)
  ✓ Dockerfile (file exists)
  ✓ kubernetes.yaml (file exists)
  ✓ README.md (file exists)
  ✓ package.json (file exists)
  ✓ package-lock.json (file exists)
```

### 4. NPM Scripts Configuration
Verify in `package.json`:
- [ ] `"start": "node src/server.js"`
- [ ] `"dev": "nodemon src/server.js"`
- [ ] `"test": "jest --coverage"`
- [ ] `"lint": "eslint ."`

### 5. Environment Configuration
`.env` file contains:
- [ ] `PORT=3000`
- [ ] `NODE_ENV=development`
- [ ] `LOG_LEVEL=info`
- [ ] `API_VERSION=1.0.0`

### 6. Git Ignore Configuration
`.gitignore` includes:
- [ ] `node_modules/`
- [ ] `.env`
- [ ] `coverage/`
- [ ] `*.log`
- [ ] OS-specific files (.DS_Store, Thumbs.db)

### 7. Docker Ignore Configuration
`.dockerignore` includes:
- [ ] `node_modules`
- [ ] `.git`
- [ ] `.env`
- [ ] `tests`
- [ ] `coverage`

### 8. ESLint Configuration
`.eslintrc.json` exists with:
- [ ] Node.js environment enabled
- [ ] Jest environment enabled
- [ ] Basic code style rules defined

### 9. Jest Configuration
`package.json` includes Jest config:
- [ ] Test environment set to "node"
- [ ] Coverage directory configured
- [ ] Test match patterns defined
- [ ] Coverage collection configured

## Test Cases

### Test Case 1: Verify Installation
```bash
cd hello-world-api
npm list --depth=0
```
**Expected**: All dependencies listed with correct versions

### Test Case 2: Check Directory Structure
```bash
find . -type d -name "node_modules" -prune -o -type d -print | sort
```
**Expected**: All required directories exist

### Test Case 3: Verify Scripts
```bash
npm run
```
**Expected**: Shows all four scripts (start, dev, test, lint)

### Test Case 4: Test Script Execution
```bash
npm run lint
```
**Expected**: ESLint runs without configuration errors

### Test Case 5: Environment File Check
```bash
cat .env | grep -E "PORT|NODE_ENV|LOG_LEVEL|API_VERSION"
```
**Expected**: All four environment variables present

### Test Case 6: Dependency Version Check
```bash
npm list express | grep express@
```
**Expected**: Shows express@4.18.2

## Validation Checklist

### Manual Verification
1. Open `package.json` and verify:
   - [ ] All scripts defined correctly
   - [ ] All dependencies have exact versions (no ^ or ~)
   - [ ] Jest configuration present
   - [ ] Engine requirement specifies Node.js >=18.0.0

2. Check file existence:
   - [ ] All directories created
   - [ ] All placeholder files created (even if empty)
   - [ ] Configuration files have content

3. Run verification commands:
   - [ ] `npm install` completes without errors
   - [ ] `npm run dev` executes (may fail to start but command runs)
   - [ ] `npm test` executes (reports no tests)
   - [ ] `npm run lint` executes

## Common Failure Points

1. **Missing exact versions**: Dependencies installed with ^ or ~ prefixes
2. **Incomplete structure**: Missing directories or files
3. **Wrong script paths**: Scripts pointing to incorrect file locations
4. **Missing configurations**: Empty or missing config files
5. **Permission issues**: Files created without proper permissions

## Success Confirmation
Task is complete when:
- All checkboxes above are checked ✓
- All test cases pass
- No npm warnings about missing dependencies
- Project structure matches specification exactly
- Configuration files contain required content