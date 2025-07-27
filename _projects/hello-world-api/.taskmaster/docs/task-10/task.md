# Task 10: Implement Health Check Endpoint

## Overview
This task implements a health check endpoint (GET /health) that provides real-time status information about the API service. The endpoint returns the service health status along with a current timestamp, enabling monitoring tools and load balancers to verify the API is operational.

## Purpose and Objectives
- Implement GET /health endpoint for service monitoring
- Return service status ('healthy') and current timestamp
- Provide a standard health check interface
- Enable integration with monitoring and orchestration tools
- Support load balancer health probes
- Follow health check best practices

## Technical Approach

### Health Check Design
1. **Endpoint Path**: /health following common conventions
2. **HTTP Method**: GET for simple status retrieval
3. **Response Format**: JSON with status and timestamp
4. **Status Code**: 200 for healthy, potential for 503 if unhealthy
5. **Timestamp Format**: ISO 8601 for universal compatibility

### Key Technical Decisions
- Keep health check lightweight (no heavy operations)
- Always return 200 with status field (per requirements)
- Use ISO timestamp format for consistency
- Place after root endpoint in route order
- Minimal implementation suitable for MVP

## Implementation Details

### Basic Health Check Implementation
```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### Enhanced Implementation (Optional)
```javascript
// Track server start time
const startTime = new Date();

// Health check endpoint with additional metrics
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: Math.floor(process.uptime()),
    startTime: startTime.toISOString()
  });
});
```

### Integration in src/index.js
```javascript
// Routes
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
```

### Response Format Example
```json
{
  "status": "healthy",
  "timestamp": "2024-01-20T15:30:45.123Z"
}
```

## Dependencies and Requirements

### Prerequisites
- Completed Task 8: Main server file exists
- Express.js server is functional
- Basic routing is understood

### Technical Requirements
- Express.js route handling
- JavaScript Date object and ISO string conversion
- JSON response formatting

## Testing Strategy

### Manual Testing

1. **Using curl**
   ```bash
   curl -i http://localhost:3000/health
   
   # Expected response:
   HTTP/1.1 200 OK
   Content-Type: application/json
   
   {
     "status": "healthy",
     "timestamp": "2024-01-20T15:30:45.123Z"
   }
   ```

2. **Timestamp Validation**
   ```bash
   # Check timestamp is current
   curl -s http://localhost:3000/health | \
   node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8'));
   const diff = Date.now() - new Date(d.timestamp);
   console.log(diff < 1000 ? '✓ Timestamp is current' : '✗ Timestamp is stale');"
   ```

3. **Multiple Requests**
   ```bash
   # Verify timestamps change
   for i in {1..3}; do
     curl -s http://localhost:3000/health | jq '.timestamp'
     sleep 1
   done
   ```

### Automated Testing
```javascript
// test-health.js
const http = require('http');

const options = {
  hostname: 'localhost',
  port: 3000,
  path: '/health',
  method: 'GET'
};

const req = http.request(options, (res) => {
  let data = '';
  
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    try {
      const response = JSON.parse(data);
      
      // Check status code
      console.log('Status Code:', res.statusCode === 200 ? '✓ 200' : '✗ ' + res.statusCode);
      
      // Check status field
      console.log('Health Status:', response.status === 'healthy' ? '✓ healthy' : '✗ ' + response.status);
      
      // Check timestamp
      const timestamp = new Date(response.timestamp);
      const isValid = !isNaN(timestamp.getTime());
      console.log('Timestamp Valid:', isValid ? '✓ Valid ISO date' : '✗ Invalid date');
      
      // Check timestamp is recent
      const age = Date.now() - timestamp.getTime();
      console.log('Timestamp Fresh:', age < 5000 ? '✓ Recent' : '✗ Stale');
      
    } catch (e) {
      console.error('✗ Failed to parse response:', e.message);
    }
  });
});

req.on('error', console.error);
req.end();
```

### Success Criteria
- ✅ GET /health returns 200 status code
- ✅ Response contains "status" field with value "healthy"
- ✅ Response contains "timestamp" field
- ✅ Timestamp is valid ISO 8601 format
- ✅ Timestamp reflects current time (within 1 second)
- ✅ Content-Type is application/json
- ✅ Response time is under 50ms

## Related Tasks
- **Previous**: Task 8 - Create Main Server File
- **Parallel**: Task 9 - Implement Root Endpoint
- **Next**: Task 11 - Add Basic Error Handling

## Notes and Considerations
- Health checks should be fast and lightweight
- In production, might check database connectivity
- Load balancers typically call this endpoint frequently
- Consider adding version information in production
- Timestamp helps verify the response is fresh
- Some systems expect specific response formats
- Could add memory/CPU metrics for detailed monitoring
- Status should reflect actual service health in production