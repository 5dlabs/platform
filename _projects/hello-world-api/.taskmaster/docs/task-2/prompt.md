# Autonomous Task Prompt: Create Express.js Server

You are tasked with creating the main Express.js server file for a Hello World API. This server will form the foundation for all API endpoints and middleware.

## Prerequisites
- Task 1 must be completed (Node.js project initialized with Express installed)
- Verify src directory exists

## Task Requirements

### 1. Create Main Server File
Create `src/index.js` with the complete Express server implementation.

### 2. Complete Server Implementation
Write the following code in `src/index.js`:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Basic route placeholder
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Server setup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

### 3. Implementation Steps
1. Import Express and create app instance
2. Define PORT constant as 3000
3. Add request logging middleware that:
   - Logs ISO timestamp
   - Logs HTTP method
   - Logs request URL
   - Calls next() to continue
4. Add temporary root route that returns "Server is running"
5. Add 404 handler for all unmatched routes
6. Start server on specified port with startup message

## Verification Steps

### 1. Start the Server
```bash
npm start
```
Expected: "Server running on http://localhost:3000"

### 2. Test Root Endpoint
```bash
curl http://localhost:3000
```
Expected:
- Response: "Server is running"
- Console log: timestamp, GET, /

### 3. Test 404 Handler
```bash
curl http://localhost:3000/nonexistent
```
Expected:
- Response: `{"error":"Not found"}`
- Status code: 404
- Console log: timestamp, GET, /nonexistent

### 4. Test Different HTTP Methods
```bash
curl -X POST http://localhost:3000
curl -X PUT http://localhost:3000
```
Expected: Logs show POST and PUT methods

## Important Considerations

### Middleware Order
1. Logging middleware must come first
2. Route handlers in the middle
3. 404 handler must be last

### Error Prevention
- Ensure `next()` is called in logging middleware
- Don't call `next()` in route handlers
- Check PORT is not already in use

### Code Quality
- Use consistent formatting
- Include helpful comments
- Follow Express.js conventions

## Expected Outcome
- Fully functional Express server
- All HTTP requests are logged
- Basic route responds correctly
- 404 errors handled gracefully
- Server runs without errors

## Troubleshooting

### Port Already in Use
```
Error: listen EADDRINUSE: address already in use :::3000
```
Solution: Stop other processes or change PORT

### Module Not Found
```
Error: Cannot find module 'express'
```
Solution: Run `npm install` in project directory

### Syntax Errors
- Check for missing parentheses, brackets
- Verify all strings are properly quoted
- Ensure middleware calls next()

Complete the implementation and verify all tests pass before marking complete.