# Task 3: Implement Express Application and Middleware - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 3: Implement Express Application and Middleware. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Express Application Setup ✓

**Given** the project has been initialized (Task 1)
**When** checking src/app.js
**Then** the file must:
- Initialize an Express application
- Configure JSON body parsing middleware
- Configure URL-encoded body parsing
- Export the app instance for use in server.js
- Include proper middleware ordering

**Test**:
```bash
node -e "const app = require('./src/app'); console.log(typeof app.listen === 'function' ? 'Express app OK' : 'Failed')"
```

### 2. Error Handling Middleware ✓

**Given** the Express app is configured
**When** an error occurs
**Then** the error handler must:
- Log errors to console with stack trace
- Return appropriate HTTP status codes
- Show detailed errors in development mode
- Hide sensitive information in production mode
- Handle validation errors with 400 status
- Handle database errors appropriately
- Provide consistent error response format

**Test**: Verify error handler is last middleware in app.js

### 3. Validation Middleware Configuration ✓

**Given** the need for input validation
**When** checking src/middleware/validation.js
**Then** it must include validation rules for:

| Operation | Validations |
|-----------|-------------|
| Create Todo | title: required, string, 1-200 chars; description: optional, string, max 1000 |
| Update Todo | id: integer; title/description/completed: optional with constraints |
| Get One | id: integer parameter |
| Delete | id: integer parameter |
| List | completed: 'true'/'false'; limit: 1-100; offset: >= 0 |

**Test**:
```bash
node -e "const {todoValidation} = require('./src/middleware/validation'); console.log(todoValidation.create ? 'Validation OK' : 'Failed')"
```

### 4. 404 Handler Implementation ✓

**Given** a request to an undefined route
**When** the server processes it
**Then** it must:
- Return 404 status code
- Return JSON error response
- Include the attempted method and path
- Be placed before the error handler

**Test**: Will be fully tested when server is running

### 5. Server Entry Point ✓

**Given** the application needs to start
**When** checking server.js
**Then** it must:
- Load environment variables with dotenv
- Import the Express app
- Start server on configured PORT
- Log startup information
- Handle graceful shutdown on SIGTERM/SIGINT
- Export server instance for testing

**Test**:
```bash
# Should start without errors
npm run dev
# Press Ctrl+C to test graceful shutdown
```

### 6. Common Middleware Utilities ✓

**Given** the need for utility middleware
**When** checking src/middleware/common.js
**Then** it should include:
- CORS header configuration
- Request ID generation
- Response time logging
- Proper export of all middleware functions

**Test**:
```bash
node -e "const {requestId, responseTime} = require('./src/middleware/common'); console.log(typeof requestId === 'function' ? 'Middleware OK' : 'Failed')"
```

### 7. Environment Configuration ✓

**Given** the application uses environment variables
**When** starting the server
**Then** it must:
- Read PORT from environment (default: 3000)
- Read NODE_ENV from environment (default: 'development')
- Use these values appropriately
- Work with the .env file from Task 1

**Test**:
```bash
PORT=4000 npm start
# Should start on port 4000
```

## Test Scenarios

### Scenario 1: Server Startup
```bash
npm run dev
# Expected output:
# Server running in development mode on port 3000
# API documentation available at http://localhost:3000/api-docs
```

### Scenario 2: 404 Response
```bash
# With server running:
curl http://localhost:3000/nonexistent
# Expected: {"error":"Not Found","message":"Cannot GET /nonexistent"}
```

### Scenario 3: JSON Body Parsing
```bash
# With server running:
curl -X POST http://localhost:3000/test \
  -H "Content-Type: application/json" \
  -d '{"test":"data"}'
# Should process JSON (will get 404, but no parse error)
```

### Scenario 4: Graceful Shutdown
```bash
npm run dev
# Press Ctrl+C
# Expected:
# SIGINT signal received: closing HTTP server
# HTTP server closed
```

### Scenario 5: Validation Functions
```javascript
// Test file to verify validation
const { todoValidation } = require('./src/middleware/validation');
console.log('Create validators:', todoValidation.create.length);
console.log('Update validators:', todoValidation.update.length);
console.log('List validators:', todoValidation.list.length);
```

## Integration Tests

### Test 1: Middleware Order
```javascript
// Verify middleware is in correct order in app.js
// 1. Body parsers first
// 2. Routes (when added)
// 3. 404 handler
// 4. Error handler last
```

### Test 2: Error Handler Response
```javascript
// When routes are added, test that errors are properly caught
// Development: Detailed error with stack
// Production: Generic error message
```

## Definition of Done

- [ ] Express app created and properly configured
- [ ] All middleware files exist with required exports
- [ ] Server starts without errors on configured port
- [ ] 404 handler returns proper JSON responses
- [ ] Error handler provides environment-appropriate responses
- [ ] Validation rules defined for all todo operations
- [ ] Graceful shutdown implemented and working
- [ ] Request logging works in development mode
- [ ] All middleware properly ordered in app.js
- [ ] Server.js exports server instance for testing

## Performance Criteria

- Server startup time < 2 seconds
- Graceful shutdown completes < 5 seconds
- Validation processing adds < 5ms overhead
- Error responses returned < 10ms

## Security Criteria

- No stack traces exposed in production
- Input validation prevents injection attacks
- Error messages don't reveal system internals
- All user input is validated before processing

## Notes

- Routes are NOT implemented in this task (Task 5)
- Controllers are NOT implemented in this task (Task 4)
- Focus is on infrastructure and middleware only
- Validation rules will be used by routes in Task 5
- Error handling will be used throughout the application