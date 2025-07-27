# Task 8: Create Main Server File

## Overview
**Title**: Create Main Server File  
**Status**: pending  
**Priority**: high  
**Dependencies**: Task 7 (Install Express.js Dependency)  

## Description
Implement the main server file (src/index.js) with basic Express.js setup. This task establishes the core server infrastructure, including Express initialization, basic middleware setup, and server startup configuration.

## Technical Approach

### 1. Server Initialization
- Import Express.js framework
- Create Express application instance
- Configure port with environment variable support

### 2. Middleware Implementation
- Add custom request logging middleware
- Implement proper middleware chain with next() calls
- Ensure middleware executes for all routes

### 3. Server Startup
- Configure server to listen on specified port
- Add startup confirmation logging
- Implement basic error handling for startup failures

## Implementation Details

### Complete Server Implementation
```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Error handling for server startup
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

server.on('error', (error) => {
  if (error.syscall !== 'listen') {
    throw error;
  }

  switch (error.code) {
    case 'EACCES':
      console.error(`Port ${PORT} requires elevated privileges`);
      process.exit(1);
      break;
    case 'EADDRINUSE':
      console.error(`Port ${PORT} is already in use`);
      process.exit(1);
      break;
    default:
      throw error;
  }
});

module.exports = app;
```

### Key Components

#### 1. Express Application Setup
- Import Express module
- Create application instance
- Export app for testing purposes

#### 2. Port Configuration
- Use environment variable PORT if available
- Default to port 3000 for development
- Follows 12-factor app principles

#### 3. Request Logging Middleware
- Logs timestamp in ISO format
- Captures HTTP method and URL
- Calls next() to continue processing

#### 4. Server Startup
- Listens on configured port
- Logs confirmation message
- Returns server instance for error handling

#### 5. Error Handling
- Handles EACCES (permission denied)
- Handles EADDRINUSE (port already in use)
- Provides clear error messages

## Subtasks Breakdown

### 1. Set up Express.js server initialization
- **Status**: pending
- **Dependencies**: None
- **Implementation**: Create index.js, import Express, initialize app
- **Validation**: File exists with proper imports

### 2. Implement request logging middleware
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Add middleware function with logging
- **Key Feature**: ISO timestamp format for logs

### 3. Configure server listening functionality
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Add app.listen() with callback
- **Output**: "Server running on port X" message

### 4. Add environment variable support for PORT
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Use process.env.PORT || 3000
- **Benefit**: Deployment flexibility

### 5. Add error handling for server startup
- **Status**: pending
- **Dependencies**: Subtask 3
- **Implementation**: Add server error event handler
- **Handles**: Permission and port-in-use errors

## Dependencies
- Express.js installed (from Task 7)
- Node.js runtime
- Write access to src directory

## Testing Strategy

### Manual Testing Steps
1. **Start Server**:
   ```bash
   npm start
   ```
   - Verify "Server running on port 3000" message
   - No errors during startup

2. **Test Logging**:
   - Open browser to http://localhost:3000
   - Check console for request log
   - Verify timestamp format and request details

3. **Port Configuration**:
   ```bash
   PORT=4000 npm start
   ```
   - Verify server starts on port 4000

4. **Error Handling**:
   - Start server twice to test EADDRINUSE
   - Try restricted port (like 80) for EACCES

### Expected Console Output
```
Server running on port 3000
2024-01-15T10:30:45.123Z - GET /
2024-01-15T10:30:46.456Z - GET /favicon.ico
```

## Common Issues and Solutions

### Issue: "Cannot find module 'express'"
**Solution**: Ensure npm install was run and Express is in node_modules

### Issue: "Port 3000 is already in use"
**Solution**: Either kill the process using port 3000 or use a different port

### Issue: "EACCES: permission denied"
**Solution**: Use a port number above 1024 or run with appropriate permissions

### Issue: Middleware not logging requests
**Solution**: Ensure middleware is defined before any routes

## Security Considerations

- Request logging should not include sensitive data
- In production, consider using proper logging libraries
- Avoid logging request bodies or headers with sensitive information
- Export app instance allows for proper testing setup

## Performance Considerations

- Console.log is synchronous - consider async logging in production
- Middleware executes for every request - keep it lightweight
- ISO timestamp generation is fast but adds overhead

## Next Steps
After completing this task:
- Implement root endpoint (Task 9)
- Add health check endpoint (Task 10)
- Implement error handling middleware (Task 11)
- Create comprehensive documentation (Task 12)

The server is now ready to handle HTTP requests and can be extended with additional routes and middleware.