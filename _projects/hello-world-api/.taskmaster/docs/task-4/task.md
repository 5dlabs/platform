# Task 4: Implement Health Check Endpoint

## Overview

This task implements a health check endpoint that provides real-time status information about the API service. Health endpoints are critical for monitoring, load balancing, and orchestration systems to determine if a service is operational.

## Purpose and Objectives

The health check endpoint serves multiple purposes:

- Provides a standardized way to monitor service availability
- Returns current timestamp for uptime tracking
- Enables automated health monitoring by infrastructure tools
- Supports container orchestration platforms (Kubernetes, Docker Swarm)
- Facilitates load balancer health checks

## Technical Approach

### 1. Health Check Standards
- Follow industry conventions for health endpoints
- Return JSON format for consistency
- Include minimal but useful information
- Ensure fast response times

### 2. Timestamp Implementation
- Use ISO 8601 format for universal compatibility
- Generate timestamp at request time (not cached)
- Ensure timezone information is included

### 3. Status Reporting
- Simple "healthy" status for basic implementation
- Extensible design for future enhancements
- No external dependency checks in basic version

## Implementation Details

### Route Implementation

Add the health check endpoint to `src/index.js` after the hello endpoint:

```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### Complete Server Structure

The health endpoint should be integrated as follows:

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

### Response Format

The health endpoint returns:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.845Z"
}
```

### Technical Details

1. **Path**: `/health` - Common convention for health checks
2. **Method**: GET - Read-only operation
3. **Status Code**: 200 - Indicates healthy service
4. **Response Fields**:
   - `status`: Always "healthy" in basic implementation
   - `timestamp`: Current time in ISO 8601 format

## Dependencies and Requirements

### Prerequisites
- Completed Tasks 1-3
- Express.js server running with hello endpoint

### Technical Requirements
- No additional npm packages required
- Uses built-in Date object for timestamps
- Maintains existing middleware chain

### Future Enhancement Possibilities
- Add service version information
- Include uptime duration
- Report memory usage
- Check database connectivity
- Monitor external service dependencies

## Testing Strategy

### Manual Testing

1. **Basic Health Check**
   ```bash
   curl http://localhost:3000/health
   ```
   Expected response format:
   ```json
   {
     "status": "healthy",
     "timestamp": "2024-01-15T14:32:17.845Z"
   }
   ```

2. **Timestamp Validation**
   ```bash
   # Make two requests and verify different timestamps
   curl http://localhost:3000/health; sleep 1; curl http://localhost:3000/health
   ```

3. **Response Headers**
   ```bash
   curl -I http://localhost:3000/health
   ```
   Verify: `HTTP/1.1 200 OK` and `Content-Type: application/json`

4. **Load Testing**
   ```bash
   # Quick load test - 100 requests
   for i in {1..100}; do curl -s http://localhost:3000/health > /dev/null & done; wait
   ```

### Automated Testing

Create `test-health-endpoint.js`:

```javascript
const http = require('http');
const assert = require('assert');

function testHealthEndpoint() {
  http.get('http://localhost:3000/health', (res) => {
    let data = '';
    
    res.on('data', chunk => data += chunk);
    
    res.on('end', () => {
      const response = JSON.parse(data);
      
      // Test status code
      console.log('Status Code:', res.statusCode === 200 ? '✓ PASS' : '✗ FAIL');
      
      // Test response structure
      console.log('Has status field:', response.status ? '✓ PASS' : '✗ FAIL');
      console.log('Has timestamp field:', response.timestamp ? '✓ PASS' : '✗ FAIL');
      
      // Test status value
      console.log('Status is healthy:', response.status === 'healthy' ? '✓ PASS' : '✗ FAIL');
      
      // Test timestamp format
      const timestampValid = !isNaN(Date.parse(response.timestamp));
      console.log('Valid ISO timestamp:', timestampValid ? '✓ PASS' : '✗ FAIL');
      
      console.log('\nResponse:', response);
    });
  });
}

testHealthEndpoint();
```

### Integration Testing

1. **With Monitoring Tools**
   - Configure monitoring tool to poll `/health` every 30 seconds
   - Verify consistent 200 responses
   - Check timestamp progression

2. **Container Health Checks**
   ```dockerfile
   HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
     CMD curl -f http://localhost:3000/health || exit 1
   ```

3. **Load Balancer Integration**
   - Configure load balancer health check to `/health`
   - Verify service remains in rotation

## Success Criteria

The health check endpoint is complete when:

1. GET `/health` returns 200 status code
2. Response contains "status" field with value "healthy"
3. Response contains "timestamp" field with valid ISO 8601 date
4. Timestamp updates with each request (not cached)
5. Response time is under 10ms
6. Endpoint handles high request volume (1000+ req/sec)
7. JSON format is valid and consistent

## Common Issues and Solutions

### Issue 1: Cached Timestamps
**Problem**: Same timestamp on multiple requests
**Solution**: Ensure `new Date().toISOString()` is called inside route handler

### Issue 2: Invalid Date Format
**Problem**: Timestamp not in ISO 8601 format
**Solution**: Use `.toISOString()` method, not custom formatting

### Issue 3: Wrong Status Value
**Problem**: Status shows "ok" or "alive" instead of "healthy"
**Solution**: Use exact string "healthy" as specified

### Issue 4: Additional Fields
**Problem**: Response includes extra fields not in spec
**Solution**: Return only status and timestamp fields

## Best Practices Implemented

1. **Standardized Path**: `/health` is widely recognized
2. **Minimal Payload**: Only essential information included
3. **Fast Response**: No external calls or heavy processing
4. **Consistent Format**: JSON response matches other endpoints
5. **Real-time Data**: Timestamp generated per request

## Use Cases

### 1. Kubernetes Liveness Probe
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```

### 2. AWS ELB Health Check
- Target: HTTP:3000/health
- Interval: 30 seconds
- Healthy threshold: 2

### 3. Monitoring Dashboard
- Poll endpoint every minute
- Graph timestamp progression
- Alert on non-200 responses

### 4. CI/CD Pipeline
```bash
# Wait for service to be healthy
until curl -f http://localhost:3000/health; do
  echo "Waiting for service..."
  sleep 1
done
```

## Performance Considerations

- Endpoint should respond in < 10ms
- No database queries or external API calls
- Minimal memory allocation per request
- Suitable for high-frequency polling

## Security Considerations

- No sensitive information exposed
- No authentication required (public health status)
- Rate limiting may be applied if needed
- No user input processed

## Next Steps

After completing this task:
- Task 5: Add comprehensive error handling
- Consider adding more health metrics:
  - Uptime duration
  - Memory usage
  - Version information
  - Connected clients count
- Implement `/ready` endpoint for readiness checks