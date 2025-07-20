# Task 1: Initialize Express Project - Acceptance Criteria

## Overview
This document defines the specific acceptance criteria that must be met for Task 1 to be considered complete. Each criterion includes clear verification steps and expected outcomes.

## Project Structure Criteria

### ✓ Directory Structure
- **Requirement**: All required directories must exist
- **Verification**:
  ```bash
  ls -la src/
  # Expected: config/ middleware/ routes/ models/ utils/ directories
  ls -la
  # Expected: public/ tests/ directories at root level
  ```
- **Expected Result**: All directories are present and properly organized

### ✓ Package Configuration
- **Requirement**: package.json is properly configured
- **Verification**:
  ```bash
  cat package.json | grep -E '"main"|"scripts"|"dependencies"'
  ```
- **Expected Result**:
  - Main entry point is "src/app.js"
  - Scripts section includes start, dev, and test commands
  - All required dependencies are listed

## Dependencies Criteria

### ✓ Production Dependencies
- **Requirement**: All production dependencies installed with correct versions
- **Verification**:
  ```bash
  npm list express dotenv cors morgan helmet
  ```
- **Expected Versions**:
  - express: ^4.19.2
  - dotenv: ^16.4.5
  - cors: ^2.8.5
  - morgan: ^1.10.0
  - helmet: ^7.1.0

### ✓ Development Dependencies
- **Requirement**: Development tools installed
- **Verification**:
  ```bash
  npm list --dev nodemon
  ```
- **Expected Version**: nodemon: ^3.1.0

## Server Functionality Criteria

### ✓ Server Startup
- **Requirement**: Server starts without errors
- **Verification**:
  ```bash
  npm run dev
  ```
- **Expected Result**: 
  - Console shows: "Server running on port 3000 in development mode"
  - No error messages
  - Server remains running

### ✓ Health Check Endpoint
- **Requirement**: Health endpoint responds correctly
- **Verification**:
  ```bash
  curl http://localhost:3000/health
  ```
- **Expected Response**:
  ```json
  {
    "status": "OK",
    "timestamp": "2025-07-20T12:00:00.000Z",
    "environment": "development"
  }
  ```
- **Status Code**: 200

### ✓ 404 Error Handling
- **Requirement**: Non-existent routes return proper error
- **Verification**:
  ```bash
  curl http://localhost:3000/nonexistent
  ```
- **Expected Response**:
  ```json
  {
    "error": {
      "message": "Not Found",
      "status": 404
    }
  }
  ```
- **Status Code**: 404

## Middleware Criteria

### ✓ Security Headers (Helmet)
- **Requirement**: Helmet middleware sets security headers
- **Verification**:
  ```bash
  curl -I http://localhost:3000/health | grep -i "x-"
  ```
- **Expected Headers**: Various X- security headers like X-Content-Type-Options

### ✓ CORS Configuration
- **Requirement**: CORS headers are present
- **Verification**:
  ```bash
  curl -I http://localhost:3000/health | grep -i "access-control"
  ```
- **Expected Result**: Access-Control-Allow-Origin header present

### ✓ Request Logging (Morgan)
- **Requirement**: HTTP requests are logged to console
- **Verification**: Make a request and check server console
- **Expected Result**: Detailed request log in combined format

### ✓ Body Parsing
- **Requirement**: JSON and URL-encoded bodies are parsed
- **Verification**: Will be tested in later tasks with POST requests
- **Expected Result**: express.json() and express.urlencoded() middleware configured

## Environment Configuration Criteria

### ✓ Environment Files
- **Requirement**: Environment configuration files exist
- **Verification**:
  ```bash
  ls -la .env*
  ```
- **Expected Files**:
  - `.env` (not in git)
  - `.env.example` (in git)

### ✓ Environment Variables Loading
- **Requirement**: Variables from .env are accessible
- **Verification**: 
  - Change PORT in .env to 3001
  - Restart server
  - Server should start on port 3001
- **Expected Result**: Server uses PORT from .env file

### ✓ .env.example Content
- **Requirement**: Template includes all required variables
- **Verification**:
  ```bash
  cat .env.example
  ```
- **Expected Content**:
  ```
  PORT=3000
  NODE_ENV=development
  JWT_SECRET=your-secret-key-here
  DATABASE_URL=./database.sqlite
  ```

## Development Setup Criteria

### ✓ npm Scripts
- **Requirement**: All required scripts are configured
- **Verification**:
  ```bash
  npm run start  # Should run node src/app.js
  npm run dev    # Should run nodemon src/app.js
  npm run test   # Should run jest (will fail until tests added)
  ```
- **Expected Result**: Scripts execute correct commands

### ✓ Git Configuration
- **Requirement**: .gitignore properly configured
- **Verification**:
  ```bash
  cat .gitignore | grep -E "node_modules|\.env|\.log"
  ```
- **Expected Result**: Sensitive files and directories are ignored

### ✓ Auto-Restart in Development
- **Requirement**: Nodemon restarts on file changes
- **Verification**:
  1. Run `npm run dev`
  2. Modify src/app.js (add a comment)
  3. Save the file
- **Expected Result**: Server automatically restarts

## Error Handling Criteria

### ✓ Global Error Handler
- **Requirement**: Unhandled errors are caught and formatted
- **Verification**: Temporarily add a route that throws an error
- **Expected Result**: 
  - Error is logged to console
  - Client receives formatted error response
  - Server doesn't crash

### ✓ Graceful Shutdown
- **Requirement**: Server shuts down cleanly on SIGTERM/SIGINT
- **Verification**:
  1. Start server with `npm run dev`
  2. Press Ctrl+C
- **Expected Result**:
  - Console shows: "Received shutdown signal, closing server gracefully..."
  - Console shows: "Server closed"
  - Process exits cleanly

## Documentation Criteria

### ✓ README.md Exists
- **Requirement**: Basic project documentation is present
- **Verification**:
  ```bash
  test -f README.md && echo "README exists"
  ```
- **Expected Result**: README.md file exists with basic project information

## Test Summary Checklist

Run through this checklist to ensure all criteria are met:

- [ ] All directories created (src/*, public/, tests/)
- [ ] package.json properly configured
- [ ] All dependencies installed with correct versions
- [ ] Server starts on configured port
- [ ] Health endpoint returns 200 with proper JSON
- [ ] 404 errors handled correctly
- [ ] Security headers present (Helmet)
- [ ] CORS headers present
- [ ] Request logging active (Morgan)
- [ ] Environment variables load from .env
- [ ] .env.example contains all variables
- [ ] npm scripts work (start, dev, test)
- [ ] .gitignore excludes sensitive files
- [ ] Nodemon auto-restarts on changes
- [ ] Global error handler catches errors
- [ ] Graceful shutdown works with Ctrl+C
- [ ] README.md exists

## Definition of Done

Task 1 is complete when:
1. All acceptance criteria above are met
2. Server runs without errors
3. Code follows project structure conventions
4. No sensitive information is committed to git
5. Development environment is fully functional

## Notes

- If any criterion fails, identify the specific issue and fix it before proceeding
- Some criteria (like body parsing) will be fully tested in subsequent tasks
- Keep the .env file secure and never commit it to version control