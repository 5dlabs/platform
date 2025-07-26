# Autonomous Task Prompt: Implement Health Check Endpoint

You are tasked with implementing a health check endpoint for the Express.js API. This endpoint is crucial for monitoring and service availability checks.

## Prerequisites
- Task 2 must be completed (Express server running)
- src/index.js exists with basic server setup
- Root endpoint (/) already implemented

## Task Requirements

### 1. Add Health Check Route
Add the following route to src/index.js after the root endpoint:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### 2. Route Placement
The health endpoint must be placed:
- After the root endpoint (app.get('/'))
- Before the 404 handler
- In the main route section of the file

### 3. Response Requirements
The endpoint must return:
- JSON object with two properties
- `status`: Always "healthy"
- `timestamp`: Current time in ISO 8601 format
- HTTP status code 200

## Step-by-Step Instructions

1. **Open src/index.js**
2. **Locate the root endpoint** (app.get('/'))
3. **Add the health endpoint immediately after**
4. **Ensure proper formatting and indentation**
5. **Save the file**
6. **Restart the server** if running

## Complete Implementation Example

Your src/index.js should look like this after implementation:

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

// Server setup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

## Verification Steps

### 1. Test Basic Functionality
```bash
curl http://localhost:3000/health
```
**Expected:** JSON with status and timestamp

### 2. Verify Response Format
```bash
curl -s http://localhost:3000/health | jq
```
**Expected Output:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.123Z"
}
```

### 3. Check Status Code
```bash
curl -i http://localhost:3000/health | head -1
```
**Expected:** `HTTP/1.1 200 OK`

### 4. Test Timestamp Changes
```bash
# Run twice with a pause
curl -s http://localhost:3000/health | jq .timestamp
sleep 1
curl -s http://localhost:3000/health | jq .timestamp
```
**Expected:** Different timestamps

### 5. Verify Logging
Check console for:
```
2024-01-15T14:32:17.123Z - GET /health
```

## Common Mistakes to Avoid

### 1. Static Timestamp
❌ `timestamp: '2024-01-15T14:32:17.123Z'`
✅ `timestamp: new Date().toISOString()`

### 2. Wrong Status Value
❌ `status: 'ok'`
❌ `status: 'healthy!'`
✅ `status: 'healthy'`

### 3. Missing Properties
❌ `res.json({ status: 'healthy' })`
✅ Include both status and timestamp

### 4. Wrong Route Path
❌ `/healthcheck`
❌ `/status`
✅ `/health`

## Troubleshooting

### Issue: 404 Not Found
- Check route is defined before 404 handler
- Verify path is exactly `/health`
- Ensure server was restarted

### Issue: Invalid Timestamp
- Use `new Date().toISOString()`
- Don't use `Date.now()` or other formats

### Issue: Wrong Response Format
- Use `res.json()` not `res.send()`
- Ensure object has both required properties

## Success Indicators
- Endpoint responds at /health
- Returns status "healthy"
- Timestamp is valid ISO 8601
- Timestamp updates each request
- Logs show GET /health entries

Complete the implementation and run all verification tests.