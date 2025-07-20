# Task 1: Project Setup and Configuration - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 1: Project Setup and Configuration. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### 1. Project Structure ✓
- [ ] Root directory contains `simple-api/` project folder
- [ ] `src/` directory exists with subdirectories:
  - [ ] `controllers/`
  - [ ] `models/`
  - [ ] `routes/`
  - [ ] `middleware/`
- [ ] `tests/` directory exists with subdirectories:
  - [ ] `unit/` with subdirectories for `models/`, `controllers/`, `middleware/`
  - [ ] `integration/`
  - [ ] `fixtures/`
- [ ] `data/` directory exists for database storage
- [ ] `docs/` directory exists for documentation

### 2. Node.js Project Configuration ✓
- [ ] `package.json` exists in project root
- [ ] Package name is set to "simple-todo-api"
- [ ] Version is set to "1.0.0"
- [ ] Description includes "REST API for managing todo items"
- [ ] Main entry point is "server.js"
- [ ] License is specified (MIT recommended)

### 3. Dependencies Installation ✓
- [ ] **Production Dependencies Installed:**
  - [ ] express (^4.18.2 or compatible)
  - [ ] better-sqlite3 (^9.0.0 or compatible)
  - [ ] express-validator (^7.0.1 or compatible)
  - [ ] swagger-ui-express (^5.0.0 or compatible)
  - [ ] swagger-jsdoc (^6.2.8 or compatible)
  - [ ] dotenv (^16.3.1 or compatible)
- [ ] **Development Dependencies Installed:**
  - [ ] jest (^29.7.0 or compatible)
  - [ ] supertest (^6.3.3 or compatible)
  - [ ] nodemon (^3.0.1 or compatible)
  - [ ] prettier (^3.0.3 or compatible)
  - [ ] @types/jest (^29.5.5 or compatible)

### 4. NPM Scripts Configuration ✓
- [ ] `npm start` - Runs production server
- [ ] `npm run dev` - Runs development server with nodemon
- [ ] `npm test` - Runs Jest test suite with coverage
- [ ] `npm run test:watch` - Runs tests in watch mode
- [ ] `npm run test:coverage` - Generates coverage report
- [ ] `npm run format` - Formats code with Prettier
- [ ] `npm run lint` - Checks code formatting
- [ ] All scripts execute without errors

### 5. Environment Configuration ✓
- [ ] `.env.example` file exists with:
  - [ ] PORT configuration
  - [ ] NODE_ENV configuration
  - [ ] DB_PATH configuration
  - [ ] API_PREFIX configuration
  - [ ] LOG_LEVEL configuration
- [ ] `.env` file created from `.env.example`
- [ ] Environment variables properly commented

### 6. Code Formatting Configuration ✓
- [ ] `.prettierrc` file exists with team standards:
  - [ ] Semi-colons enabled
  - [ ] Single quotes for strings
  - [ ] 2-space indentation
  - [ ] Line width of 80 characters
- [ ] `.prettierignore` excludes:
  - [ ] node_modules/
  - [ ] coverage/
  - [ ] data/
  - [ ] *.db files

### 7. Version Control Configuration ✓
- [ ] `.gitignore` file exists and includes:
  - [ ] node_modules/
  - [ ] .env files (except .env.example)
  - [ ] Database files (*.db, *.sqlite)
  - [ ] Coverage reports
  - [ ] IDE-specific files
  - [ ] OS-specific files
  - [ ] Log files

### 8. Documentation ✓
- [ ] `README.md` exists with:
  - [ ] Project title and description
  - [ ] Prerequisites (Node.js 18+)
  - [ ] Installation instructions
  - [ ] Available npm scripts
  - [ ] Getting started guide
  - [ ] Link to task documentation

## Non-Functional Acceptance Criteria

### Performance
- [ ] npm install completes in under 2 minutes on standard hardware
- [ ] Project structure can be created in under 30 seconds

### Compatibility
- [ ] Setup works on Windows, macOS, and Linux
- [ ] Compatible with Node.js 18.x and higher
- [ ] No platform-specific dependencies without fallbacks

### Developer Experience
- [ ] Clear error messages if prerequisites not met
- [ ] All configuration files have helpful comments
- [ ] Setup process documented step-by-step

## Test Cases

### Test Case 1: Fresh Installation
```bash
# Starting from empty directory
git clone <repository>
cd simple-api
npm install
```
**Expected Result**: All dependencies install without errors

### Test Case 2: Environment Setup
```bash
cp .env.example .env
node -e "require('dotenv').config(); console.log(process.env.PORT)"
```
**Expected Result**: Outputs "3000"

### Test Case 3: Prettier Configuration
```bash
echo "const x={a:1,b:2}" > test.js
npm run format
cat test.js
rm test.js
```
**Expected Result**: File is reformatted according to .prettierrc rules

### Test Case 4: Project Structure Verification
```bash
find . -type d -name "controllers" | grep -q "src/controllers" && echo "PASS" || echo "FAIL"
find . -type d -name "models" | grep -q "src/models" && echo "PASS" || echo "FAIL"
find . -type d -name "routes" | grep -q "src/routes" && echo "PASS" || echo "FAIL"
```
**Expected Result**: All checks output "PASS"

### Test Case 5: Script Execution
```bash
npm run lint
npm run format
```
**Expected Result**: Both commands execute without errors

## Definition of Done
- [ ] All functional acceptance criteria are met
- [ ] All non-functional acceptance criteria are met
- [ ] All test cases pass successfully
- [ ] No errors or warnings during setup process
- [ ] Documentation is complete and accurate
- [ ] Setup tested on at least one development machine
- [ ] Code formatting tools are working correctly

## Notes
- If using Windows, ensure Git Bash or WSL is available for shell commands
- SQLite may require additional build tools on some platforms
- Node.js version 18 or higher is a hard requirement