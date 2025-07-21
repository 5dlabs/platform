# Task 6: Implement API Documentation with Swagger - Autonomous Prompt

You are tasked with adding comprehensive interactive API documentation using OpenAPI/Swagger to a Simple Todo REST API. The documentation should be auto-generated from code comments and served through Swagger UI.

## Your Mission

Create complete API documentation that allows developers to understand, explore, and test all API endpoints through an interactive web interface. Use OpenAPI 3.0 specification with Swagger UI.

## Required Actions

1. **Create Swagger Configuration (`src/config/swagger.js`)**
   
   Set up OpenAPI specification with:
   - API title, version, description
   - Server URLs (development and production)
   - Component schemas for:
     - Todo (full model)
     - TodoInput (creation)
     - TodoUpdate (updates)
     - Error (error responses)
     - HealthStatus (health checks)
   - Common responses (NotFound, ValidationError, InternalError)

2. **Document Todo Endpoints (`src/swagger/todos.js`)**
   
   Add JSDoc comments for each endpoint:
   - GET /api/todos - List with query parameters
   - POST /api/todos - Create with request body
   - GET /api/todos/:id - Get by ID
   - PUT /api/todos/:id - Update with request body
   - DELETE /api/todos/:id - Delete by ID
   - GET /api/todos/stats/summary - Statistics
   
   For each endpoint document:
   - Summary and description
   - Parameters (path, query, body)
   - All possible responses
   - Example values

3. **Document Health Endpoints (`src/swagger/health.js`)**
   
   Document health check endpoints:
   - GET /api/health - Main health check
   - GET /api/health/ready - Readiness probe
   - GET /api/health/live - Liveness probe

4. **Create Swagger Middleware (`src/middleware/swagger.js`)**
   
   Configure Swagger UI:
   - Import swagger-ui-express
   - Set up custom CSS for better appearance
   - Configure UI options (persistence, filters, etc.)
   - Export serve and setup functions

5. **Integrate into Express App**
   
   Update `src/app.js`:
   - Import Swagger middleware
   - Mount at `/api-docs` BEFORE other routes
   - Update root endpoint to include documentation URL

## OpenAPI Schema Requirements

### Todo Schema
```yaml
Todo:
  type: object
  required: [title]
  properties:
    id: integer
    title: string (1-200 chars)
    description: string (max 1000 chars, nullable)
    completed: boolean (default: false)
    createdAt: date-time
    updatedAt: date-time
```

### Error Schema
```yaml
Error:
  type: object
  properties:
    error: string
    message: string
    details: array (for validation errors)
    requestId: string
```

### Response Examples

Include realistic examples for:
- Successful responses
- Validation errors (400)
- Not found errors (404)
- Server errors (500)

## Documentation Standards

### Endpoint Documentation
```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: Short description
 *     description: Detailed description
 *     tags: [Todos]
 *     parameters: [...]
 *     responses:
 *       200:
 *         description: Success
 *         content: ...
 *       400:
 *         $ref: '#/components/responses/ValidationError'
 */
```

### Tag Groups
- **Todos**: All todo CRUD operations
- **Health**: Health monitoring endpoints
- **General**: API information

## Success Verification

Your documentation should:

- [ ] Be accessible at http://localhost:3000/api-docs
- [ ] Show all API endpoints organized by tags
- [ ] Allow interactive testing of endpoints
- [ ] Display request/response schemas
- [ ] Show example values for all fields
- [ ] Include all error responses
- [ ] Work with "Try it out" feature
- [ ] Be OpenAPI 3.0 compliant

## Testing the Documentation

1. **Start the server**:
   ```bash
   npm run dev
   ```

2. **Open Swagger UI**:
   - Navigate to http://localhost:3000/api-docs
   - Should see interactive documentation

3. **Test each endpoint**:
   - Click "Try it out"
   - Fill in parameters
   - Execute request
   - Verify response matches documentation

4. **Verify completeness**:
   - All 8 endpoints documented
   - All parameters explained
   - All responses covered
   - Examples provided

## Important Notes

- Place Swagger middleware BEFORE route handlers
- Use JSDoc @swagger comments for documentation
- Reference shared schemas with $ref
- Include realistic example values
- Document all possible error responses
- Group endpoints logically with tags
- Ensure consistency between docs and actual API

## Common Pitfalls to Avoid

1. **Wrong middleware order**: Swagger must be mounted before routes
2. **Missing file paths**: Ensure apis array includes all documentation files
3. **Schema mismatches**: Component names must match exactly
4. **Invalid YAML**: Use proper indentation in JSDoc comments
5. **Missing examples**: Provide examples for better usability

## Quality Checklist

- [ ] Clear, concise descriptions
- [ ] All parameters documented
- [ ] All responses documented
- [ ] Realistic examples
- [ ] Proper error documentation
- [ ] Interactive testing works
- [ ] No broken references
- [ ] Consistent formatting

Once complete, developers will have comprehensive, interactive documentation to understand and test your API without reading the code.