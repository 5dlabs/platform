# Task 10: Implement Health Check Endpoint

## Overview
**Title**: Implement Health Check Endpoint  
**Status**: pending  
**Priority**: medium  
**Dependencies**: Task 8 (Create Main Server File)  

## Description
Create a health check endpoint that returns the service status and current timestamp. This endpoint is essential for monitoring tools, load balancers, and operational dashboards to verify that the API service is running and responsive.

## Technical Approach

### 1. Basic Implementation
- Create GET /health route handler
- Return JSON with status and timestamp
- Use ISO 8601 timestamp format

### 2. Response Structure
- Status field indicating service health
- Timestamp showing current server time
- Consistent JSON format

### 3. Extensibility
- Design for future enhancements
- Consider additional health metrics
- Maintain simple initial implementation

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

### Integration in index.js
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

// ... error handling code ...
```

### Response Specification

#### HTTP Response
- **Status Code**: 200 (OK)
- **Content-Type**: application/json
- **Body**: 
  ```json
  {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:45.123Z"
  }
  ```

### Advanced Implementation (Future Enhancement)
```javascript
const os = require('os');

// Service status determination
const getServiceStatus = () => {
  // Future: Check database connections, external services, etc.
  return 'healthy';
};

// Generate ISO timestamp
const generateTimestamp = () => {
  return new Date().toISOString();
};

// Enhanced health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: getServiceStatus(),
    timestamp: generateTimestamp(),
    // Future enhancements:
    // uptime: process.uptime(),
    // memory: {
    //   free: os.freemem(),
    //   total: os.totalmem()
    // },
    // cpu: os.cpus().length
  });
});
```

## Subtasks Breakdown

### 1. Create basic health check route handler
- **Status**: pending
- **Dependencies**: None
- **Implementation**: Add GET /health route
- **Response**: Basic JSON with status and timestamp

### 2. Add timestamp generation function
- **Status**: pending
- **Dependencies**: Subtask 1
- **Purpose**: Modularize timestamp generation
- **Benefit**: Improved testability

### 3. Add service status determination logic
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Function to determine health
- **Future**: Could check dependencies

### 4. Add detailed health information to response
- **Status**: pending
- **Dependencies**: Subtasks 1, 3
- **Enhancement**: System metrics (uptime, memory)
- **Note**: Optional for basic implementation

### 5. Add documentation for the health check endpoint
- **Status**: pending
- **Dependencies**: All subtasks
- **Implementation**: Code comments and API docs
- **Purpose**: Maintainability

## Dependencies
- Express.js server running (from Task 8)
- No external services required
- Node.js built-in Date object

## Testing Strategy

### Manual Testing

#### 1. Basic Health Check
```bash
curl http://localhost:3000/health
# Expected output:
# {"status":"healthy","timestamp":"2024-01-15T10:30:45.123Z"}
```

#### 2. Verify Response Headers
```bash
curl -v http://localhost:3000/health
# Should show:
# < HTTP/1.1 200 OK
# < Content-Type: application/json; charset=utf-8
```

#### 3. Timestamp Validation
```bash
# Make multiple requests and verify timestamps change
curl http://localhost:3000/health; sleep 1; curl http://localhost:3000/health
```

#### 4. Browser Test
- Navigate to http://localhost:3000/health
- Verify JSON response displayed
- Refresh to see timestamp updates

### Expected Server Logs
```
2024-01-15T10:30:45.123Z - GET /health
```

## Common Issues and Solutions

### Issue: Timestamp in wrong format
**Solution**: Use new Date().toISOString() for ISO 8601 format

### Issue: Status always shows healthy
**Solution**: This is correct for basic implementation; enhance later

### Issue: Route conflicts with other endpoints
**Solution**: Ensure /health is unique and defined properly

## API Documentation

### GET /health
**Description**: Returns the current health status of the API service  
**Purpose**: Used by monitoring tools and load balancers  
**Parameters**: None  
**Headers**: None required  

**Success Response**:
- **Code**: 200
- **Content**: 
  ```json
  {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:45.123Z"
  }
  ```

**Response Fields**:
- `status` (string): Service health status ("healthy" or "unhealthy")
- `timestamp` (string): Current server time in ISO 8601 format

**Example Request**:
```bash
GET /health HTTP/1.1
Host: localhost:3000
```

**Example Response**:
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Content-Length: 63

{"status":"healthy","timestamp":"2024-01-15T10:30:45.123Z"}
```

## Use Cases

### 1. Load Balancer Health Checks
- Periodic requests to verify service availability
- Remove unhealthy instances from rotation
- Typically checked every 5-30 seconds

### 2. Monitoring Systems
- Track service uptime and availability
- Alert on health check failures
- Collect timestamp for latency monitoring

### 3. Container Orchestration
- Kubernetes liveness/readiness probes
- Docker health checks
- Auto-restart on failures

### 4. Manual Troubleshooting
- Quick verification of service status
- Check server time synchronization
- Validate basic connectivity

## Performance Considerations

- Minimal processing overhead
- No external dependencies
- Response time should be < 5ms
- Suitable for frequent polling

## Security Considerations

- No sensitive information exposed
- No authentication required (public endpoint)
- Safe for external monitoring services
- Consider rate limiting in production

## Next Steps
After completing this task:
- Add error handling middleware (Task 11)
- Create comprehensive documentation (Task 12)
- Test all endpoints together (Task 13)
- Consider advanced health metrics

The API now has both functional and operational endpoints ready for deployment and monitoring.