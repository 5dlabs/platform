# Hello World API - Architecture Document

## System Overview
The Hello World API is a lightweight Node.js REST service designed for testing the 5D Labs orchestrator workflow. It follows a simple three-layer architecture with clear separation of concerns.

## Architecture Layers

### 1. API Layer (Routes)
- **Express.js routes** handling HTTP requests
- **Input validation** using middleware
- **Error handling** with standardized responses
- **OpenAPI documentation** generation

### 2. Business Logic Layer (Services)
- **GreetingService** - Handles greeting logic
- **InfoService** - Manages service information
- **HealthService** - Health check implementation
- **ValidationService** - Input validation logic

### 3. Infrastructure Layer
- **Logger** - Structured logging with correlation IDs
- **Config** - Environment-based configuration
- **ErrorHandler** - Centralized error processing
- **ResponseFormatter** - Standardized API responses

## Directory Structure
```
hello-world-api/
├── src/
│   ├── controllers/          # Route handlers
│   │   ├── healthController.js
│   │   ├── helloController.js
│   │   ├── echoController.js
│   │   └── infoController.js
│   ├── services/             # Business logic
│   │   ├── greetingService.js
│   │   ├── infoService.js
│   │   └── healthService.js
│   ├── middleware/           # Express middleware
│   │   ├── errorHandler.js
│   │   ├── logger.js
│   │   └── validator.js
│   ├── utils/               # Utilities
│   │   ├── responseFormatter.js
│   │   └── config.js
│   └── app.js               # Express app setup
├── tests/                   # Test files
│   ├── unit/               # Unit tests
│   ├── integration/        # Integration tests
│   └── fixtures/           # Test data
├── docs/                   # Documentation
│   └── openapi.yaml        # API specification
├── Dockerfile              # Container definition
├── k8s/                    # Kubernetes manifests
│   ├── deployment.yaml
│   ├── service.yaml
│   └── configmap.yaml
├── package.json
└── README.md
```

## API Design

### Endpoint Structure
```
GET  /health          -> Health check
GET  /hello           -> Basic greeting
GET  /hello/:name     -> Personalized greeting
POST /echo            -> Echo request body
GET  /info            -> Service information
GET  /docs            -> API documentation
```

### Response Format
All endpoints return JSON in this standardized format:
```json
{
  "status": "success|error",
  "message": "Human readable message",
  "data": "Response payload",
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

### Error Handling
- **4xx errors**: Client errors (validation, not found)
- **5xx errors**: Server errors (internal failures)
- Correlation IDs for request tracing
- Structured error responses with details

## Configuration Management
Environment variables for runtime configuration:
- `PORT` - Server port (default: 3000)
- `NODE_ENV` - Environment (development/production)
- `LOG_LEVEL` - Logging verbosity
- `API_VERSION` - Service version identifier

## Container Strategy
Multi-stage Docker build:
1. **Build stage**: Install dependencies, run tests
2. **Production stage**: Copy only runtime files
3. Health check endpoint for container orchestration
4. Non-root user for security
5. Graceful shutdown handling

## Deployment Architecture
```
┌─────────────────┐
│   Load Balancer │
└─────────┬───────┘
          │
┌─────────▼───────┐
│  Kubernetes     │
│  Service        │
└─────────┬───────┘
          │
    ┌─────▼─────┐ ┌─────────┐ ┌─────────┐
    │   Pod 1   │ │  Pod 2  │ │  Pod N  │
    │ (API App) │ │(API App)│ │(API App)│
    └───────────┘ └─────────┘ └─────────┘
```

## Monitoring & Observability
- Health check endpoint for liveness/readiness probes
- Structured JSON logging
- Request correlation IDs
- Response time tracking in logs
- Container metrics via Kubernetes

## Testing Strategy
- **Unit tests**: Individual functions and services
- **Integration tests**: API endpoint testing
- **Contract tests**: OpenAPI specification validation
- **Load tests**: Performance under concurrent load
- Coverage target: >90%

## Security Considerations
- Input validation on all endpoints
- No sensitive data in logs
- Non-root container execution
- Minimal attack surface (no unnecessary dependencies)
- CORS configuration for browser access

## Performance Targets
- Response time: <100ms (95th percentile)
- Throughput: 100 concurrent requests
- Memory usage: <128MB per container
- CPU usage: <0.1 cores per container
- Container startup: <5 seconds

## Dependencies
**Runtime Dependencies:**
- express: Web framework
- helmet: Security headers
- cors: Cross-origin resource sharing
- winston: Structured logging

**Development Dependencies:**
- jest: Testing framework
- supertest: HTTP testing
- eslint: Code linting
- swagger-ui-express: API documentation

## Deployment Pipeline
1. Code commit triggers CI
2. Run unit and integration tests
3. Build Docker image
4. Push to container registry
5. Deploy to Kubernetes cluster
6. Run smoke tests
7. Health check validation