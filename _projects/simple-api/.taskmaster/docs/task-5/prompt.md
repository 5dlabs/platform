# Autonomous Task Prompt: Implement Global Error Handling Middleware

You are tasked with implementing comprehensive error handling middleware for the Express API to ensure consistent error responses and proper error logging.

## Context
- Express API with user endpoints exists
- Need centralized error handling
- Must handle 404s and unexpected errors
- Consistent error response format required
- **NEW: Must integrate with health check endpoint for error monitoring**
- **NEW: Must add request ID tracking for better debugging**

## Your Mission
Create a robust error handling system with custom error classes and middleware that catches all errors and returns consistent JSON responses.

## Steps to Complete

1. **Create error handler middleware**
   - Build `/src/middleware/errorHandler.js`
   - Implement main error handler function
   - Create 404 not found handler
   - Add error logging with timestamps
   - Include stack traces only in development
   - **NEW: Add request ID generation and tracking**
   - **NEW: Integrate error counter for health monitoring**

2. **Define custom error classes**
   - Create `/src/utils/errors.js`
   - Build base AppError class
   - Create specific error types:
     - ValidationError (400)
     - NotFoundError (404)
     - **NEW: RateLimitError (429)**
   - Ensure proper inheritance and stack traces

3. **Integrate with Express app**
   - Import middleware in main server file
   - Place 404 handler after all routes
   - Place error handler as last middleware
   - Ensure proper middleware ordering
   - **NEW: Add request ID middleware before routes**

4. **Update existing code**
   - Refactor controllers to use error classes
   - Replace res.status().json() with throwing errors
   - Use next(error) for async error handling
   - Maintain backward compatibility
   - **NEW: Update health endpoint to report error stats**

5. **Test error scenarios**
   - Unknown routes return 404
   - Validation errors return 400
   - Server errors return 500
   - All errors have consistent format
   - **NEW: Verify request IDs in responses**
   - **NEW: Test rate limiting scenarios**

## Error Response Standards

### Standard Error Format
```json
{
  "error": "ErrorType",
  "message": "Human-readable error message",
  "requestId": "req_123456789"
}
```

### Development Mode (with stack)
```json
{
  "error": "ErrorType",
  "message": "Human-readable error message",
  "requestId": "req_123456789",
  "stack": "Error stack trace..."
}
```

## NEW: Request ID Integration
- Generate unique request IDs using crypto.randomUUID()
- Add request ID to all error responses
- Include request ID in error logs
- Store request ID in req.requestId for use throughout request lifecycle

## NEW: Health Check Integration
- Track error counts by type
- Expose error statistics at /api/health
- Reset counters daily at midnight
- Include last error timestamp

## Implementation Guidelines
- Use ES6 classes for errors
- Capture stack traces properly
- Log all errors with timestamps
- Hide sensitive information
- Follow Express error handling patterns
- Handle both sync and async errors
- **NEW: Ensure request IDs are consistent across logs and responses**

## Success Criteria
- All errors caught and handled
- No unhandled rejections
- Consistent response format
- Proper HTTP status codes
- Clean error logging
- Development/production mode distinction
- **NEW: Request IDs working correctly**
- **NEW: Health endpoint shows error statistics**

## Security Considerations
- Never expose internal details in production
- Sanitize error messages
- Log security-relevant errors
- Prevent error-based enumeration attacks
- **NEW: Request IDs don't expose sensitive information**