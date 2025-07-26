# Autonomous Agent Prompt: Implement Health Check Endpoint

You are tasked with implementing a health check endpoint for the Express.js API that provides service status monitoring capabilities.

## Your Mission
Add a health check endpoint that returns the current service status and timestamp for monitoring and automated health verification.

## Prerequisites
- Task 2 completed (Express server with basic structure)
- Server file `src/index.js` exists
- Root endpoint already implemented

## Required Actions

### 1. Locate Correct Position
Open `src/index.js` and add the health endpoint:
- After the root endpoint (`/`)
- Before any error handlers
- Maintain logical route ordering

### 2. Implement Health Endpoint
Add the following code:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### 3. Verify Implementation Details
Ensure your endpoint:
- Uses GET method for `/health` path
- Returns exactly two fields: `status` and `timestamp`
- Status value is always "healthy"
- Timestamp uses ISO 8601 format
- Returns 200 status code

### 4. Document the Endpoint
Add a comment block explaining the endpoint:

```javascript
/**
 * Health check endpoint
 * Returns service status and current timestamp
 * Used for monitoring and automated health checks
 */
```

## Validation Tests

### Test 1: Basic Health Check
```bash
curl http://localhost:3000/health
```
**Expected Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-26T10:30:45.123Z"
}
```

### Test 2: Status Code Verification
```bash
curl -i http://localhost:3000/health
```
**Expected:**
- Status: 200 OK
- Content-Type: application/json

### Test 3: Timestamp Format Validation
```bash
# Check timestamp is valid ISO format
curl -s http://localhost:3000/health | jq -r .timestamp | date -d @$(date +%s) 
```
**Expected:** No date parsing errors

### Test 4: Response Time Check
```bash
time curl http://localhost:3000/health
```
**Expected:** Response time < 50ms

### Test 5: Repeated Calls
```bash
# Make 3 calls and check timestamps differ
for i in {1..3}; do
  curl -s http://localhost:3000/health | jq -r .timestamp
  sleep 1
done
```
**Expected:** Three different timestamps, 1 second apart

### Test 6: JSON Structure Validation
```bash
curl -s http://localhost:3000/health | jq 'keys'
```
**Expected Output:**
```json
["status", "timestamp"]
```

## Common Implementation Errors

### 1. Wrong Status Value
❌ `status: 'ok'`
❌ `status: 'healthy!'`
✅ `status: 'healthy'`

### 2. Wrong Timestamp Format
❌ `timestamp: new Date()`
❌ `timestamp: Date.now()`
✅ `timestamp: new Date().toISOString()`

### 3. Extra Fields
❌ Adding `version`, `uptime`, etc.
✅ Only `status` and `timestamp`

### 4. Wrong Path
❌ `/healthcheck`
❌ `/status`
✅ `/health`

## Testing Script
Create a test script to validate all requirements:

```bash
#!/bin/bash
echo "Testing health endpoint..."

# Test 1: Check endpoint exists
response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health)
if [ $response -eq 200 ]; then
    echo "✓ Health endpoint returns 200"
else
    echo "✗ Health endpoint failed (status: $response)"
fi

# Test 2: Check JSON structure
curl -s http://localhost:3000/health | jq -e '.status == "healthy" and .timestamp' > /dev/null
if [ $? -eq 0 ]; then
    echo "✓ JSON structure is correct"
else
    echo "✗ JSON structure is incorrect"
fi

# Test 3: Validate timestamp
timestamp=$(curl -s http://localhost:3000/health | jq -r .timestamp)
date -d "$timestamp" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Timestamp is valid ISO format"
else
    echo "✗ Timestamp format is invalid"
fi
```

## Success Criteria
- Endpoint accessible at `/health`
- Returns exactly: `{"status":"healthy","timestamp":"[ISO-DATE]"}`
- Status code is 200
- Timestamp updates on each request
- Response time under 50ms
- No authentication required

## Production Considerations
For future production use, health endpoints typically:
- Check database connections
- Verify external service availability
- Monitor resource usage
- Return 503 when unhealthy

However, for this task, keep it simple with just status and timestamp.

Complete the implementation and run all validation tests before proceeding.