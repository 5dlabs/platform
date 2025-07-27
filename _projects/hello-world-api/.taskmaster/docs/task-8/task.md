# Task 8: Create Main Server File

## Overview
This task implements the main Express.js server file (src/index.js) that serves as the entry point for the Hello World API. It establishes the basic server setup with request logging middleware and proper error handling for startup.

## Purpose and Objectives
- Create the main Express.js application entry point
- Implement basic request logging middleware
- Configure server to listen on the specified port
- Add environment variable support for port configuration
- Implement error handling for server startup
- Establish foundation for adding API endpoints

## Technical Approach

### Server Architecture
1. **Express App Initialization**: Create and configure Express application instance
2. **Middleware Pipeline**: Set up request logging as first middleware
3. **Port Configuration**: Support both environment variable and default port
4. **Server Startup**: Implement robust server listening with error handling
5. **Logging Strategy**: Use ISO timestamp format for request logs

### Key Technical Decisions
- Use built-in console.log for simple logging (suitable for this minimal API)
- Support PORT environment variable for deployment flexibility
- Implement middleware before routes for proper request processing
- Add graceful error handling for server startup failures
- Keep the implementation minimal per project requirements

## Implementation Details

### Complete src/index.js Implementation
```javascript
const express = require('express');
const app = express();

// Port configuration with environment variable support
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Error handling middleware (for future use)
app.use((err, req, res, next) => {
  console.error(`${new Date().toISOString()} - ERROR:`, err.message);
  res.status(500).json({ error: 'Internal Server Error' });
});

// Start the server with error handling
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
  console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
});

// Handle server startup errors
server.on('error', (error) => {
  if (error.code === 'EADDRINUSE') {
    console.error(`Port ${PORT} is already in use`);
  } else if (error.code === 'EACCES') {
    console.error(`Port ${PORT} requires elevated privileges`);
  } else {
    console.error('Server error:', error);
  }
  process.exit(1);
});

// Graceful shutdown handling
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
    process.exit(0);
  });
});

module.exports = app; // Export for testing purposes
```

### Middleware Pipeline Order
1. Request logging (all requests)
2. Future middleware (body parsing, CORS, helmet)
3. Routes (to be added in subsequent tasks)
4. Error handling middleware (catches all errors)

## Dependencies and Requirements

### Prerequisites
- Completed Task 7: Express.js installed
- Node.js 20.x or higher
- Express.js 4.x available in node_modules

### Runtime Requirements
- Port 3000 available (or custom PORT via environment)
- Write permissions for console output
- Node.js process management capabilities

## Testing Strategy

### Manual Testing Steps
1. **Server Startup Test**
   ```bash
   npm start
   # Expected: "Server running on port 3000"
   ```

2. **Port Environment Variable Test**
   ```bash
   PORT=8080 npm start
   # Expected: "Server running on port 8080"
   ```

3. **Request Logging Test**
   ```bash
   # In another terminal:
   curl http://localhost:3000/
   # Server console should show: "2024-01-20T10:30:00.000Z - GET /"
   ```

4. **Port Conflict Test**
   ```bash
   # Start server twice to test EADDRINUSE handling
   npm start & npm start
   # Second instance should show: "Port 3000 is already in use"
   ```

### Automated Verification
```javascript
// test-server.js
const http = require('http');

const testPort = process.env.PORT || 3000;
const options = {
  hostname: 'localhost',
  port: testPort,
  path: '/',
  method: 'GET'
};

const req = http.request(options, (res) => {
  console.log(`✓ Server responding on port ${testPort}`);
  console.log(`✓ Status code: ${res.statusCode}`);
  process.exit(0);
});

req.on('error', (error) => {
  console.error('✗ Server not responding:', error.message);
  process.exit(1);
});

req.end();
```

### Success Criteria
- ✅ Server starts without errors on port 3000
- ✅ PORT environment variable is respected
- ✅ All requests are logged with timestamp
- ✅ Server handles startup errors gracefully
- ✅ SIGTERM signal triggers graceful shutdown
- ✅ Console shows startup confirmation message

## Related Tasks
- **Previous**: Task 7 - Install Express.js Dependency
- **Next**: Task 9 - Implement Root Endpoint
- **Enables**: All endpoint implementation tasks

## Notes and Considerations
- The logging middleware runs for ALL requests (including 404s)
- Error handling middleware must be defined after all other middleware
- Exporting the app enables future testing capabilities
- SIGTERM handling important for containerized deployments
- Consider using Morgan middleware in production for better logging
- The server.on('error') handler prevents uncaught exceptions
- Environment detection helps with debugging deployment issues