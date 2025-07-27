# Autonomous AI Agent Prompt: Implement Root Endpoint

## Task Overview
You need to add a root endpoint (GET /) to the Express.js server that returns a JSON response with a "Hello, World!" message. This endpoint should be added to the existing src/index.js file.

## Detailed Instructions

### Step 1: Locate the Correct Position
1. Open the existing `src/index.js` file
2. Find the section after middleware setup but before error handling
3. This is where routes should be defined

### Step 2: Add the Root Endpoint
Add the following code in the appropriate location:
```javascript
// Routes
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Step 3: Verify Placement
Ensure the route is placed:
- AFTER the request logging middleware
- BEFORE the error handling middleware
- BEFORE the app.listen() call

### Complete Code Context
The route should fit into the file structure like this:
```javascript
// ... existing imports and setup ...

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Routes
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Error handling middleware (if present)
// ... rest of the file ...
```

## Expected Outcomes

### API Response
When accessing GET http://localhost:3000/, the response should be:
- **Status Code**: 200
- **Headers**: Content-Type: application/json
- **Body**: 
  ```json
  {
    "message": "Hello, World!"
  }
  ```

### Server Logs
The request logging middleware should output:
```
2024-01-20T10:30:00.000Z - GET /
```

## Validation Steps

1. **Syntax Check**
   ```bash
   node -c src/index.js
   ```

2. **Start Server**
   ```bash
   npm start
   ```

3. **Test with curl**
   ```bash
   curl -i http://localhost:3000/
   ```
   
   Expected output:
   ```
   HTTP/1.1 200 OK
   Content-Type: application/json; charset=utf-8
   
   {"message":"Hello, World!"}
   ```

4. **Test with JSON parsing**
   ```bash
   curl -s http://localhost:3000/ | node -e "console.log(JSON.parse(require('fs').readFileSync(0, 'utf8')))"
   ```
   
   Expected: `{ message: 'Hello, World!' }`

## Common Issues and Solutions

### Issue: 404 Not Found
**Causes & Solutions**:
- Route not defined: Ensure the app.get() is added
- Route in wrong position: Must be before app.listen()
- Server not restarted: Stop and restart the server

### Issue: Cannot GET /
**Solution**: The route handler might be missing or have syntax errors

### Issue: Response is not JSON
**Solution**: Use res.json() not res.send() or res.write()

### Issue: Wrong status code
**Solution**: Explicitly set res.status(200) before .json()

## Important Notes

- **Do NOT** create a new file - modify the existing src/index.js
- **Do NOT** use res.send() - use res.json() for JSON responses
- **Do NOT** forget the status code - explicitly set to 200
- The route must be defined before any error handling middleware
- The exact message "Hello, World!" (with comma and exclamation) is required
- Express automatically sets Content-Type header for JSON responses

## Testing the Implementation

After adding the endpoint, test it thoroughly:

1. **Basic GET request**: Should return the JSON message
2. **Check status code**: Must be 200
3. **Verify JSON format**: Response must be valid JSON
4. **Check exact message**: Must be "Hello, World!" exactly
5. **Verify logging**: Request should appear in server logs