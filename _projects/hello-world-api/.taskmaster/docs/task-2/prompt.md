# Task 2: Implement Core Application Structure - Autonomous Agent Prompt

You are an experienced Node.js developer tasked with implementing the core application structure for the Hello World API. This involves setting up Express.js with proper middleware configuration, error handling, and server initialization.

## Your Mission
Create the foundational Express.js application structure with all necessary middleware, error handling, correlation ID tracking, and graceful shutdown capabilities.

## Detailed Instructions

### 1. Create the Main Express Application (src/app.js)
Create a file at `src/app.js` that sets up the Express application with the following components:

**Required Imports:**
- express
- cors
- helmet
- pino-http for logging
- swagger-jsdoc and swagger-ui-express (import but don't configure yet)

**Implementation Requirements:**
1. Create an Express application instance
2. Implement correlation ID middleware:
   - Check for existing `x-correlation-id` header
   - Generate a new ID if not present (use Math.random().toString(36).substring(2, 15))
   - Attach ID to request object as `req.correlationId`
   - Set response header `x-correlation-id`
3. Configure middleware in this exact order:
   - helmet() for security headers
   - cors() for cross-origin requests
   - express.json() for JSON body parsing
   - pino logger with correlation ID configuration
4. Add route mounting (use `app.use('/', require('./routes'))`)
5. Implement error handling middleware:
   - Log errors using req.log.error
   - Return standardized error response
   - Use error status or default to 500
6. Implement 404 handler for undefined routes
7. Export the app instance

**Pino Logger Configuration:**
```javascript
pino({
  genReqId: (req) => req.correlationId,
  redact: ['req.headers.authorization'],
})
```

### 2. Create the Server Entry Point (src/server.js)
Create a file at `src/server.js` that handles server initialization:

**Implementation Requirements:**
1. Load environment variables using dotenv
2. Import the Express app from './app'
3. Get PORT from environment or default to 3000
4. Start the server and log the port
5. Implement graceful shutdown:
   - Listen for SIGTERM and SIGINT signals
   - Close the server gracefully
   - Set a 10-second timeout for forced shutdown
   - Exit with appropriate codes (0 for success, 1 for forced)
6. Export the server instance for testing

**Graceful Shutdown Pattern:**
```javascript
const shutdown = () => {
  console.log('Shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  
  setTimeout(() => {
    console.error('Forcing shutdown after timeout');
    process.exit(1);
  }, 10000);
};
```

### 3. Create Response Utility Functions (src/utils/response.js)
Create a file at `src/utils/response.js` with standardized response helpers:

**Implementation Requirements:**
1. Export a `success` function:
   - Parameters: message (string), data (any, default null)
   - Returns object with: status: 'success', message, data, timestamp (ISO string)
2. Export an `error` function:
   - Parameters: message (string), status (number, default 400), data (any, default null)
   - Creates Error instance with message
   - Attaches status and data properties to error
   - Returns the error object

### 4. Create Placeholder Routes File
Create an empty routes file at `src/routes/index.js`:
```javascript
const express = require('express');
const router = express.Router();

// Routes will be implemented in Task 3

module.exports = router;
```

## Important Implementation Notes

### Error Response Format
All error responses must follow this structure:
```json
{
  "status": "error",
  "message": "Error message here",
  "data": null,
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

### Success Response Format
All success responses must follow this structure:
```json
{
  "status": "success",
  "message": "Success message here",
  "data": {}, // or null
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

### Middleware Order Matters
The middleware must be configured in the exact order specified:
1. Correlation ID (custom)
2. Helmet
3. CORS
4. Body parser
5. Logger
6. Routes
7. Error handler
8. 404 handler

## Validation Steps
After implementation:
1. Ensure all files are created in the correct locations
2. Verify no syntax errors by running `node src/server.js`
3. Check that the server starts and displays the port message
4. Test graceful shutdown by starting the server and pressing Ctrl+C
5. Verify that making a request returns a 404 with proper format

## Expected Behavior
- Server starts successfully on the configured port
- All requests get a correlation ID in response headers
- Undefined routes return 404 with standard error format
- Errors are logged with correlation IDs
- Server shuts down gracefully on SIGTERM/SIGINT

## Common Pitfalls to Avoid
- Don't forget to require dotenv at the very beginning of server.js
- Ensure error handler has 4 parameters (err, req, res, next)
- Place error handler after routes but before 404 handler
- Don't forget to export the server instance from server.js
- Make sure to create the routes/index.js file even if empty

Complete this task by implementing all three files exactly as specified. The application should be ready to have routes added in the next task.