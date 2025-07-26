# Autonomous Task Prompt: Implement Hello Endpoint

You are tasked with implementing the main "Hello, World!" endpoint for the Express.js API. This endpoint is the primary feature of the API and must return a specific JSON response.

## Prerequisites
- Task 2 must be completed (Express server running)
- Server file exists at src/index.js
- Server has placeholder route that needs replacement

## Task Requirements

### 1. Locate the Placeholder Route
Find and identify the placeholder route in src/index.js:
```javascript
// Basic route placeholder
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### 2. Replace with Hello Endpoint
Replace the placeholder with the proper implementation:
```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### 3. Implementation Requirements
- Must use GET method
- Must be at root path (/)
- Must return JSON response
- Must include status code 200
- Response format: `{"message":"Hello, World!"}`

### 4. Code Placement
Ensure the route is placed:
- After request logging middleware
- Before the 404 handler
- In place of the placeholder route

## Step-by-Step Instructions

1. **Open src/index.js**
2. **Find the placeholder route** (around line 10-13)
3. **Replace the entire route handler** with the new implementation
4. **Save the file**
5. **Restart the server** if it's running

## Verification Steps

### 1. Start/Restart Server
```bash
npm start
```

### 2. Test with curl
```bash
curl -i http://localhost:3000/
```
**Expected Output:**
```
HTTP/1.1 200 OK
X-Powered-By: Express
Content-Type: application/json; charset=utf-8
Content-Length: 25
...

{"message":"Hello, World!"}
```

### 3. Test Response Format
```bash
curl -s http://localhost:3000/ | jq
```
**Expected Output:**
```json
{
  "message": "Hello, World!"
}
```

### 4. Verify Logging
Check console for log entry:
```
2024-01-15T10:30:45.123Z - GET /
```

### 5. Test with Browser
Navigate to http://localhost:3000 and verify JSON is displayed

## Common Mistakes to Avoid

### 1. Wrong Response Method
❌ `res.send({message: 'Hello, World!'})`
✅ `res.json({message: 'Hello, World!'})`

### 2. Missing Status Code
❌ `res.json({message: 'Hello, World!'})`
✅ `res.status(200).json({message: 'Hello, World!'})`

### 3. Wrong Response Format
❌ `res.json('Hello, World!')`
❌ `res.json({msg: 'Hello, World!'})`
✅ `res.json({message: 'Hello, World!'})`

### 4. Keeping Placeholder
Ensure you completely remove the old placeholder route

## Expected Final Code Structure
```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
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

## Success Indicators
- GET / returns exactly `{"message":"Hello, World!"}`
- Status code is 200
- Content-Type is application/json
- No errors in console
- Logging still works

Complete the implementation and verify all tests pass.