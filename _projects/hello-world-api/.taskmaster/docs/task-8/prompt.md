# Autonomous AI Agent Prompt: Create Main Server File

## Task Overview
You need to create the main Express.js server file at `src/index.js`. This file will initialize the Express application, configure middleware for request logging, and start the server with proper error handling.

## Detailed Instructions

### Step 1: Create the Main Server File
1. Ensure the `src` directory exists
2. Create a new file at `src/index.js`

### Step 2: Implement Basic Server Code
Create the file with this initial structure:
```javascript
const express = require('express');
const app = express();

// Port configuration with environment variable support
const PORT = process.env.PORT || 3000;
```

### Step 3: Add Request Logging Middleware
Add middleware that logs all incoming requests:
```javascript
// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

### Step 4: Implement Server Startup
Add the server listening code with startup confirmation:
```javascript
// Start the server
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
  console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
});
```

### Step 5: Add Error Handling
Implement error handling for common server startup issues:
```javascript
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
```

### Step 6: Add Graceful Shutdown
Implement SIGTERM handling for clean shutdown:
```javascript
// Graceful shutdown handling
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
    process.exit(0);
  });
});
```

### Step 7: Export the App
Add module export for potential testing:
```javascript
module.exports = app;
```

## Complete File Structure
The final `src/index.js` should contain:
1. Express import and app creation
2. PORT configuration
3. Request logging middleware
4. Server startup with confirmation
5. Error handling for startup failures
6. Graceful shutdown handling
7. Module export

## Expected Outcomes

### Server Behavior
1. Server starts on port 3000 (or PORT env variable)
2. Logs "Server running on port 3000" on startup
3. Logs each request with timestamp, method, and URL
4. Handles port conflicts gracefully
5. Responds to SIGTERM for clean shutdown

### Console Output Examples
```
Server running on port 3000
Environment: development
2024-01-20T10:30:00.000Z - GET /
2024-01-20T10:30:05.000Z - POST /api/data
```

## Validation Steps

1. **File Creation Verification**
   ```bash
   test -f src/index.js && echo "File created" || echo "File missing"
   ```

2. **Syntax Validation**
   ```bash
   node -c src/index.js
   ```

3. **Server Startup Test**
   ```bash
   npm start
   # Should see: "Server running on port 3000"
   ```

4. **Port Conflict Test**
   ```bash
   # In one terminal: npm start
   # In another terminal: npm start
   # Second should show port conflict error
   ```

## Common Issues and Solutions

### Issue: "Cannot find module 'express'"
**Solution**: Ensure you're in the project root and Express is installed

### Issue: "Port 3000 is already in use"
**Solution**: Either stop the other process or use a different PORT:
```bash
PORT=3001 npm start
```

### Issue: "EACCES" error on port
**Solution**: Use a port number above 1024, or run with appropriate permissions

### Issue: Server starts but immediately exits
**Solution**: Ensure no syntax errors and the listen() call is present

## Notes
- Keep all console.log statements for debugging purposes
- The middleware order is important - logging should come first
- Don't add routes yet - they come in later tasks
- The error handling prepares for production deployment
- SIGTERM handling is important for containerized environments