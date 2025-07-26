# Task 4: Implement Health Check Endpoint

## Overview
This task implements a health check endpoint that allows monitoring services and load balancers to verify the API is operational. The endpoint returns the service status and current timestamp, providing essential information for automated health monitoring and service discovery.

## Objectives
- Implement GET /health endpoint
- Return service status as "healthy"
- Include current timestamp in ISO 8601 format
- Ensure HTTP 200 status code
- Enable automated monitoring capabilities

## Technical Approach

### 1. Health Check Pattern
Health endpoints are a standard practice in modern APIs, providing:
- Service availability confirmation
- Timestamp for uptime tracking
- Simple response for minimal overhead
- Consistent format for monitoring tools

### 2. Response Structure
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.000Z"
}
```

### 3. Timestamp Generation
Uses JavaScript's `Date.toISOString()` for consistent ISO 8601 format, ensuring compatibility with monitoring systems worldwide.

## Implementation Details

### Complete Route Handler
```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

### Route Placement
1. After the root endpoint (/)
2. Before error handling middleware
3. After request logging middleware
4. Before 404 catch-all handler

### Response Characteristics
- **Dynamic**: Timestamp changes with each request
- **Lightweight**: Minimal processing required
- **Stateless**: No database or external service checks
- **Always Available**: Returns healthy if server is running

## Dependencies
- **Task 2**: Create Express.js Server (provides server infrastructure)
- **Express.js**: Web framework for route handling
- **JavaScript Date API**: For timestamp generation

## Success Criteria
- [ ] GET /health endpoint defined
- [ ] Returns JSON with status and timestamp
- [ ] Status value is exactly "healthy"
- [ ] Timestamp is valid ISO 8601 format
- [ ] HTTP status code is 200
- [ ] Endpoint responds quickly (< 50ms)

## Testing Strategy

### Manual Testing
```bash
# Basic health check
curl http://localhost:3000/health

# With headers
curl -i http://localhost:3000/health

# Pretty print JSON
curl -s http://localhost:3000/health | jq
```

### Expected Response
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.123Z"
}
```

### Automated Testing Script
```bash
#!/bin/bash
# Test health endpoint
response=$(curl -s http://localhost:3000/health)
status=$(echo $response | jq -r .status)
timestamp=$(echo $response | jq -r .timestamp)

if [ "$status" = "healthy" ] && [ -n "$timestamp" ]; then
  echo "Health check passed"
else
  echo "Health check failed"
  exit 1
fi
```

### Validation Points
1. Status is always "healthy"
2. Timestamp is present and valid
3. Timestamp changes between requests
4. Response time is acceptable
5. JSON format is valid

## Use Cases

### 1. Load Balancer Health Checks
- AWS ELB/ALB health checks
- Kubernetes liveness probes
- Docker health checks

### 2. Monitoring Systems
- Prometheus endpoint scraping
- DataDog service checks
- New Relic synthetics

### 3. Service Discovery
- Consul health checks
- Eureka heartbeats
- Custom service registries

## Related Tasks
- **Task 3**: Implements root endpoint (similar pattern)
- **Task 5**: Add error handling (affects all endpoints)
- **Task 8**: Will test this endpoint
- **Task 9**: May suggest health check best practices

## Notes
- Consider adding more detailed health checks in production (database, dependencies)
- Timestamp helps debug timezone issues
- Status could be expanded (healthy, degraded, unhealthy)
- Consider adding version information in future iterations