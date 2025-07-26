# Autonomous Agent Prompt: Implement Health Check Endpoint

You are an autonomous agent tasked with implementing a health check endpoint for the Hello World API. This endpoint will provide service status information and is crucial for monitoring and orchestration.

## Prerequisites

Verify before starting:
- Express.js server is running (from Task 2)
- Hello endpoint is implemented at `/` (from Task 3)
- Server file structure is intact

## Task Requirements

### 1. Add Health Check Route

Add the following route to `src/index.js` after the hello endpoint:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### 2. Proper Route Placement

Ensure correct route order in the file:
1. Express setup and constants
2. Logging middleware
3. Hello endpoint (GET /)
4. **Health check endpoint (GET /health)** ← Add here
5. 404 handler (must remain last)

### 3. Implementation Details

The health endpoint must:
- Respond to GET requests at `/health`
- Return HTTP status 200
- Return JSON with two fields:
  - `status`: Always "healthy"
  - `timestamp`: Current time in ISO format
- Generate fresh timestamp for each request

### 4. Complete Code Context

After implementation, your routes section should look like:

```javascript
// Hello endpoint
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

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

## Validation Steps

### Step 1: Basic Functionality Test
```bash
curl http://localhost:3000/health
```
**Expected Output (timestamp will vary):**
```json
{"status":"healthy","timestamp":"2024-01-15T14:32:17.845Z"}
```

### Step 2: Verify Timestamp Format
The timestamp should:
- Be in ISO 8601 format
- Include date (YYYY-MM-DD)
- Include time with milliseconds
- End with 'Z' (UTC indicator)

Example: `2024-01-15T14:32:17.845Z`

### Step 3: Test Dynamic Timestamps
```bash
# Make two requests and compare timestamps
curl http://localhost:3000/health
sleep 2
curl http://localhost:3000/health
```
**Verify:** Timestamps are different

### Step 4: Check HTTP Status
```bash
curl -I http://localhost:3000/health
```
**Verify:** `HTTP/1.1 200 OK`

### Step 5: Test Request Logging
Check server console for:
```
2024-XX-XX...Z - GET /health
```

### Step 6: Verify Other Endpoints Still Work
```bash
# Test hello endpoint
curl http://localhost:3000/
# Should return: {"message":"Hello, World!"}

# Test 404 handler
curl http://localhost:3000/invalid
# Should return: {"error":"Not found"}
```

## Common Mistakes to Avoid

1. **Static Timestamp**: Don't create timestamp once and reuse
   ```javascript
   // WRONG - timestamp never changes
   const timestamp = new Date().toISOString();
   app.get('/health', (req, res) => {
     res.json({ status: 'healthy', timestamp });
   });
   ```

2. **Wrong Date Format**: Use `.toISOString()`, not custom formatting
   ```javascript
   // WRONG - not ISO format
   timestamp: new Date().toString()
   ```

3. **Missing Status Code**: Explicitly set 200
   ```javascript
   // CORRECT
   res.status(200).json({...})
   ```

4. **Wrong Status Value**: Must be exactly "healthy"
   ```javascript
   // WRONG
   status: 'ok'  // Should be 'healthy'
   ```

## Testing Script

Create `quick-health-test.js`:
```javascript
const http = require('http');

// Test health endpoint
http.get('http://localhost:3000/health', (res) => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    const response = JSON.parse(data);
    console.log('Health Check Response:', response);
    console.log('Status healthy?', response.status === 'healthy');
    console.log('Has timestamp?', !!response.timestamp);
    console.log('Valid ISO date?', !isNaN(Date.parse(response.timestamp)));
  });
});
```

## Success Indicators

The task is complete when:
- `/health` returns `{"status":"healthy","timestamp":"..."}`
- Timestamp is in ISO 8601 format
- Timestamp changes with each request
- HTTP status is 200
- All existing endpoints still work
- Request logging captures health checks

## Final Verification

Run this command to verify implementation:
```bash
# Should show healthy status with timestamp
curl -s http://localhost:3000/health | grep -q "healthy" && echo "✓ Health endpoint working" || echo "✗ Health endpoint failed"
```

## Why This Matters

Health endpoints are used by:
- **Load Balancers**: To route traffic only to healthy instances
- **Monitoring Systems**: To track service availability
- **Container Orchestrators**: For liveness/readiness probes
- **CI/CD Pipelines**: To verify successful deployments

Your implementation provides the foundation for production-grade service monitoring.