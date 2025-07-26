# Task 2: Create Express.js Server

## Overview
This task establishes the core Express.js server infrastructure for the Hello World API. It creates the main application file that serves as the entry point for all HTTP requests and implements basic middleware for request logging.

## Objectives
- Create the main server file (src/index.js)
- Initialize Express.js application
- Implement request logging middleware
- Configure server to listen on port 3000
- Add basic route handling and 404 responses

## Technical Approach

### 1. Express Application Setup
The server uses Express.js as the web framework, providing a minimal and flexible structure for handling HTTP requests. The application is initialized with `express()` and configured to use middleware for cross-cutting concerns.

### 2. Middleware Architecture
Middleware functions execute sequentially for each request:
1. **Request Logger**: Captures timestamp, HTTP method, and URL
2. **Route Handlers**: Process specific endpoint requests
3. **404 Handler**: Catches all unmatched routes

### 3. Port Configuration
The server listens on port 3000 by default, following common development practices. This can be made configurable through environment variables in production.

## Implementation Details

### Step 1: Create Main Server File
```javascript
const express = require('express');
const app = express();
const PORT = 3000;
```

### Step 2: Add Request Logging Middleware
```javascript
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

### Step 3: Add Placeholder Route
```javascript
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### Step 4: Add 404 Handler
```javascript
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

### Step 5: Start Server
```javascript
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

## Dependencies
- **Task 1**: Initialize Node.js Project (must be completed first)
- **Express.js**: Web framework (installed in Task 1)

## Success Criteria
- [ ] src/index.js file exists with complete server code
- [ ] Server starts successfully with `npm start`
- [ ] Console displays startup message
- [ ] All requests are logged with timestamp
- [ ] GET / returns "Server is running"
- [ ] Undefined routes return 404 JSON response

## Testing Strategy

### Manual Testing
1. Start server: `npm start`
2. Verify startup message appears
3. Test root endpoint: `curl http://localhost:3000`
4. Test undefined route: `curl http://localhost:3000/undefined`
5. Verify all requests appear in console logs

### Expected Outputs
- Root endpoint: HTTP 200, "Server is running"
- Undefined routes: HTTP 404, `{"error":"Not found"}`
- Console logs: ISO timestamp, method, and URL for each request

## Related Tasks
- **Task 3**: Implement Root Endpoint - Will replace placeholder route
- **Task 4**: Implement Health Endpoint - Adds new route handler
- **Task 5**: Add Error Handling - Enhances middleware chain
- **Task 6**: Request Logging - Builds on logging middleware

## Notes
- The logging middleware uses `next()` to pass control to subsequent handlers
- ISO 8601 timestamps provide consistent, sortable log entries
- The 404 handler must be placed after all route definitions
- Consider adding body parsing middleware for future POST requests