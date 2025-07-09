# Task 13: Implement Health Check Endpoint

## Overview
Create a GET /health endpoint that returns service status and timestamp to enable monitoring and health checks for the Express application.

## Description
This task involves implementing a health check endpoint that provides essential service status information. The endpoint will return a simple JSON response indicating the service is operational and include a timestamp for monitoring purposes.

## Priority
Medium

## Dependencies
- Task 11: Initialize Express TypeScript Project (must be completed first)

## Implementation Steps

### 1. Create health route file
- Create `src/routes/health.ts` with Express router setup
- Import necessary Express types and functions
- Set up router instance for health-related routes

### 2. Implement health handler
- Create GET /health handler that returns status and timestamp
- Return JSON response with consistent structure
- Include current timestamp in ISO format
- Add optional service information

### 3. Wire health routes to app
- Import health router in main Express app
- Mount health routes at appropriate path
- Ensure middleware compatibility

## Implementation Details

### Health Route Structure
```typescript
import { Router, Request, Response } from 'express';

const router = Router();

router.get('/health', (req: Request, res: Response) => {
  res.json({
    status: 'ok',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    service: 'express-typescript-api'
  });
});

export default router;
```

### Integration with Main App
```typescript
// In src/index.ts
import healthRoutes from './routes/health';

app.use('/api', healthRoutes);
// or
app.use(healthRoutes);
```

### Response Format
```json
{
  "status": "ok",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "uptime": 123.456,
  "service": "express-typescript-api"
}
```

## File Structure
```
src/
├── routes/
│   └── health.ts
└── index.ts (updated)
```

## Test Strategy
- Manual test with curl or Postman to verify 200 response
- Check response format and required fields
- Verify timestamp is current and in ISO format
- Test endpoint availability after server restart
- Load test to ensure performance under stress

## Expected Outcomes
- Functional /health endpoint responding with 200 OK
- Consistent JSON response format
- Proper integration with Express application
- Foundation for monitoring and alerting systems

## Common Issues
- **Route conflicts**: Ensure health route doesn't conflict with other routes
- **Middleware order**: Health checks should bypass authentication
- **Response format**: Maintain consistent JSON structure
- **Error handling**: Health checks should always return 200 unless service is down

## Monitoring Integration
- Can be used by load balancers for health checks
- Suitable for monitoring systems (Prometheus, Grafana)
- Container orchestration health probes (Kubernetes, Docker)
- Uptime monitoring services

## Enhanced Features (Optional)
- Database connectivity check
- External service dependency status
- Memory and CPU usage information
- Custom health check logic
- Different status codes for different states

## Next Steps
After completion, this endpoint will:
- Enable basic service monitoring
- Support container orchestration
- Provide foundation for more complex health checks
- Allow integration with monitoring systems