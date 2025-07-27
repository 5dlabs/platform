# Autonomous AI Agent Prompt: Implement Health Check Endpoint

## Task Overview
You need to add a health check endpoint (GET /health) to the Express.js server. This endpoint should return a JSON response with the service status and current timestamp.

## Detailed Instructions

### Step 1: Locate the Correct Position
1. Open the existing `src/index.js` file
2. Find where the root endpoint (GET /) is defined
3. Add the health check endpoint after the root endpoint

### Step 2: Add the Health Check Endpoint
Add the following code after the root endpoint:
```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### Step 3: Verify Route Order
Ensure the routes are in this order:
1. Root endpoint (GET /)
2. Health check endpoint (GET /health)
3. Any error handling middleware

### Complete Integration Example
```javascript
// ... existing middleware ...

// Routes
// Root endpoint - Returns a welcome message
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

// ... error handling and server startup ...
```

## Expected Outcomes

### API Response
When accessing GET http://localhost:3000/health:
- **Status Code**: 200
- **Headers**: Content-Type: application/json
- **Body Example**: 
  ```json
  {
    "status": "healthy",
    "timestamp": "2024-01-20T15:30:45.123Z"
  }
  ```

### Key Requirements
1. Status must always be "healthy" (as per requirements)
2. Timestamp must be ISO 8601 format
3. Timestamp should be current (generated on each request)

## Validation Steps

1. **Syntax Check**
   ```bash
   node -c src/index.js
   ```

2. **Start Server**
   ```bash
   npm start
   ```

3. **Test Health Endpoint**
   ```bash
   curl -i http://localhost:3000/health
   ```
   
   Expected output:
   ```
   HTTP/1.1 200 OK
   Content-Type: application/json; charset=utf-8
   
   {"status":"healthy","timestamp":"2024-01-20T15:30:45.123Z"}
   ```

4. **Verify Timestamp Changes**
   ```bash
   # Run twice with a delay
   curl -s http://localhost:3000/health; echo
   sleep 2
   curl -s http://localhost:3000/health; echo
   # Timestamps should be different
   ```

5. **Validate JSON Structure**
   ```bash
   curl -s http://localhost:3000/health | \
   node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8'));
   console.log('Has status:', 'status' in d);
   console.log('Has timestamp:', 'timestamp' in d);
   console.log('Status is healthy:', d.status === 'healthy');"
   ```

## Common Issues and Solutions

### Issue: Endpoint returns 404
**Solution**: 
- Ensure route is defined before app.listen()
- Check for typos in the path '/health'
- Restart the server after changes

### Issue: Invalid timestamp format
**Solution**: Use `new Date().toISOString()` exactly as shown

### Issue: Missing fields in response
**Solution**: Ensure both 'status' and 'timestamp' are included

### Issue: Static timestamp
**Solution**: Generate new Date() inside the route handler, not outside

## Important Notes

- **Do NOT** hardcode the timestamp - it must be generated fresh
- **Do NOT** use any format other than ISO 8601 for the timestamp
- **Do NOT** add extra fields beyond status and timestamp (per requirements)
- The status should always be 'healthy' for this simple implementation
- The endpoint should be lightweight and fast
- This endpoint is typically used by:
  - Load balancers for health probes
  - Monitoring systems
  - Container orchestration platforms

## Testing the Implementation

After adding the endpoint:

1. **Check both endpoints work**:
   ```bash
   curl http://localhost:3000/      # Should return Hello, World!
   curl http://localhost:3000/health # Should return health status
   ```

2. **Verify response format**:
   - Must be valid JSON
   - Must have exactly two fields: status and timestamp
   - Status must be "healthy"
   - Timestamp must be ISO format

3. **Check request logging**:
   - Both GET / and GET /health should appear in server logs