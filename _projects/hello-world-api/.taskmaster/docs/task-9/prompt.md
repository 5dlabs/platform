# Autonomous Agent Prompt for Task 9: Implement Root Endpoint

## Task Context
You need to add the root endpoint to the Express.js server created in Task 8. This endpoint will respond to GET requests at the root path (/) with a "Hello, World!" message.

## Your Mission
Implement a GET route handler that returns a JSON response with a "Hello, World!" message when users access the root of your API.

## Step-by-Step Instructions

### 1. Open the Main Server File
Navigate to the project and prepare to edit the server file:
```bash
cd hello-world-api
```

### 2. Add Root Endpoint
Edit `src/index.js` to add the root endpoint. The route should be added after middleware but before the server.listen() call:

```javascript
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### 3. Complete Server File Structure
Your `src/index.js` should now look like this:

```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

module.exports = app;
```

## Validation Steps

### 1. Start the Server
```bash
npm start
# Expected: Server running on port 3000
```

### 2. Test with curl
```bash
curl http://localhost:3000/
# Expected output:
# {"message":"Hello, World!"}
```

### 3. Test with Verbose curl
```bash
curl -v http://localhost:3000/
# Expected: 
# HTTP/1.1 200 OK
# Content-Type: application/json
# {"message":"Hello, World!"}
```

### 4. Test with Browser
Open http://localhost:3000/ in a web browser
Expected: JSON response displayed with "Hello, World!" message

### 5. Verify Response Format
```bash
curl -s http://localhost:3000/ | python3 -m json.tool
# Expected formatted output:
# {
#     "message": "Hello, World!"
# }
```

### 6. Test HTTP Status Code
```bash
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/
# Expected output: 200
```

## Expected Result
- Root endpoint handler added to server file
- GET / returns JSON with "Hello, World!" message
- HTTP status code is 200
- Content-Type header is application/json
- Server logs show incoming GET / requests

## Important Notes
- Place the route after middleware to ensure logging works
- Place the route before app.listen() to ensure it's registered
- Use res.status(200) to explicitly set the status code
- Use res.json() for proper JSON response handling
- The message must be exactly "Hello, World!" (with exclamation mark)

## Common Issues to Avoid
1. Placing route after app.listen() - route won't be registered
2. Using res.send() instead of res.json() - improper content type
3. Forgetting the exclamation mark in "Hello, World!"
4. Not setting status code explicitly
5. Route conflicts with other paths