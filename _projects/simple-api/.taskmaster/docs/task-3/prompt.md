# Task 3: Implement Express Application and Middleware - Autonomous Prompt

You are tasked with implementing the Express application infrastructure for a Simple Todo REST API. This involves setting up the core application with proper middleware, error handling, and validation utilities.

## Your Mission

Create a robust Express.js application structure with comprehensive middleware for handling HTTP requests, validating input, and managing errors. The application should be production-ready with proper error handling and validation.

## Required Actions

1. **Create Express Application (`src/app.js`)**
   - Initialize Express application
   - Configure body parsing middleware for JSON
   - Add request logging for development mode
   - Implement 404 handler for undefined routes
   - Create global error handling middleware with:
     - Validation error handling
     - Database error handling
     - Environment-specific error messages
     - Proper HTTP status codes

2. **Implement Validation Middleware (`src/middleware/validation.js`)**
   
   Create validation rules for:
   - **Create Todo**: Validate title (required, string, 1-200 chars) and description (optional, string, max 1000 chars)
   - **Update Todo**: Validate ID (integer), optional title/description/completed fields
   - **Get One Todo**: Validate ID parameter (integer)
   - **Delete Todo**: Validate ID parameter (integer)
   - **List Todos**: Validate query params (completed: boolean, limit: 1-100, offset: >= 0)
   
   Include a validation error handler that formats errors consistently.

3. **Create Common Middleware (`src/middleware/common.js`)**
   - CORS headers (prepare for future use)
   - Request ID generation for tracking
   - Response time logging
   - Any other utility middleware

4. **Implement Server Entry Point (`server.js`)**
   - Load environment variables with dotenv
   - Start Express server on configured port
   - Log server status and API documentation URL
   - Implement graceful shutdown handlers for SIGTERM and SIGINT

5. **Configure Middleware Order**
   
   Ensure proper middleware ordering:
   1. Body parsers
   2. Common middleware (logging, etc.)
   3. Routes (placeholder for Task 5)
   4. 404 handler
   5. Error handler (must be last)

## Implementation Requirements

### Error Response Format
```json
{
  "error": "Error Type",
  "message": "Human-readable message",
  "details": [] // For validation errors
}
```

### Validation Error Format
```json
{
  "error": "Validation Error",
  "details": [
    {
      "field": "title",
      "message": "Title is required",
      "value": null
    }
  ]
}
```

### Environment Variables
- `PORT`: Server port (default: 3000)
- `NODE_ENV`: Environment (development/production)

## Success Verification

Ensure all of the following work correctly:

- [ ] Server starts without errors: `npm run dev`
- [ ] Returns 404 for undefined routes
- [ ] Handles JSON request bodies
- [ ] Validation middleware rejects invalid input
- [ ] Error handler returns appropriate status codes
- [ ] Development mode shows detailed errors
- [ ] Production mode hides sensitive error details
- [ ] Graceful shutdown works with Ctrl+C

## Important Notes

- Do NOT implement routes yet (that's Task 5)
- Do NOT implement controllers yet (that's Task 4)
- Focus only on the application infrastructure
- Use express-validator for validation rules
- Maintain consistent error response format
- Keep middleware functions small and focused
- Test each middleware component independently

## Context

This task builds the web server foundation that will:
- Handle all HTTP requests and responses
- Validate input before it reaches controllers
- Provide consistent error handling
- Support development and production environments

The validation rules defined here will ensure data integrity throughout the application. The error handling will provide clear feedback to API consumers while maintaining security in production.

## Testing Hints

You can test your implementation with:
```bash
# Start server
npm run dev

# Test 404
curl http://localhost:3000/not-found

# Test server is running
curl http://localhost:3000/api/health
# (This will 404 until routes are added)
```

Once complete, the Express application will be ready to receive routes and controllers in subsequent tasks.