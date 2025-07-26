# Task 2: Create Express.js Server

## Overview
This task implements the core Express.js server with request logging middleware and basic route handling, establishing the foundation for the Hello World API endpoints.

## Objectives
- Create the main server file (src/index.js)
- Configure Express.js application
- Implement request logging middleware
- Set up server to listen on port 3000
- Add basic route handling and 404 responses

## Technical Approach

### 1. Express Application Setup
The Express.js framework provides a minimal and flexible Node.js web application framework. We initialize an Express application instance that will handle HTTP requests and responses.

### 2. Middleware Architecture
Express middleware functions execute during the request-response cycle. Our logging middleware:
- Intercepts all incoming requests
- Logs request details with ISO timestamp
- Passes control to the next middleware using `next()`

### 3. Server Configuration
The server is configured to:
- Listen on port 3000 (configurable via environment)
- Log startup confirmation
- Handle graceful shutdown

## Implementation Details

### Step 1: File Creation and Express Import
```javascript
const express = require('express');
const app = express();
const PORT = 3000;
```

### Step 2: Request Logging Middleware
```javascript
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

### Step 3: Basic Route Handler
```javascript
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### Step 4: 404 Error Handler
```javascript
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

### Step 5: Server Listener
```javascript
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

## Dependencies
- Task 1 must be completed (Node.js project initialized)
- Express.js must be installed via npm

## Testing Strategy

### Unit Tests
1. **Server Startup Test**
   - Run `npm start`
   - Verify console output: "Server running on http://localhost:3000"
   - No error messages should appear

2. **Request Logging Test**
   - Make HTTP GET request to http://localhost:3000
   - Verify console shows: `[timestamp] - GET /`

3. **Route Response Test**
   - GET http://localhost:3000
   - Expect: 200 status, "Server is running" response

4. **404 Handler Test**
   - GET http://localhost:3000/nonexistent
   - Expect: 404 status, `{"error": "Not found"}` response

### Integration Tests
- Verify middleware execution order
- Test concurrent request handling
- Validate proper error propagation

## Success Criteria
- ✅ Server starts without errors
- ✅ Listens on port 3000
- ✅ Logs all incoming requests with timestamp
- ✅ Responds to root path with 200 status
- ✅ Returns 404 for undefined routes
- ✅ Console logging is properly formatted

## Error Handling
- **Port in use**: Error message should clearly indicate port conflict
- **Missing Express**: npm install should resolve dependency
- **Syntax errors**: Server should fail to start with clear error message

## Performance Considerations
- Logging middleware adds minimal overhead (<1ms per request)
- Synchronous console.log may block under heavy load
- Consider async logging for production use

## Security Notes
- Request logging should not include sensitive data
- Avoid logging request bodies or headers containing tokens
- 404 responses should not leak internal path information

## Next Steps
After server setup is complete:
- Implement specific API endpoints (Tasks 3 & 4)
- Add error handling middleware
- Enhance logging with request duration