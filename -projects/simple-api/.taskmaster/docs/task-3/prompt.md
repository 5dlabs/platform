# Task 3: Implement Express Application and Middleware - Autonomous Prompt

You are an AI agent tasked with setting up the Express application and implementing all necessary middleware for the Simple Todo REST API. Your goal is to create the application layer with proper middleware configuration, error handling, and validation setup.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- Task 1 (Project Setup) has been completed
- Dependencies including Express and express-validator are installed

## Your Mission
Create the Express application with all required middleware including JSON parsing, CORS configuration, request logging, validation middleware, and comprehensive error handling. Set up the application structure to support the routes that will be added in Task 5.

## Required Actions

### 1. Create Main Application File
Create `src/app.js`:
- Initialize Express application
- Configure JSON and URL-encoded body parsing
- Add request logging for development
- Configure CORS headers for cross-origin support
- Set up placeholder for routes
- Implement 404 handler
- Create global error handler

### 2. Implement Validation Middleware
Create `src/middleware/validation.js`:
- Import express-validator functions
- Create validation wrapper to handle errors
- Implement todo validation rules:
  - `create`: title (required, 1-200 chars), description (optional, max 1000)
  - `update`: id param, optional fields with same constraints
  - `getOne`: id parameter validation
  - `delete`: id parameter validation
  - `list`: query param validation (completed, limit, offset)
- Ensure all validations include proper error messages

### 3. Create Error Handler Utilities
Create `src/middleware/errorHandler.js`:
- Define custom error classes:
  - `AppError`: Base error class with status and code
  - `NotFoundError`: For 404 errors
  - `ValidationError`: For validation failures
- Create `asyncHandler` wrapper for async route handlers
- Export all error utilities

### 4. Create Server Entry Point
Create `server.js` in root directory:
- Load environment variables with dotenv
- Import the Express app
- Start server on configured port
- Add graceful shutdown handling
- Log startup information

### 5. Configure Error Response Format
Ensure consistent error responses:
```json
{
  "error": {
    "message": "Human-readable message",
    "code": "ERROR_CODE",
    "details": [...]  // For validation errors
  }
}
```

### 6. Add Health Check Endpoint
In `src/app.js`, add basic health endpoint:
- Route: GET /api/health
- Response: status, timestamp, environment

## Validation Criteria
- Express app starts without errors
- All middleware is properly configured
- CORS headers are set correctly
- Request logging works in development
- Validation middleware validates correctly
- Error handler catches and formats all errors
- 404 responses use consistent format
- Health endpoint returns proper response
- Server handles graceful shutdown

## Important Notes
- Place middleware in correct order (body parsing before routes)
- Error handler must be the last middleware
- Use environment variables for configuration
- Follow the error response format from architecture
- Implement proper async error handling
- Don't implement actual routes (done in Task 5)
- Ensure validation messages are user-friendly

## Testing the Implementation
After implementation, verify:
1. Server starts on configured port
2. Health endpoint responds correctly
3. Invalid JSON returns proper error
4. 404 errors return consistent format
5. CORS headers are present
6. Request logging works in dev mode
7. Validation middleware is exportable
8. Server shuts down gracefully on SIGTERM

## Expected Outcome
A complete Express application setup with:
- Fully configured Express app
- All required middleware in place
- Comprehensive error handling
- Validation middleware ready for routes
- Server entry point configured
- Ready for route implementation in Task 5

Execute all steps systematically and ensure the application starts correctly with all middleware functioning as expected.