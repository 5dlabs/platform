# Task 4: Implement API Documentation with Swagger - Autonomous Agent Prompt

You are an experienced Node.js developer tasked with implementing comprehensive API documentation using OpenAPI/Swagger. You need to create interactive documentation that developers can use to understand and test the API.

## Your Mission
Set up Swagger documentation for the Hello World API, including a base OpenAPI specification, Swagger UI integration, and automatic documentation generation from existing JSDoc comments in the route files.

## Detailed Instructions

### 1. Create Base OpenAPI Specification (docs/openapi.yaml)
Create a new file with the following content:

```yaml
openapi: 3.0.0
info:
  title: Hello World API
  description: A simple REST API that serves as a "Hello World" example
  version: 1.0.0
servers:
  - url: http://localhost:3000
    description: Local development server
paths:
  # Paths will be generated from JSDoc comments
components:
  schemas:
    Response:
      type: object
      properties:
        status:
          type: string
          enum: [success, error]
        message:
          type: string
        data:
          type: object
          nullable: true
        timestamp:
          type: string
          format: date-time
      required:
        - status
        - message
        - timestamp
```

**Important Notes:**
- This file defines the base structure and reusable schemas
- The paths section is intentionally empty - it will be populated from JSDoc comments
- The Response schema matches the standardized response format from Task 2

### 2. Update Express Application (src/app.js)
You need to add Swagger configuration to the existing app.js file. Add the following code AFTER the middleware configuration but BEFORE the routes configuration:

```javascript
// Swagger documentation setup
const swaggerOptions = {
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'Hello World API',
      version: '1.0.0',
      description: 'A simple REST API that serves as a "Hello World" example',
    },
    servers: [
      {
        url: `http://localhost:${process.env.PORT || 3000}`,
        description: 'Local development server',
      },
    ],
  },
  apis: ['./src/routes/*.js', './docs/openapi.yaml'],
};

const swaggerSpec = swaggerJsdoc(swaggerOptions);
app.use('/docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec));
app.get('/docs.json', (req, res) => {
  res.setHeader('Content-Type', 'application/json');
  res.send(swaggerSpec);
});
```

### 3. Add Root Redirect to Documentation
In the same app.js file, add this redirect BEFORE the line that mounts the main routes (`app.use('/', require('./routes'))`):

```javascript
// Redirect root to documentation
app.get('/', (req, res) => {
  res.redirect('/docs');
});
```

### 4. Verify JSDoc Comments
Check that all route files from Task 3 contain proper JSDoc Swagger comments. They should already be present in the format:

```javascript
/**
 * @swagger
 * /endpoint:
 *   method:
 *     summary: ...
 *     description: ...
 *     responses:
 *       200:
 *         description: ...
 */
```

If any are missing, refer to the route implementations from Task 3 for the correct format.

### 5. Order of Operations in app.js
Ensure the correct order in app.js:
1. Import statements
2. Create Express app
3. Correlation ID middleware
4. Other middleware (helmet, cors, etc.)
5. **Swagger documentation setup (NEW)**
6. **Root redirect (NEW)**
7. Main routes
8. Error handling
9. 404 handler

## Testing Your Implementation

After making all changes:

1. **Start the server:**
   ```bash
   npm run dev
   ```

2. **Test root redirect:**
   ```bash
   curl -I http://localhost:3000/
   # Should show: Location: /docs
   ```

3. **Access documentation UI:**
   - Open browser to http://localhost:3000
   - Should redirect to Swagger UI
   - All endpoints should be visible

4. **Test JSON endpoint:**
   ```bash
   curl http://localhost:3000/docs.json | jq .info
   # Should show API title and version
   ```

5. **Verify interactive features:**
   - Click on any endpoint in Swagger UI
   - Click "Try it out"
   - Fill in parameters if needed
   - Click "Execute"
   - Should see actual API response

## Common Issues and Solutions

### Issue 1: "Cannot read property 'serve' of undefined"
**Solution:** Ensure swagger-ui-express is properly imported at the top of app.js

### Issue 2: No endpoints showing in documentation
**Solution:** Check that the `apis` array in swaggerOptions includes the correct path patterns

### Issue 3: Documentation shows at /docs but root doesn't redirect
**Solution:** Ensure the root redirect is placed before the main routes are mounted

### Issue 4: Server URL in docs doesn't match actual port
**Solution:** Verify the server URL uses `process.env.PORT || 3000` dynamically

## Expected Results
- Accessing http://localhost:3000 redirects to /docs
- Swagger UI displays with "Hello World API" title
- All 5 endpoints are documented (health, hello, hello/:name, echo, info)
- Each endpoint shows request/response details
- "Try it out" functionality works for all endpoints
- /docs.json returns valid OpenAPI 3.0 specification

## Validation Checklist
- [ ] docs/openapi.yaml file created with base specification
- [ ] Swagger middleware added to app.js
- [ ] Root redirect implemented
- [ ] All endpoints appear in documentation
- [ ] Interactive testing works
- [ ] JSON specification endpoint works
- [ ] No console errors when loading documentation

Complete this task by implementing all components exactly as specified. The documentation should be fully functional and provide a great developer experience for API consumers.