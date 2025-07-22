# Acceptance Criteria: Implement Global Error Handling Middleware

## Core Requirements

### 1. File Structure
- [ ] `/src/middleware/errorHandler.js` exists
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
- [ ] Response format is consistent JSON

### 3. Custom Error Classes
- [ ] Base `AppError` class extends Error
- [ ] `ValidationError` class (status 400)
- [ ] `NotFoundError` class (status 404)
- [ ] All error classes have:
  - [ ] Proper name property
  - [ ] Status/statusCode property
  - [ ] Stack trace capture

### 4. Integration Requirements
- [ ] Error middleware added to Express app
- [ ] Middleware order is correct:
  1. Routes
  2. 404 handler
  3. Error handler (last)
- [ ] No middleware after error handler

### 5. Error Response Format
- [ ] All errors return JSON with:
  - [ ] `error` field (error type/name)
  - [ ] `message` field (description)
- [ ] Stack trace only in development mode
- [ ] Consistent structure for all error types

## Test Cases

### Test 1: 404 Not Found
```bash
curl -i http://localhost:3000/api/nonexistent

# Expected:
# HTTP/1.1 404 Not Found
# {
#   "error": "Not Found",
#   "message": "Cannot GET /api/nonexistent"
# }
```

### Test 2: Validation Error
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{}'

# Expected:
# HTTP/1.1 400 Bad Request
# {
#   "error": "ValidationError",
#   "message": "Name and email are required"
# }
```

### Test 3: Development vs Production
```bash
# In development (NODE_ENV=development):
# Errors include "stack" field

# In production (NODE_ENV=production):
# Errors exclude "stack" field
```

### Test 4: Server Error (500)
```javascript
// Trigger an unhandled error
// Should return:
// HTTP/1.1 500 Internal Server Error
// {
//   "error": "Internal Server Error",
//   "message": "An unexpected error occurred"
// }
```

### Test 5: Error Logging
```bash
# Check console output for error logs
# Should see:
# [2025-01-22T10:00:00.000Z] Error: <stack trace>
```

### Test 6: Async Error Handling
```javascript
// Async route errors should be caught
app.get('/test-async', async (req, res, next) => {
  throw new Error('Async error');
});
// Should be handled by global error handler
```

## Edge Cases
- [ ] Handles errors with no message
- [ ] Handles errors with no status code
- [ ] Handles non-Error objects thrown
- [ ] Handles Promise rejections
- [ ] Handles syntax errors in routes

## Performance Requirements
- [ ] Error handling adds < 10ms latency
- [ ] No memory leaks from error objects
- [ ] Logging doesn't block response
- [ ] Stack trace generation is efficient

## Security Requirements
- [ ] No sensitive data in error messages
- [ ] No internal paths exposed
- [ ] No database details revealed
- [ ] Stack traces hidden in production
- [ ] Error messages don't reveal system info

## Logging Requirements
- [ ] All errors logged to console
- [ ] Timestamp in ISO 8601 format
- [ ] Include request context (method, path)
- [ ] Stack trace for debugging
- [ ] Different log levels if needed

## Definition of Done
- All error types handled consistently
- No unhandled errors crash the server
- Error format is uniform across API
- Logging provides debugging info
- Security best practices followed
- Ready for production deployment