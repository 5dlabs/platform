# Autonomous Agent Prompt for Task 8: Create Main Server File

## Task Context
You need to create the main Express.js server file that will serve as the entry point for the Hello World API. Express.js has already been installed in the previous task.

## Your Mission
Implement a basic Express.js server in `src/index.js` with request logging middleware and proper startup configuration.

## Step-by-Step Instructions

### 1. Navigate to Project
```bash
cd hello-world-api
```

### 2. Create Main Server File
Create `src/index.js` with the following implementation:

```javascript
const express = require('express');
const morgan = require('morgan');
const config = require('./config/express');

// Create Express app
const app = express();

// Environment-based port configuration
const PORT = process.env.PORT || 3000;

// Use Morgan for request logging in development
if (config.env !== 'production') {
  app.use(morgan(config.morganFormat));
}

// Custom request logging middleware (as backup or for custom format)
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Basic error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(err.stack);
  res.status(500).json({
    error: 'Internal Server Error',
    message: config.env === 'development' ? err.message : undefined
  });
});

// Start server with error handling
const server = app.listen(PORT, () => {
  console.log(`✓ Server running on port ${PORT}`);
  console.log(`✓ Environment: ${config.env}`);
  console.log(`✓ URL: http://localhost:${PORT}`);
}).on('error', (err) => {
  console.error('✗ Server failed to start');
  console.error(`✗ Error: ${err.message}`);
  
  if (err.code === 'EADDRINUSE') {
    console.error(`✗ Port ${PORT} is already in use`);
  }
  
  process.exit(1);
});

// Graceful shutdown handling
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

module.exports = app;
```

### 3. Alternative Minimal Implementation
If you prefer the exact implementation from the task details:

```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
```

## Validation Steps

### 1. Test Server Startup
```bash
npm start
# Expected output:
# Server running on port 3000
# OR (if using enhanced version):
# ✓ Server running on port 3000
# ✓ Environment: development
# ✓ URL: http://localhost:3000
```

### 2. Test Port Already in Use
```bash
# In one terminal:
npm start

# In another terminal:
npm start
# Expected: Error message about port already in use
```

### 3. Test Request Logging
```bash
# Start server
npm start

# In another terminal, make a request:
curl http://localhost:3000
# Expected in server console: timestamp - GET /
```

### 4. Test Environment Variable
```bash
PORT=4000 npm start
# Expected: Server running on port 4000
```

### 5. Test Graceful Shutdown
```bash
# Start server
npm start

# Press Ctrl+C
# Expected: Server shuts down cleanly
```

## Expected Result
- Server file created at `src/index.js`
- Server starts successfully on port 3000
- All requests are logged to console
- Environment variable PORT is respected
- Error handling for common startup issues
- Ready for endpoint implementation

## Important Notes
- Ensure proper middleware ordering (logging before routes)
- Always call `next()` in middleware to continue the request pipeline
- Export the app for potential testing purposes
- Consider using the configuration from Task 7
- Handle common errors like port already in use