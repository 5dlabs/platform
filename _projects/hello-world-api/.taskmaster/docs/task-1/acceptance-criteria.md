# Task 1: Setup Project Structure and Dependencies - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. Project Initialization
- [ ] Project directory `hello-world-api` exists
- [ ] Valid `package.json` file is present in the root directory
- [ ] Package name is set to `hello-world-api`
- [ ] Package version is set to `1.0.0`
- [ ] Main entry point is set to `src/server.js`

### 2. Dependencies Installation

#### Core Dependencies (Exact Versions)
- [ ] express@4.18.2 is installed
- [ ] cors@2.8.5 is installed
- [ ] helmet@7.0.0 is installed
- [ ] pino@8.15.0 is installed
- [ ] pino-http@8.5.0 is installed
- [ ] dotenv@16.3.1 is installed

#### Development Dependencies (Exact Versions)
- [ ] jest@29.6.4 is installed as devDependency
- [ ] supertest@6.3.3 is installed as devDependency
- [ ] nodemon@3.0.1 is installed as devDependency
- [ ] eslint@8.48.0 is installed as devDependency
- [ ] swagger-jsdoc@6.2.8 is installed as devDependency
- [ ] swagger-ui-express@5.0.0 is installed as devDependency

### 3. Project Structure

#### Directories
- [ ] `src/` directory exists
- [ ] `src/middleware/` directory exists
- [ ] `src/routes/` directory exists
- [ ] `src/utils/` directory exists
- [ ] `tests/` directory exists
- [ ] `tests/unit/` directory exists
- [ ] `tests/integration/` directory exists
- [ ] `docs/` directory exists

#### Files
- [ ] `src/app.js` file exists
- [ ] `src/server.js` file exists
- [ ] `docs/openapi.yaml` file exists (can be empty)
- [ ] `.env` file exists
- [ ] `.dockerignore` file exists
- [ ] `.gitignore` file exists
- [ ] `Dockerfile` file exists (can be empty initially)
- [ ] `kubernetes.yaml` file exists (can be empty initially)
- [ ] `README.md` file exists

### 4. Package.json Scripts
Verify package.json contains these exact scripts:
- [ ] `"start": "node src/server.js"`
- [ ] `"dev": "nodemon src/server.js"`
- [ ] `"test": "jest --coverage"`
- [ ] `"lint": "eslint ."`

### 5. Configuration Files Content

#### .env File
- [ ] Contains `PORT=3000`
- [ ] Contains `NODE_ENV=development`
- [ ] Contains `API_VERSION=1.0.0`
- [ ] Contains `LOG_LEVEL=info`
- [ ] Contains `SERVICE_NAME=hello-world-api`

#### .gitignore File
- [ ] Contains `node_modules/`
- [ ] Contains `.env` and `.env.*` patterns
- [ ] Contains `coverage/`
- [ ] Contains common IDE patterns (`.vscode/`, `.idea/`)
- [ ] Contains OS-specific patterns (`.DS_Store`, `Thumbs.db`)

#### .dockerignore File
- [ ] Contains `node_modules/`
- [ ] Contains `.env` and `.env.*`
- [ ] Contains `tests/`
- [ ] Contains `coverage/`
- [ ] Contains `.git/`

### 6. Application Files

#### src/app.js
- [ ] Imports express, cors, and helmet
- [ ] Creates express application instance
- [ ] Configures helmet() middleware
- [ ] Configures cors() middleware
- [ ] Configures express.json() middleware
- [ ] Configures express.urlencoded() middleware
- [ ] Exports the app instance

#### src/server.js
- [ ] Loads environment variables with dotenv
- [ ] Imports app from ./app
- [ ] Reads PORT from environment with fallback to 3000
- [ ] Starts server and logs the port
- [ ] Implements graceful shutdown on SIGTERM
- [ ] Exports the server instance

### 7. Functional Tests

#### Installation Test
```bash
# This should complete without errors
npm install
```
- [ ] All dependencies install successfully
- [ ] No peer dependency warnings for specified versions
- [ ] package-lock.json is created

#### Server Startup Test
```bash
# This should start the server
npm run dev
```
- [ ] Server starts without errors
- [ ] Logs "Server running on port 3000"
- [ ] Nodemon is watching for file changes
- [ ] Can be stopped with Ctrl+C

#### Basic Server Response Test
```bash
# With server running, in another terminal:
curl http://localhost:3000/
```
- [ ] Server responds (even with 404 is acceptable at this stage)
- [ ] No server crashes

### 8. Code Quality Checks

#### ESLint Configuration
```bash
npm run lint
```
- [ ] ESLint runs (configuration warnings are acceptable)
- [ ] No critical errors in initial files

#### File Permissions
- [ ] All files are readable
- [ ] `.env` file has appropriate permissions (not executable)
- [ ] All directories are accessible

### 9. README Content
- [ ] Contains project title
- [ ] Contains installation instructions
- [ ] Contains commands for running in dev and production mode
- [ ] Contains testing command
- [ ] Mentions API documentation endpoint

### 10. Edge Cases and Error Scenarios

#### Port Already in Use
- [ ] When PORT 3000 is already in use, error message is clear
- [ ] Can be configured to use different port via .env

#### Missing .env File
- [ ] Server still starts with default values
- [ ] Uses fallback PORT 3000

#### Graceful Shutdown
- [ ] Sending SIGTERM signal triggers shutdown message
- [ ] Server closes cleanly without hanging processes

## Test Commands Summary

Run these commands to verify the setup:

```bash
# 1. Check installation
npm install
ls -la node_modules/ | grep -E "express|cors|helmet|jest"

# 2. Verify structure
find . -type d -name "middleware" -o -name "routes" -o -name "utils" -o -name "unit" -o -name "integration"

# 3. Test server startup
npm run dev
# In another terminal:
curl -I http://localhost:3000/

# 4. Check scripts
npm run | grep -E "start|dev|test|lint"

# 5. Verify file contents
grep -q "helmet" src/app.js && echo "✓ Helmet configured"
grep -q "SIGTERM" src/server.js && echo "✓ Graceful shutdown configured"
grep -q "PORT" .env && echo "✓ Environment variables set"
```

## Definition of Done
This task is considered complete when:
1. All acceptance criteria checkboxes above can be marked as complete
2. Server starts successfully with `npm run dev`
3. No critical errors are present
4. Project structure matches the specification exactly
5. All configuration files contain the required content
6. Dependencies are installed with exact versions specified