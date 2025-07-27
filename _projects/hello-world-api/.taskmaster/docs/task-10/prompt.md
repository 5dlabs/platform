# Autonomous Agent Prompt: Implement Health Check Endpoint

## Context
You are continuing the development of a Hello World API. The Express.js server has been created with a root endpoint that returns "Hello, World!". Now you need to implement a health check endpoint that monitoring tools and load balancers can use to verify the service is operational.

## Objective
Add a health check endpoint (GET /health) to the Express application that returns a JSON response with the service status and current timestamp.

## Task Requirements

### 1. Add Health Check Route
Modify `src/index.js` to include a GET route handler for the /health path:
- Use Express's `app.get()` method
- Path should be '/health'
- Place it after the root endpoint

### 2. Implement JSON Response
The route handler should:
- Set HTTP status code to 200
- Return JSON response with structure:
  ```json
  {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:45.123Z"
  }
  ```
- Use ISO 8601 format for timestamp

### 3. Code Placement
Ensure the route is added:
- After the root endpoint (GET /)
- Before the server.listen() call
- With appropriate comments

## Complete Implementation

The route handler to add to src/index.js:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

## Integration Example

The src/index.js structure after adding the health check:

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

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling for server startup
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// ... existing error handling code ...

module.exports = app;
```

## Step-by-Step Execution

1. **Open the server file**:
   - Locate src/index.js
   - Find the position after the root endpoint

2. **Add the health check route**:
   - Insert the GET /health route handler
   - Include the descriptive comment
   - Ensure proper indentation

3. **Save and test**:
   - Save the file
   - Restart the server with `npm start`
   - Test the endpoint

## Validation Criteria

### Success Indicators
- [ ] Route handler added to src/index.js
- [ ] GET /health returns JSON response
- [ ] Response has "status" field with "healthy" value
- [ ] Response has "timestamp" field with ISO date
- [ ] HTTP status code is 200
- [ ] Content-Type header is application/json
- [ ] Request is logged by middleware

### Testing Commands

1. **Basic Test**:
   ```bash
   curl http://localhost:3000/health
   # Expected: {"status":"healthy","timestamp":"2024-01-15T10:30:45.123Z"}
   ```

2. **Detailed Test**:
   ```bash
   curl -v http://localhost:3000/health
   # Should show 200 OK status and JSON content type
   ```

3. **Timestamp Verification**:
   ```bash
   # Run twice to see different timestamps
   curl http://localhost:3000/health
   sleep 2
   curl http://localhost:3000/health
   ```

## Expected Behavior

### Server Console Output
When a request is made to the health endpoint:
```
2024-01-15T10:30:45.123Z - GET /health
```

### Client Response
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:45.123Z"
}
```

### Response Fields
- **status**: Always "healthy" for basic implementation
- **timestamp**: Current server time in ISO 8601 format (YYYY-MM-DDTHH:mm:ss.sssZ)

## Common Mistakes to Avoid

1. **Wrong timestamp format**: Use toISOString(), not toString()
2. **Missing status code**: Explicitly set status(200)
3. **Incorrect placement**: Add after root endpoint, not before
4. **Typos in route path**: Ensure it's exactly '/health'
5. **Wrong response structure**: Match the exact JSON format

## Important Notes

- The timestamp should update with each request
- "healthy" status is hardcoded for this basic implementation
- This endpoint is typically used by:
  - Load balancers for health checks
  - Monitoring systems for uptime tracking
  - Container orchestration for liveness probes
- No authentication is required for health checks

## Future Enhancements (Not Required Now)

For reference, advanced health checks might include:
- Process uptime
- Memory usage
- CPU information
- Database connectivity status
- External service dependencies

But for this task, keep it simple with just status and timestamp.

## Tools Required
- File system access to modify src/index.js
- Text editing capability for JavaScript code
- Command execution for testing

Proceed with implementing the health check endpoint, ensuring it returns the correct JSON response with current timestamp and integrates properly with the existing server code.