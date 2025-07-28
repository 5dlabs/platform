# Task 3: Implement API Endpoints

## Overview
This task implements all required REST API endpoints for the Hello World API service. It creates the core functionality including health checks, greeting endpoints, echo service, and service information endpoints, all following the standardized response format.

## Objectives
- Implement health check endpoint for monitoring
- Create basic and personalized greeting endpoints
- Build echo service for request/response testing
- Develop service information endpoint
- Ensure all responses follow standardized format
- Add Swagger/OpenAPI documentation annotations

## Technical Approach

### Endpoint Architecture
The implementation follows a modular approach with separate route files for each logical group of endpoints:
- **health.js**: Service health monitoring
- **hello.js**: Greeting functionality
- **echo.js**: Request echo service
- **info.js**: Service metadata
- **index.js**: Route consolidation

### API Endpoints Overview
1. **GET /health** - Returns service health status
2. **GET /hello** - Returns basic greeting message
3. **GET /hello/:name** - Returns personalized greeting
4. **POST /echo** - Echoes back the request body
5. **GET /info** - Returns service information

### Response Consistency
All endpoints utilize the standardized response utilities from Task 2, ensuring consistent response format across the API.

## Implementation Details

### Step 1: Create Route Consolidator (src/routes/index.js)
```javascript
const express = require('express');
const router = express.Router();

router.use(require('./health'));
router.use(require('./hello'));
router.use(require('./echo'));
router.use(require('./info'));

module.exports = router;
```

### Step 2: Implement Health Check (src/routes/health.js)
```javascript
const express = require('express');
const router = express.Router();
const { success } = require('../utils/response');

/**
 * @swagger
 * /health:
 *   get:
 *     summary: Health check endpoint
 *     description: Returns the service health status
 *     responses:
 *       200:
 *         description: Service is healthy
 */
router.get('/health', (req, res) => {
  res.json(success('Service is healthy', { status: 'up' }));
});

module.exports = router;
```

### Step 3: Implement Greeting Endpoints (src/routes/hello.js)
```javascript
const express = require('express');
const router = express.Router();
const { success, error } = require('../utils/response');

/**
 * @swagger
 * /hello:
 *   get:
 *     summary: Basic hello world endpoint
 *     description: Returns a greeting message
 *     responses:
 *       200:
 *         description: Greeting message
 */
router.get('/hello', (req, res) => {
  res.json(success('Hello, World!', { greeting: 'Hello, World!' }));
});

/**
 * @swagger
 * /hello/{name}:
 *   get:
 *     summary: Personalized greeting
 *     description: Returns a personalized greeting with the provided name
 *     parameters:
 *       - in: path
 *         name: name
 *         required: true
 *         schema:
 *           type: string
 *     responses:
 *       200:
 *         description: Personalized greeting
 *       400:
 *         description: Invalid name parameter
 */
router.get('/hello/:name', (req, res, next) => {
  const { name } = req.params;
  
  if (!name || name.trim() === '') {
    return next(error('Name parameter is required', 400));
  }
  
  const sanitizedName = name.replace(/[^a-zA-Z0-9 ]/g, '');
  res.json(success(`Hello, ${sanitizedName}!`, { greeting: `Hello, ${sanitizedName}!` }));
});

module.exports = router;
```

### Step 4: Implement Echo Service (src/routes/echo.js)
```javascript
const express = require('express');
const router = express.Router();
const { success, error } = require('../utils/response');

/**
 * @swagger
 * /echo:
 *   post:
 *     summary: Echo service
 *     description: Returns the posted JSON data
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             type: object
 *     responses:
 *       200:
 *         description: Echoed data
 *       400:
 *         description: Invalid request body
 */
router.post('/echo', (req, res, next) => {
  if (!req.body || Object.keys(req.body).length === 0) {
    return next(error('Request body is required', 400));
  }
  
  res.json(success('Echo response', req.body));
});

module.exports = router;
```

### Step 5: Implement Service Info (src/routes/info.js)
```javascript
const express = require('express');
const router = express.Router();
const { success } = require('../utils/response');
const os = require('os');
const package = require('../../package.json');

// Track server start time
const startTime = new Date();

/**
 * @swagger
 * /info:
 *   get:
 *     summary: Service information
 *     description: Returns version, uptime, and environment information
 *     responses:
 *       200:
 *         description: Service information
 */
router.get('/info', (req, res) => {
  const uptime = Math.floor((new Date() - startTime) / 1000);
  
  const info = {
    version: package.version,
    name: package.name,
    uptime: `${uptime} seconds`,
    environment: process.env.NODE_ENV || 'development',
    hostname: os.hostname(),
    platform: os.platform(),
    memory: {
      total: `${Math.round(os.totalmem() / (1024 * 1024))} MB`,
      free: `${Math.round(os.freemem() / (1024 * 1024))} MB`
    }
  };
  
  res.json(success('Service information', info));
});

module.exports = router;
```

### Key Implementation Details

#### Input Validation
- Name parameter sanitization removes special characters
- Empty name validation with appropriate error response
- Request body validation for echo endpoint

#### Security Considerations
- Input sanitization to prevent injection attacks
- No sensitive information exposed in /info endpoint
- Proper error handling without exposing internals

#### Performance Optimizations
- Uptime calculation uses simple date arithmetic
- Memory values rounded for readability
- Minimal dependencies for fast response times

## Dependencies and Requirements
- Task 2 must be completed (core application structure)
- Response utilities must be available
- Express router functionality
- Access to os module for system information
- Package.json must exist with version information

## Testing Strategy

### Integration Tests
1. **Health Check Test**
   - Verify 200 status code
   - Check response format
   - Validate status field is 'up'

2. **Greeting Endpoint Tests**
   - Test basic /hello endpoint
   - Test personalized greeting with valid name
   - Test name sanitization with special characters
   - Test empty name parameter handling

3. **Echo Service Tests**
   - Test with valid JSON body
   - Test with empty body
   - Test with various data types
   - Verify exact echo of input

4. **Info Endpoint Test**
   - Verify all required fields present
   - Check uptime increases over time
   - Validate version matches package.json

### Manual Testing Examples
```bash
# Health check
curl http://localhost:3000/health

# Basic greeting
curl http://localhost:3000/hello

# Personalized greeting
curl http://localhost:3000/hello/Alice

# Echo service
curl -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "test"}'

# Service info
curl http://localhost:3000/info
```

## Success Criteria
- All 5 endpoints respond with correct status codes
- Response format matches specification for all endpoints
- Input validation works correctly
- Error cases return appropriate error responses
- Correlation IDs appear in all responses
- Swagger annotations are present for documentation
- No security vulnerabilities in input handling

## Related Tasks
- Task 2: Core Application Structure (prerequisite)
- Task 4: Add Input Validation Middleware (enhancement)
- Task 5: Implement OpenAPI Documentation (uses Swagger annotations)
- Task 6: Create Unit and Integration Tests (tests these endpoints)
- Task 7: Containerize Application (packages these endpoints)