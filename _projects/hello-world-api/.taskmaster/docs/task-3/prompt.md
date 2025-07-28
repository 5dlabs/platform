# Task 3: Implement API Endpoints - Autonomous Agent Prompt

You are an experienced Node.js developer tasked with implementing all the REST API endpoints for the Hello World API service. You need to create modular route handlers that follow best practices and use the standardized response format.

## Your Mission
Implement five API endpoints across four route modules, ensuring proper input validation, error handling, and consistent response formatting using the utilities created in Task 2.

## Detailed Instructions

### 1. Update the Main Router (src/routes/index.js)
First, update the existing placeholder file to import and use all route modules:
```javascript
const express = require('express');
const router = express.Router();

router.use(require('./health'));
router.use(require('./hello'));
router.use(require('./echo'));
router.use(require('./info'));

module.exports = router;
```

### 2. Create Health Check Endpoint (src/routes/health.js)
Create a new file that implements the health check endpoint:

**Requirements:**
- GET /health endpoint
- Returns a success response with status 'up'
- Use the success utility from '../utils/response'
- Include Swagger documentation comment
- Response data should be: `{ status: 'up' }`

### 3. Create Greeting Endpoints (src/routes/hello.js)
Create a new file with two endpoints:

**Basic Greeting (GET /hello):**
- Returns "Hello, World!" message
- Response data should include: `{ greeting: 'Hello, World!' }`

**Personalized Greeting (GET /hello/:name):**
- Accepts name as URL parameter
- Validates name is not empty or whitespace
- Sanitizes name by removing special characters (keep only letters, numbers, spaces)
- Returns personalized greeting
- Handle errors using the error utility
- Response data should include: `{ greeting: 'Hello, [name]!' }`

**Validation Rules:**
- Empty name should return 400 error
- Use regex `/[^a-zA-Z0-9 ]/g` to sanitize input

### 4. Create Echo Service (src/routes/echo.js)
Create a new file that implements the echo endpoint:

**Requirements:**
- POST /echo endpoint
- Validates request body exists and is not empty
- Returns the exact request body as response data
- Use error utility for validation failures
- Check with: `!req.body || Object.keys(req.body).length === 0`

### 5. Create Service Info Endpoint (src/routes/info.js)
Create a new file that implements the info endpoint:

**Requirements:**
- GET /info endpoint
- Track server start time at module level
- Calculate uptime in seconds
- Return comprehensive service information

**Required Info Fields:**
```javascript
{
  version: // from package.json
  name: // from package.json
  uptime: // "[seconds] seconds"
  environment: // NODE_ENV or 'development'
  hostname: // from os.hostname()
  platform: // from os.platform()
  memory: {
    total: // "[MB] MB" - use os.totalmem()
    free: // "[MB] MB" - use os.freemem()
  }
}
```

**Implementation Notes:**
- Import package.json using: `require('../../package.json')`
- Convert memory from bytes to MB: `Math.round(bytes / (1024 * 1024))`
- Calculate uptime: `Math.floor((new Date() - startTime) / 1000)`

## Swagger Documentation Requirements

Each endpoint must include JSDoc Swagger comments. Example format:
```javascript
/**
 * @swagger
 * /endpoint:
 *   method:
 *     summary: Brief description
 *     description: Detailed description
 *     parameters: (if applicable)
 *     requestBody: (if applicable)
 *     responses:
 *       200:
 *         description: Success description
 *       400:
 *         description: Error description
 */
```

## Response Format Compliance

All endpoints must use the response utilities:
- Success responses: `res.json(success(message, data))`
- Error responses: `next(error(message, statusCode))`

Example success response:
```json
{
  "status": "success",
  "message": "Descriptive message",
  "data": { ... },
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

## Testing Your Implementation

After creating all files, test each endpoint:

```bash
# Start the server
npm run dev

# Test health
curl http://localhost:3000/health

# Test hello
curl http://localhost:3000/hello

# Test personalized hello
curl http://localhost:3000/hello/John

# Test echo
curl -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'

# Test info
curl http://localhost:3000/info
```

## Common Pitfalls to Avoid
- Don't forget to export the router from each file
- Ensure all imports use correct relative paths
- Remember to use `next()` for error handling
- Don't expose sensitive information in /info
- Make sure to sanitize user input in /hello/:name
- Check that request body exists before accessing properties

## Expected Results
- All endpoints return 200 status for success cases
- All responses include correlation ID header
- Error responses use appropriate status codes
- Response format is consistent across all endpoints
- No crashes on invalid input

Complete this task by implementing all five endpoints across the four route files, ensuring each follows the specifications exactly.