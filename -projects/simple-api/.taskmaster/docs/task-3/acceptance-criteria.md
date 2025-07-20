# Task 3: Implement Express Application and Middleware - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 3: Implement Express Application and Middleware. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### 1. Express Application Setup ✓
- [ ] `src/app.js` file exists and exports Express app instance
- [ ] Express app created with `express()`
- [ ] JSON body parser configured with 10mb limit
- [ ] URL-encoded body parser configured with extended: true
- [ ] Trust proxy setting enabled in production
- [ ] App does NOT start the server (that's in server.js)

### 2. Request Processing Middleware ✓
- [ ] **Development Logging**:
  - [ ] Request logging active when NODE_ENV=development
  - [ ] Logs include timestamp, method, and path
  - [ ] No logging in production mode
- [ ] **CORS Configuration**:
  - [ ] Access-Control-Allow-Origin set to '*'
  - [ ] Allowed methods include GET, POST, PUT, DELETE, OPTIONS
  - [ ] Proper headers for preflight OPTIONS requests
  - [ ] OPTIONS requests return 200 immediately
- [ ] **Security Headers**:
  - [ ] X-Content-Type-Options: nosniff
  - [ ] X-Frame-Options: DENY
  - [ ] X-XSS-Protection: 1; mode=block
- [ ] **Response Time Tracking**:
  - [ ] X-Response-Time header added to all responses
  - [ ] Time measured in milliseconds

### 3. Validation Middleware ✓
- [ ] `src/middleware/validation.js` file exists
- [ ] **Todo Validation Rules**:
  - [ ] `create`: title required (1-200 chars), description optional (max 1000)
  - [ ] `update`: id param validated, all fields optional
  - [ ] `getOne`: id param must be positive integer
  - [ ] `delete`: id param must be positive integer
  - [ ] `list`: query params validated with proper types
- [ ] **Validation Features**:
  - [ ] String inputs are trimmed
  - [ ] Data types are converted (toInt, toBoolean)
  - [ ] Error messages are descriptive
- [ ] **Error Handler**:
  - [ ] `handleValidationErrors` middleware implemented
  - [ ] Returns 400 status for validation failures
  - [ ] Error format includes field, message, and value

### 4. Error Handling Middleware ✓
- [ ] `src/middleware/errorHandler.js` file exists
- [ ] **Central Error Handler**:
  - [ ] Handles all unhandled errors
  - [ ] Logs errors in development mode
  - [ ] Returns consistent error format
  - [ ] Hides sensitive details in production
  - [ ] Handles specific error types:
    - [ ] ValidationError → 400
    - [ ] CastError → 400 with "Invalid ID format"
    - [ ] SQLITE_CONSTRAINT → 400
    - [ ] Default → 500
- [ ] **404 Handler**:
  - [ ] Returns proper 404 response
  - [ ] Includes requested path in error
  - [ ] Uses consistent error format
- [ ] **Async Handler**:
  - [ ] Wraps async functions to catch rejections
  - [ ] Passes errors to error middleware

### 5. Server Entry Point ✓
- [ ] `server.js` file exists in project root
- [ ] Loads environment variables with dotenv
- [ ] Imports app from src/app.js
- [ ] Starts server on configured PORT (default 3000)
- [ ] Logs startup message with port and environment
- [ ] **Graceful Shutdown**:
  - [ ] Handles SIGTERM and SIGINT signals
  - [ ] Closes server connections gracefully
  - [ ] Force exits after 10 second timeout
  - [ ] Logs shutdown progress

### 6. Middleware Organization ✓
- [ ] `src/middleware/index.js` exists
- [ ] Exports all middleware functions
- [ ] Groups related middleware together
- [ ] Enables clean imports in other files

## Non-Functional Acceptance Criteria

### Performance
- [ ] Middleware adds < 5ms overhead to requests
- [ ] Body size limits prevent memory exhaustion
- [ ] No blocking operations in middleware

### Security
- [ ] All security headers present on responses
- [ ] Input validation prevents injection attacks
- [ ] Error messages don't leak system information
- [ ] CORS properly configured for API usage

### Code Quality
- [ ] Consistent error response format
- [ ] Middleware functions properly documented
- [ ] Clear separation of concerns
- [ ] Reusable validation rules

### Developer Experience
- [ ] Helpful validation error messages
- [ ] Development logging aids debugging
- [ ] Graceful shutdown prevents data loss
- [ ] Clear middleware execution order

## Test Cases

### Test Case 1: Server Startup
```bash
NODE_ENV=development npm run dev
```
**Expected Result**: 
- Server starts successfully
- Logs "Server running on port 3000 in development mode"
- No errors during startup

### Test Case 2: 404 Handling
```bash
curl http://localhost:3000/api/nonexistent
```
**Expected Result**:
```json
{
  "error": {
    "message": "Resource not found",
    "code": "NOT_FOUND",
    "path": "/api/nonexistent"
  }
}
```

### Test Case 3: CORS Headers
```bash
curl -I http://localhost:3000/api/health
```
**Expected Result**: Response includes:
- Access-Control-Allow-Origin: *
- Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block

### Test Case 4: OPTIONS Request
```bash
curl -X OPTIONS http://localhost:3000/api/todos
```
**Expected Result**: 
- Status code 200
- CORS headers present

### Test Case 5: Request Logging
```bash
# With NODE_ENV=development
curl http://localhost:3000/api/todos
```
**Expected Result**: Console shows:
- Timestamp in ISO format
- "GET /api/todos"

### Test Case 6: Graceful Shutdown
```bash
# Start server, then press Ctrl+C
npm run dev
# Press Ctrl+C
```
**Expected Result**:
- "Received shutdown signal, closing server..."
- "Server closed"
- Process exits cleanly

### Test Case 7: Validation Testing
```javascript
// In a test file or REPL after implementing routes
const { todoValidation, handleValidationErrors } = require('./src/middleware/validation');

// Test validation rules exist
console.log(typeof todoValidation.create); // 'object' (array)
console.log(todoValidation.create.length > 0); // true
```

## Definition of Done
- [ ] All functional acceptance criteria are met
- [ ] All non-functional acceptance criteria are met
- [ ] All test cases pass successfully
- [ ] Server starts without errors
- [ ] Middleware executes in correct order
- [ ] Error handling is comprehensive
- [ ] Security headers are present
- [ ] Code follows project conventions
- [ ] Ready for route implementation

## Notes
- Routes will be added in Task 5, so 404s are expected for now
- The app should be fully functional but have no actual endpoints
- Focus is on the middleware pipeline and application structure
- Validation rules will be used when routes are implemented