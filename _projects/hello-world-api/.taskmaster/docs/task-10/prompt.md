# Autonomous Agent Prompt for Task 10: Implement Health Check Endpoint

## Task Context
You need to add a health check endpoint to the Express.js server. This endpoint will help monitoring tools and load balancers verify that the API is running and healthy.

## Your Mission
Implement a GET /health endpoint that returns the service status and current timestamp in JSON format.

## Step-by-Step Instructions

### 1. Navigate to Project
```bash
cd hello-world-api
```

### 2. Add Health Check Endpoint
Edit `src/index.js` to add the health endpoint after the root endpoint:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### 3. Enhanced Implementation (Recommended)
For a more robust health check, consider this implementation:

```javascript
const os = require('os');

// Service uptime tracking
const startTime = Date.now();

// Health check endpoint with detailed information
app.get('/health', (req, res) => {
  const healthCheck = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: Math.floor((Date.now() - startTime) / 1000),
    service: {
      name: 'hello-world-api',
      version: '1.0.0',
      environment: process.env.NODE_ENV || 'development'
    }
  };
  
  // Add system information in non-production environments
  if (process.env.NODE_ENV !== 'production') {
    healthCheck.system = {
      memory: {
        free: Math.round(os.freemem() / 1024 / 1024), // MB
        total: Math.round(os.totalmem() / 1024 / 1024) // MB
      },
      cpus: os.cpus().length,
      platform: os.platform(),
      nodejs: process.version
    };
  }
  
  res.status(200).json(healthCheck);
});
```

### 4. Complete Server Structure
Your server file should now have both endpoints:

```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Root endpoint
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
```

### 2. Test Health Endpoint
```bash
curl http://localhost:3000/health
# Expected output (basic):
# {"status":"healthy","timestamp":"2023-12-01T12:00:00.000Z"}
```

### 3. Verify Response Format
```bash
curl -s http://localhost:3000/health | python3 -m json.tool
# Expected formatted output:
# {
#     "status": "healthy",
#     "timestamp": "2023-12-01T12:00:00.000Z"
# }
```

### 4. Test HTTP Status
```bash
curl -I http://localhost:3000/health
# Expected headers:
# HTTP/1.1 200 OK
# Content-Type: application/json
```

### 5. Verify Timestamp Format
```bash
# Check that timestamp is valid ISO 8601
curl -s http://localhost:3000/health | python3 -c "
import json, sys
from datetime import datetime
data = json.load(sys.stdin)
datetime.fromisoformat(data['timestamp'].replace('Z', '+00:00'))
print('Timestamp is valid ISO 8601')
"
```

### 6. Multiple Request Test
```bash
# Make requests 1 second apart to see timestamp changes
for i in {1..3}; do
  curl -s http://localhost:3000/health | grep timestamp
  sleep 1
done
# Expected: Different timestamps for each request
```

## Expected Result
- Health endpoint added to server
- GET /health returns JSON with status and timestamp
- Status is "healthy"
- Timestamp is current time in ISO format
- HTTP 200 status code returned
- Endpoint logs requests like other routes

## Important Notes
- Place endpoint after root endpoint but before server.listen()
- Timestamp should update on each request
- Use ISO 8601 format for timestamp (includes 'Z' for UTC)
- Consider adding more health information for production use
- Keep response format consistent

## Common Issues to Avoid
1. Static timestamp instead of dynamic
2. Wrong timestamp format (not ISO 8601)
3. Missing status or timestamp fields
4. Incorrect HTTP status code
5. Endpoint defined after server.listen()