# Autonomous Agent Prompt: Create Main Server File

## Context
You are implementing the core server file for a Hello World API. The Node.js project has been initialized and Express.js with middleware packages have been installed. Now you need to create the main server file that will run the Express application.

## Objective
Create the main server file (src/index.js) with a properly configured Express.js server, including basic middleware for request logging and error handling for server startup.

## Task Requirements

### 1. Create the Main Server File
Create `src/index.js` with the following structure:
- Import Express.js
- Initialize the Express application
- Configure the port with environment variable support
- Export the app instance for testing purposes

### 2. Implement Request Logging Middleware
Add custom middleware that logs all incoming requests with:
- ISO format timestamp
- HTTP method
- Request URL
- Proper next() call to continue processing

### 3. Configure Server Startup
- Use app.listen() to start the server
- Log a confirmation message when server starts
- Store the server instance for error handling

### 4. Add Error Handling
Implement error handling for common startup issues:
- EACCES: Port requires elevated privileges
- EADDRINUSE: Port is already in use
- General error handling with clear messages

## Complete Implementation

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

## Step-by-Step Execution

1. **Navigate to project directory**:
   ```bash
   cd hello-world-api
   ```

2. **Create the server file**:
   ```bash
   # Ensure src directory exists
   mkdir -p src
   # Create index.js with the implementation above
   ```

3. **Test the server**:
   ```bash
   npm start
   ```

4. **Verify functionality**:
   - Check for "Server running on port 3000" message
   - Open browser to http://localhost:3000
   - Verify request logs appear in console

## Validation Criteria

### Success Indicators
- [ ] File `src/index.js` exists with Express server code
- [ ] Server starts without errors using `npm start`
- [ ] "Server running on port X" message appears
- [ ] Request logging works for incoming requests
- [ ] Error handling implemented for startup failures
- [ ] App instance is exported for testing

### Quality Checks
1. **Code Structure**:
   - Clean, readable code with proper formatting
   - Comments where helpful
   - Consistent style

2. **Functionality**:
   - Server starts on port 3000 by default
   - Respects PORT environment variable
   - Logs all incoming requests

3. **Error Handling**:
   - Graceful handling of port conflicts
   - Clear error messages
   - Proper process exit codes

## Testing Scenarios

### Test 1: Basic Server Startup
```bash
npm start
# Expected: "Server running on port 3000"
```

### Test 2: Custom Port
```bash
PORT=4000 npm start
# Expected: "Server running on port 4000"
```

### Test 3: Request Logging
```bash
# Start server, then in another terminal:
curl http://localhost:3000
# Expected: Request log with timestamp in server console
```

### Test 4: Port Already in Use
```bash
# Start server in one terminal
npm start
# Try to start again in another terminal
npm start
# Expected: "Port 3000 is already in use" error
```

## Important Notes

- The middleware must be defined before any routes (though no routes exist yet)
- The timestamp should use ISO format for consistency
- The server instance is stored to attach error handlers
- Module.exports allows the app to be imported for testing
- Error handling provides clear feedback for common issues

## Common Pitfalls to Avoid

1. **Don't forget** to call next() in middleware
2. **Ensure** error handling doesn't crash the process unexpectedly
3. **Remember** to use environment variable for PORT
4. **Avoid** complex logic in the logging middleware

## Expected Console Output

When server starts:
```
Server running on port 3000
```

When requests are made:
```
2024-01-15T10:30:45.123Z - GET /
2024-01-15T10:30:46.456Z - GET /favicon.ico
```

## Tools Required
- File system access to create/write src/index.js
- Command execution for testing with npm start
- Text editing capability for JavaScript code

Proceed with implementing the main server file, ensuring all requirements are met and the server is ready for route implementations in subsequent tasks.