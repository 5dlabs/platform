# Autonomous AI Agent Prompt: Add Error Handling

## Task Overview
You need to add error handling middleware to the Express.js server. This includes a 500 error handler for server errors and a 404 handler for undefined routes. These handlers must be placed after all route definitions.

## Detailed Instructions

### Step 1: Locate the Correct Position
1. Open `src/index.js`
2. Find where routes are defined (GET / and GET /health)
3. Error handlers must be added AFTER all routes
4. Error handlers must be BEFORE app.listen()

### Step 2: Add Error Handling Middleware
Add the following code after all routes:
```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});
```

### Step 3: Verify Middleware Order
The complete middleware order should be:
1. Request logging middleware
2. Routes (GET /, GET /health)
3. Error handling middleware (4 parameters)
4. 404 handler (2 parameters)
5. Server startup (app.listen)

### Complete Integration Example
```javascript
// ... existing code ...

// Routes
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Start the server
const server = app.listen(PORT, () => {
  // ... existing code ...
});
```

## Expected Outcomes

### 404 Response
For undefined routes like GET /api/users:
- **Status Code**: 404
- **Body**: 
  ```json
  {
    "error": "Not Found"
  }
  ```

### 500 Response
For server errors:
- **Status Code**: 500
- **Body**: 
  ```json
  {
    "error": "Internal Server Error"
  }
  ```
- **Console**: Error message logged

## Validation Steps

1. **Syntax Check**
   ```bash
   node -c src/index.js
   ```

2. **Start Server**
   ```bash
   npm start
   ```

3. **Test 404 Handler**
   ```bash
   # Test undefined route
   curl -i http://localhost:3000/undefined-route
   
   # Expected:
   HTTP/1.1 404 Not Found
   {"error":"Not Found"}
   ```

4. **Test Different Methods**
   ```bash
   # POST to root should be 404
   curl -X POST -i http://localhost:3000/
   
   # PUT to health should be 404
   curl -X PUT -i http://localhost:3000/health
   ```

5. **Test Multiple Undefined Routes**
   ```bash
   curl http://localhost:3000/api
   curl http://localhost:3000/users
   curl http://localhost:3000/test
   # All should return 404
   ```

## Common Issues and Solutions

### Issue: 404 handler not working
**Causes & Solutions**:
- Handler placed before routes: Move it after all routes
- Handler has wrong signature: Use `(req, res)` not `(req, res, next)`
- Server not restarted: Restart after changes

### Issue: Error handler not catching errors
**Solutions**:
- Must have exactly 4 parameters: `(err, req, res, next)`
- Must be before 404 handler
- Errors must be passed with `next(err)`

### Issue: "Cannot set headers after they are sent"
**Solution**: Ensure only one response is sent per request

### Issue: Stack traces visible to client
**Solution**: Only log details to console, not in response

## Important Notes

### Critical Requirements
- **Error handler MUST have 4 parameters**: `(err, req, res, next)`
- **404 handler MUST have 2 parameters**: `(req, res)`
- **Order is critical**: Routes → Error Handler → 404 Handler
- **Do NOT expose stack traces** in responses

### Best Practices
- Log errors with timestamps for debugging
- Keep error messages generic for security
- Consider adding request details to logs
- Test with various undefined routes

## Testing Error Scenarios

### Testing 404 Handler
```bash
# Various 404 scenarios
curl http://localhost:3000/api/v1
curl http://localhost:3000/users/123
curl -X DELETE http://localhost:3000/
curl -X PATCH http://localhost:3000/health
```

### Creating a Test Error Route (Optional)
To test the 500 error handler, you could temporarily add:
```javascript
app.get('/test-error', (req, res, next) => {
  next(new Error('Test error'));
});
```

Then test:
```bash
curl http://localhost:3000/test-error
# Should return 500 with {"error":"Internal Server Error"}
# Should log "Error: Test error" to console
```

### Verifying Complete Setup
1. Existing routes still work (/, /health)
2. Undefined routes return 404
3. Different HTTP methods on defined routes return 404
4. Error details logged but not exposed to client
5. Server remains stable after errors