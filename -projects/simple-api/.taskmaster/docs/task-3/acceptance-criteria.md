# Task 3: Implement Express Application and Middleware - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 3: Implement Express Application and Middleware. All criteria must be met for the task to be considered complete.

## Functional Criteria

### 1. Express Application Setup
- [ ] Express app is created and exported from `src/app.js`
- [ ] Application can be started without errors
- [ ] Server listens on configured port
- [ ] Environment variables are loaded correctly

### 2. Middleware Configuration
Body parsing middleware:
- [ ] JSON bodies are parsed correctly
- [ ] URL-encoded bodies are parsed
- [ ] Large payloads are handled appropriately

Request logging:
- [ ] Requests are logged in development mode
- [ ] Logging includes timestamp, method, and path
- [ ] Logging is disabled in production

CORS configuration:
- [ ] CORS headers are set on all responses
- [ ] Allows all origins (Access-Control-Allow-Origin: *)
- [ ] Supports GET, POST, PUT, DELETE methods
- [ ] Handles OPTIONS preflight requests

### 3. Error Handling
Global error handler:
- [ ] Catches all unhandled errors
- [ ] Returns consistent error format
- [ ] Includes appropriate status codes
- [ ] Hides stack traces in production
- [ ] Logs errors to console

404 handler:
- [ ] Catches undefined routes
- [ ] Returns consistent error format
- [ ] Uses 404 status code
- [ ] Includes requested path in error

### 4. Validation Middleware
Todo validation rules implemented for:
- [ ] Create: title required (1-200 chars), description optional (max 1000)
- [ ] Update: id param, optional fields with constraints
- [ ] GetOne: id parameter is integer
- [ ] Delete: id parameter is integer
- [ ] List: query params (completed boolean, limit/offset integers)

Validation features:
- [ ] Error messages are descriptive
- [ ] Multiple errors are reported together
- [ ] String trimming is applied
- [ ] Type checking is enforced

### 5. Error Utilities
Custom error classes:
- [ ] `AppError` base class with status and code
- [ ] `NotFoundError` extends AppError
- [ ] `ValidationError` extends AppError
- [ ] Error classes capture stack traces

Async handler:
- [ ] `asyncHandler` wraps async functions
- [ ] Catches promise rejections
- [ ] Passes errors to error middleware

### 6. Server Entry Point
`server.js` functionality:
- [ ] Loads environment variables
- [ ] Starts server on configured port
- [ ] Logs startup information
- [ ] Handles SIGTERM for graceful shutdown
- [ ] Closes connections properly

### 7. Health Endpoint
- [ ] GET /api/health returns 200 status
- [ ] Response includes status: "ok"
- [ ] Includes current timestamp
- [ ] Shows environment (dev/prod)

## Technical Criteria

### 1. Middleware Order
- [ ] Body parsers are registered first
- [ ] CORS is set before routes
- [ ] Routes are mounted before error handlers
- [ ] Error handler is last middleware

### 2. Error Response Format
All errors follow format:
```json
{
  "error": {
    "message": "string",
    "code": "string",
    "details": [] // optional
  }
}
```

### 3. Environment Configuration
- [ ] PORT defaults to 3000
- [ ] NODE_ENV defaults to 'development'
- [ ] .env file is loaded if present
- [ ] Environment vars override defaults

### 4. Code Organization
- [ ] Middleware is modular and reusable
- [ ] Error classes are properly exported
- [ ] Validation rules are centralized
- [ ] Clear separation of concerns

## Validation Tests

### 1. Server Startup Test
```bash
# Server should start successfully
npm run dev
# Should see: "Server running on port 3000"
```

### 2. Health Check Test
```bash
curl http://localhost:3000/api/health
# Should return: {"status":"ok","timestamp":"...","environment":"development"}
```

### 3. Error Handling Tests
```bash
# 404 error
curl http://localhost:3000/nonexistent
# Should return proper error format with 404 status

# Invalid JSON
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"invalid json}'
# Should return parsing error
```

### 4. CORS Test
```bash
curl -I http://localhost:3000/api/health
# Should include Access-Control-Allow-Origin header

# OPTIONS request
curl -X OPTIONS http://localhost:3000/api/todos
# Should return 200 with CORS headers
```

### 5. Validation Test
```javascript
// Test validation middleware
const validation = require('./src/middleware/validation');
// Should export todo validation rules
console.assert(validation.create !== undefined);
console.assert(validation.update !== undefined);
```

## Edge Cases to Verify

1. **Large Payloads**: Body parser handles size limits
2. **Invalid Content-Type**: Appropriate error returned
3. **Malformed Requests**: Don't crash server
4. **Concurrent Requests**: Server handles multiple requests
5. **Graceful Shutdown**: Server closes cleanly

## Success Indicators

- [ ] Express app starts without errors
- [ ] All middleware functions correctly
- [ ] Error handling is comprehensive
- [ ] Validation is ready for use
- [ ] Server handles edge cases gracefully
- [ ] Health endpoint confirms app status

## Performance Criteria

- [ ] Startup time is reasonable (<2 seconds)
- [ ] Request logging doesn't impact performance
- [ ] Error handling doesn't leak memory
- [ ] Middleware execution is efficient

## Security Criteria

- [ ] Stack traces hidden in production
- [ ] Error messages don't expose internals
- [ ] Body size limits prevent DoS
- [ ] Headers are set securely

## Notes for Reviewers

When reviewing this task:
1. Start the server and verify it runs
2. Test health endpoint functionality
3. Verify error responses match format
4. Check middleware ordering
5. Test CORS with browser/Postman
6. Ensure validation exports are correct

Task is complete when all checkboxes above can be marked as done.