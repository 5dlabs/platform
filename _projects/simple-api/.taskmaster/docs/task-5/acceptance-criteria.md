# Acceptance Criteria: Implement Global Error Handling Middleware

## Core Requirements

### 1. File Structure
- [ ] `/src/middleware/errorHandler.js` exists
- [ ] `/src/middleware/requestId.js` exists (NEW)
- [ ] `/src/routes/health.js` exists (NEW)
- [ ] `/src/utils/` directory exists
- [ ] `/src/utils/errors.js` exists
- [ ] All files use ES6 module syntax

### 2. Error Handler Middleware
- [ ] `errorHandler` function with (err, req, res, next) signature
- [ ] `notFoundHandler` function for 404 errors
- [ ] Error logging includes:
  - [ ] ISO timestamp
  - [ ] Error stack trace
  - [ ] Request method and path
  - [ ] Request ID for tracking (NEW)
- [ ] Response format is consistent JSON
- [ ] Error statistics tracking (NEW)

### 3. Request ID Middleware (NEW)
- [ ] `requestIdMiddleware` function generates unique IDs
- [ ] Uses crypto.randomUUID() for ID generation
- [ ] Sets X-Request-ID header on response
- [ ] Stores request ID in req.requestId
- [ ] Applied before all routes

### 4. Custom Error Classes
- [ ] Base `AppError` class extends Error
- [ ] `ValidationError` class (status 400)
- [ ] `NotFoundError` class (status 404)
- [ ] `RateLimitError` class (status 429) (NEW)
- [ ] All error classes have:
  - [ ] Proper name property
  - [ ] Status/statusCode property
  - [ ] Stack trace capture

### 5. Integration Requirements
- [ ] Request ID middleware added first
- [ ] Error middleware added to Express app
- [ ] Health endpoint integrated
- [ ] Middleware order is correct:
  1. Request ID middleware
  2. Health routes
  3. Other routes
  4. 404 handler
  5. Error handler (last)
- [ ] No middleware after error handler

### 6. Error Response Format
- [ ] All errors return JSON with:
  - [ ] `error` field (error type/name)
  - [ ] `message` field (description)
  - [ ] `requestId` field (NEW)
- [ ] Stack trace only in development mode
- [ ] Consistent structure for all error types

### 7. Health Endpoint (NEW)
- [ ] Available at `/api/health`
- [ ] Returns JSON with:
  - [ ] `status` field
  - [ ] `timestamp` field
  - [ ] `uptime` field
  - [ ] `requestId` field
  - [ ] `errors` object with statistics
- [ ] Error statistics include:
  - [ ] Total error count
  - [ ] Errors by type
  - [ ] Last error timestamp
  - [ ] Last reset timestamp

## Test Cases

### Test 1: 404 Not Found
```bash
curl -i http://localhost:3000/api/nonexistent

# Expected:
# HTTP/1.1 404 Not Found
# X-Request-ID: <uuid>
# {
#   "error": "Not Found",
#   "message": "Cannot GET /api/nonexistent",
#   "requestId": "<uuid>"
# }
```

### Test 2: Validation Error
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{}'

# Expected:
# HTTP/1.1 400 Bad Request
# X-Request-ID: <uuid>
# {
#   "error": "ValidationError",
#   "message": "Name and email are required",
#   "requestId": "<uuid>"
# }
```

### Test 3: Development vs Production
```bash
# In development (NODE_ENV=development):
# Errors include "stack" field and "requestId"

# In production (NODE_ENV=production):
# Errors exclude "stack" field but include "requestId"
```

### Test 4: Server Error (500)
```javascript
// Trigger an unhandled error
// Should return:
// HTTP/1.1 500 Internal Server Error
// X-Request-ID: <uuid>
// {
#   "error": "Internal Server Error",
#   "message": "An unexpected error occurred",
#   "requestId": "<uuid>"
# }
```

### Test 5: Error Logging
```bash
# Check console output for error logs
# Should see:
# [2025-01-22T10:00:00.000Z] Error (req_123456789): <stack trace>
```

### Test 6: Health Endpoint (NEW)
```bash
curl http://localhost:3000/api/health

# Expected:
# HTTP/1.1 200 OK
# X-Request-ID: <uuid>
# {
#   "status": "healthy",
#   "timestamp": "2025-01-22T10:00:00.000Z",
#   "uptime": 3600,
#   "requestId": "<uuid>",
#   "errors": {
#     "total": 0,
#     "byType": {},
#     "lastError": null,
#     "lastReset": "2025-01-22T00:00:00.000Z"
#   }
# }
```

### Test 7: Request ID Consistency (NEW)
```bash
# Make request and verify:
# - Response header X-Request-ID matches response body requestId
# - Same request ID appears in server logs
# - Each request gets unique ID
```

### Test 8: Error Statistics (NEW)
```bash
# 1. Trigger several different error types
# 2. Check health endpoint shows updated statistics
# 3. Verify error counts increment correctly
# 4. Confirm lastError timestamp updates
```

### Test 9: Rate Limiting Error (NEW)
```javascript
// If rate limiting is implemented:
// Should return:
// HTTP/1.1 429 Too Many Requests
// {
#   "error": "RateLimitError",
#   "message": "Too many requests",
#   "requestId": "<uuid>"
# }
```

### Test 10: Async Error Handling
```javascript
// Async route errors should be caught
app.get('/test-async', async (req, res, next) => {
  throw new Error('Async error');
});
// Should be handled by global error handler with request ID
```

## Edge Cases
- [ ] Handles errors with no message
- [ ] Handles errors with no status code
- [ ] Handles non-Error objects thrown
- [ ] Handles Promise rejections
- [ ] Handles syntax errors in routes
- [ ] Handles missing request ID gracefully (NEW)
- [ ] Error statistics persist during server lifetime (NEW)

## Performance Requirements
- [ ] Error handling adds < 10ms latency
- [ ] Request ID generation adds < 1ms latency (NEW)
- [ ] No memory leaks from error objects
- [ ] Logging doesn't block response
- [ ] Stack trace generation is efficient
- [ ] Error statistics collection is lightweight (NEW)

## Security Requirements
- [ ] No sensitive data in error messages
- [ ] No internal paths exposed
- [ ] No database details revealed
- [ ] Stack traces hidden in production
- [ ] Error messages don't reveal system info
- [ ] Request IDs don't expose sensitive information (NEW)
- [ ] Health endpoint doesn't leak internal details (NEW)

## Logging Requirements
- [ ] All errors logged to console
- [ ] Timestamp in ISO 8601 format
- [ ] Include request context (method, path)
- [ ] Include request ID in all error logs (NEW)
- [ ] Stack trace for debugging
- [ ] Different log levels if needed
- [ ] Error statistics logged on reset (NEW)

## Definition of Done
- All error types handled consistently
- No unhandled errors crash the server
- Error format is uniform across API
- Logging provides debugging info
- Security best practices followed
- Request ID tracking working correctly (NEW)
- Health monitoring endpoint functional (NEW)
- Error statistics accurate and useful (NEW)
- Ready for production deployment