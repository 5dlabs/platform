# Acceptance Criteria for Task 1: Setup Basic Express Server

## Overview

This document defines the specific acceptance criteria that must be met for Task 1 to be considered complete and successful.

## Functional Requirements

### 1. Project Initialization
- [ ] `package.json` file exists in the project root
- [ ] `package.json` contains valid JSON structure
- [ ] `package.json` includes Express.js in dependencies
- [ ] `package.json` includes a "start" script that runs the server

### 2. Server Implementation
- [ ] `server.js` file exists in the project root
- [ ] Server listens on port 3000 (or PORT environment variable)
- [ ] Server starts without errors when running `npm start`
- [ ] Console logs display startup messages with port information

### 3. Health Endpoint
- [ ] GET `/health` endpoint exists and is accessible
- [ ] Health endpoint returns HTTP status code 200
- [ ] Health endpoint returns valid JSON response
- [ ] JSON response includes required fields:
  - `status` (string): "healthy"
  - `timestamp` (string): ISO timestamp
  - `message` (string): descriptive message

### 4. Response Format
The `/health` endpoint must return JSON in this exact format:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:45.123Z",
  "message": "Express server is running!"
}
```

## Technical Requirements

### 1. Dependencies
- [ ] Express.js is properly installed via npm
- [ ] `node_modules/` directory exists
- [ ] `package-lock.json` file is generated

### 2. Code Quality
- [ ] Server code uses proper Express.js patterns
- [ ] Code includes appropriate error handling
- [ ] Console logging is implemented for debugging
- [ ] Code is readable and well-structured

### 3. Configuration
- [ ] Server port is configurable via PORT environment variable
- [ ] Default port is 3000 when PORT is not set
- [ ] Server gracefully handles startup process

## Testing Criteria

### 1. Manual Testing
- [ ] `npm start` command executes successfully
- [ ] Server starts and displays startup logs
- [ ] `curl http://localhost:3000/health` returns expected response
- [ ] Server can be stopped gracefully (Ctrl+C)

### 2. Automated Verification
- [ ] HTTP GET request to `/health` returns status 200
- [ ] Response Content-Type is `application/json`
- [ ] Response body contains all required JSON fields
- [ ] Timestamp field contains valid ISO date string

## Performance Requirements

### 1. Startup Time
- [ ] Server starts within 5 seconds of running `npm start`
- [ ] No unnecessary delays in the startup process

### 2. Response Time
- [ ] `/health` endpoint responds within 100ms under normal conditions
- [ ] Server handles concurrent requests to `/health` endpoint

## Error Handling

### 1. Graceful Failures
- [ ] Server displays helpful error messages if startup fails
- [ ] Port conflicts are handled with clear error messages
- [ ] Missing dependencies result in clear error messages

### 2. Runtime Stability
- [ ] Server continues running after handling requests
- [ ] No memory leaks during normal operation
- [ ] Proper cleanup when server is terminated

## File Structure Validation

After completion, the project should have this structure:
```
example-express/
├── package.json
├── package-lock.json
├── server.js
├── node_modules/
│   └── express/
└── .taskmaster/
    └── [existing structure]
```

## Success Verification Commands

Run these commands to verify successful completion:

1. **Start the server:**
   ```bash
   npm start
   ```
   Expected: Server starts, displays port message

2. **Test health endpoint:**
   ```bash
   curl -i http://localhost:3000/health
   ```
   Expected: HTTP 200 response with JSON body

3. **Verify JSON structure:**
   ```bash
   curl -s http://localhost:3000/health | jq '.'
   ```
   Expected: Properly formatted JSON with required fields

## Definition of Done

Task 1 is considered complete when:
- ✅ All functional requirements are met
- ✅ All technical requirements are satisfied
- ✅ Manual testing passes successfully
- ✅ Automated verification confirms proper operation
- ✅ File structure matches expected layout
- ✅ Success verification commands execute without errors

The implementation should be production-ready, following Node.js and Express.js best practices for a minimal web server.