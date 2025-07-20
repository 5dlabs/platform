# Task 3: Implement Express Application and Middleware - Autonomous Prompt

You are an AI agent tasked with setting up the Express.js application foundation for a Simple Todo REST API. You will create the core application structure, implement essential middleware, and establish error handling patterns.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: Task 1 (Project Setup) must be completed
- **Framework**: Express.js 4.x with express-validator
- **Working Directory**: Project root (simple-api/)
- **References**:
  - Architecture: .taskmaster/docs/architecture.md (see Middleware section)
  - Requirements: .taskmaster/docs/prd.txt

## Your Mission

Create a robust Express application with comprehensive middleware for validation, error handling, security, and request processing. The application should be production-ready with proper error handling and input validation.

## Detailed Implementation Steps

1. **Create Main Application File** (`src/app.js`)
   - Initialize Express application
   - Configure JSON and URL-encoded body parsers with size limits
   - Add request logging for development environment
   - Implement CORS headers for API access
   - Add security headers (X-Content-Type-Options, X-Frame-Options, X-XSS-Protection)
   - Add response time tracking
   - Export the app (don't start the server here)

2. **Implement Validation Middleware** (`src/middleware/validation.js`)
   - Import express-validator functions
   - Create `todoValidation` object with validation rules:
     - `create`: Validate title (required, 1-200 chars) and description (optional, max 1000 chars)
     - `update`: Validate id param and optional fields
     - `getOne`: Validate id parameter is positive integer
     - `delete`: Validate id parameter
     - `list`: Validate query params (completed boolean, limit 1-100, offset >= 0)
   - Implement `handleValidationErrors` middleware to format validation errors consistently
   - Use `.trim()` on string inputs
   - Convert and validate data types properly

3. **Create Error Handling Middleware** (`src/middleware/errorHandler.js`)
   - Implement central `errorHandler` function:
     - Log errors in development mode
     - Handle different error types (ValidationError, CastError, SQLite constraints)
     - Return consistent error format with message, code, and details
     - Hide sensitive information in production
   - Create `notFoundHandler` for 404 responses
   - Create `asyncHandler` wrapper for async route handlers

4. **Update Application with Middleware** (Update `src/app.js`)
   - Import error handling middleware
   - Add placeholder comments for routes (to be added in Task 5)
   - Register 404 handler after all routes
   - Register global error handler as the last middleware

5. **Create Server Entry Point** (`server.js` in root)
   - Load environment variables with dotenv
   - Import the Express app
   - Start server on configured PORT
   - Implement graceful shutdown handling:
     - Listen for SIGTERM and SIGINT signals
     - Close server connections gracefully
     - Force exit after timeout
   - Log server startup information

6. **Create Middleware Index** (`src/middleware/index.js`)
   - Export all middleware functions for convenient importing
   - Group related middleware together

## Code Structure Requirements

```javascript
// Example error response format
{
  error: {
    message: "Human readable error message",
    code: "ERROR_CODE",
    details: [...] // Optional, only in development
  }
}

// Example validation error detail
{
  field: "title",
  message: "Title is required",
  value: null
}
```

## Implementation Standards

- Use middleware in correct order (body parser → CORS → routes → 404 → error handler)
- All middleware should call `next()` or send a response
- Validation should sanitize inputs (trim whitespace, convert types)
- Error messages should be helpful but not expose system details
- Use environment variables for configuration

## Success Criteria
- ✅ Express app starts without errors
- ✅ JSON bodies are parsed correctly
- ✅ CORS headers allow API access
- ✅ Security headers are set on all responses
- ✅ Validation rules work as specified
- ✅ Error handler formats all errors consistently
- ✅ 404 handler catches undefined routes
- ✅ Server shuts down gracefully on signals
- ✅ Development logging works when NODE_ENV=development

## Testing Your Implementation

Start the server and test:
```bash
# Start server
npm run dev

# Test 404 handling
curl http://localhost:3000/nonexistent

# Test JSON parsing
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Test"}'

# Check security headers
curl -I http://localhost:3000/api/health
```

## Important Notes
- Do NOT implement actual routes yet - that's Task 5
- Do NOT connect to database yet - focus on middleware
- The app should start but return 404 for all routes
- Graceful shutdown is important for production deployments
- Keep sensitive error details out of production responses

## Common Pitfalls to Avoid
1. Registering error handler before routes (must be last)
2. Forgetting to call `next()` in middleware
3. Not handling async errors properly
4. Exposing stack traces in production
5. Missing CORS headers for OPTIONS requests

## Expected File Structure
```
simple-api/
├── src/
│   ├── app.js
│   └── middleware/
│       ├── validation.js
│       ├── errorHandler.js
│       └── index.js
└── server.js
```

Remember: This creates the foundation for request handling. Make it robust and secure as all API operations will flow through these middleware layers.