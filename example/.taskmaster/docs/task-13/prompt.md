# Autonomous Prompt: Implement Health Check Endpoint

## Task Context
You are an AI assistant tasked with implementing a health check endpoint for an Express.js TypeScript application. This endpoint will provide service status information for monitoring and operational purposes.

## Objective
Create a robust health check endpoint that returns service status, timestamp, and operational information in a consistent JSON format.

## Required Actions

### 1. Create Health Route File
Create `src/routes/health.ts` with the following implementation:

```typescript
import { Router, Request, Response } from 'express';

const router = Router();

/**
 * Health check endpoint
 * Returns service status and operational information
 */
router.get('/health', (req: Request, res: Response) => {
  const healthData = {
    status: 'ok',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    service: 'express-typescript-api',
    version: '1.0.0',
    environment: process.env.NODE_ENV || 'development'
  };

  res.status(200).json(healthData);
});

/**
 * Simple ping endpoint for basic connectivity tests
 */
router.get('/ping', (req: Request, res: Response) => {
  res.status(200).json({ message: 'pong' });
});

export default router;
```

### 2. Enhanced Health Check (Optional)
For a more comprehensive health check, extend the implementation:

```typescript
import { Router, Request, Response } from 'express';

const router = Router();

interface HealthStatus {
  status: 'ok' | 'warning' | 'error';
  timestamp: string;
  uptime: number;
  service: string;
  version: string;
  environment: string;
  checks: {
    memory: {
      status: 'ok' | 'warning' | 'error';
      used: number;
      total: number;
      percentage: number;
    };
    disk?: {
      status: 'ok' | 'warning' | 'error';
      available: number;
    };
  };
}

router.get('/health', (req: Request, res: Response) => {
  const memoryUsage = process.memoryUsage();
  const totalMemory = memoryUsage.heapTotal;
  const usedMemory = memoryUsage.heapUsed;
  const memoryPercentage = (usedMemory / totalMemory) * 100;

  const healthData: HealthStatus = {
    status: 'ok',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    service: 'express-typescript-api',
    version: '1.0.0',
    environment: process.env.NODE_ENV || 'development',
    checks: {
      memory: {
        status: memoryPercentage > 90 ? 'error' : memoryPercentage > 70 ? 'warning' : 'ok',
        used: usedMemory,
        total: totalMemory,
        percentage: Math.round(memoryPercentage * 100) / 100
      }
    }
  };

  // Set overall status based on individual checks
  if (healthData.checks.memory.status === 'error') {
    healthData.status = 'error';
  } else if (healthData.checks.memory.status === 'warning') {
    healthData.status = 'warning';
  }

  const statusCode = healthData.status === 'ok' ? 200 : 
                    healthData.status === 'warning' ? 200 : 503;

  res.status(statusCode).json(healthData);
});

export default router;
```

### 3. Update Main Application
Modify `src/index.ts` to include the health routes:

```typescript
import express from 'express';
import healthRoutes from './routes/health';

const app = express();
const port = process.env.PORT || 3000;

// Middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Health check routes (should be early in middleware stack)
app.use('/api', healthRoutes);

// Default route
app.get('/', (req, res) => {
  res.json({ 
    message: 'Express TypeScript server is running!',
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV || 'development'
  });
});

// Start server
app.listen(port, () => {
  console.log(`⚡️ Server is running at http://localhost:${port}`);
  console.log(`📊 Health check available at http://localhost:${port}/api/health`);
});

export default app;
```

### 4. Create Health Check Types (Optional)
Create `src/types/health.ts` for type safety:

```typescript
export interface HealthCheck {
  status: 'ok' | 'warning' | 'error';
  timestamp: string;
  uptime: number;
  service: string;
  version: string;
  environment: string;
}

export interface DetailedHealthCheck extends HealthCheck {
  checks: {
    memory: MemoryCheck;
    disk?: DiskCheck;
    database?: DatabaseCheck;
  };
}

export interface MemoryCheck {
  status: 'ok' | 'warning' | 'error';
  used: number;
  total: number;
  percentage: number;
}

export interface DiskCheck {
  status: 'ok' | 'warning' | 'error';
  available: number;
  total: number;
  percentage: number;
}

export interface DatabaseCheck {
  status: 'ok' | 'warning' | 'error';
  responseTime: number;
  connected: boolean;
}
```

## Validation Steps
1. **Build Test**: Run `npm run build` to ensure TypeScript compiles
2. **Server Start**: Run `npm run dev` to start the development server
3. **Health Check Test**: Test the endpoint with curl:
   ```bash
   curl -X GET http://localhost:3000/api/health
   ```
4. **Response Validation**: Verify the response contains all required fields
5. **Status Code**: Confirm 200 OK status code is returned
6. **JSON Format**: Verify response is valid JSON

## Testing Commands
```bash
# Start the server
npm run dev

# Test health endpoint
curl -X GET http://localhost:3000/api/health

# Test with headers
curl -X GET http://localhost:3000/api/health -H "Accept: application/json"

# Test ping endpoint (if implemented)
curl -X GET http://localhost:3000/api/ping

# Test response time
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/api/health
```

## Success Criteria
- [ ] Health endpoint responds with 200 OK
- [ ] Response includes status, timestamp, uptime, service name
- [ ] Timestamp is in ISO 8601 format
- [ ] Response is valid JSON
- [ ] Endpoint is accessible after server restart
- [ ] No authentication required for health checks
- [ ] Response time is under 100ms
- [ ] Proper TypeScript types are used

## Error Handling
- If route conflicts occur, check path mounting in main app
- If TypeScript errors appear, verify import statements
- If server doesn't start, check port availability
- If health endpoint returns 404, verify route registration

## Load Testing (Optional)
Test the health endpoint under load:
```bash
# Install hey (HTTP load testing tool)
# go install github.com/rakyll/hey@latest

# Test with 100 concurrent requests
hey -n 1000 -c 100 http://localhost:3000/api/health
```

## Integration with Monitoring Systems

### Prometheus Integration
```typescript
// Add metrics endpoint for Prometheus
router.get('/metrics', (req: Request, res: Response) => {
  const metrics = `
# HELP api_uptime_seconds Total uptime of the API
# TYPE api_uptime_seconds gauge
api_uptime_seconds ${process.uptime()}

# HELP api_memory_usage_bytes Current memory usage
# TYPE api_memory_usage_bytes gauge
api_memory_usage_bytes ${process.memoryUsage().heapUsed}
`;
  
  res.set('Content-Type', 'text/plain');
  res.send(metrics);
});
```

### Docker Health Check
```dockerfile
# Add to Dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/api/health || exit 1
```

### Kubernetes Readiness/Liveness Probes
```yaml
# Add to Kubernetes deployment
livenessProbe:
  httpGet:
    path: /api/health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /api/health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Final Deliverables
- [ ] `src/routes/health.ts` with health check implementation
- [ ] Updated `src/index.ts` with health routes integration
- [ ] Optional: `src/types/health.ts` with TypeScript definitions
- [ ] Health endpoint responding at `/api/health`
- [ ] Comprehensive response with service information
- [ ] Proper error handling and status codes
- [ ] Documentation for monitoring integration