# Task 6: Implement API Documentation with Swagger - Autonomous Prompt

You are an AI agent tasked with adding comprehensive API documentation using OpenAPI/Swagger to a Simple Todo REST API. This will provide interactive documentation that developers can use to understand and test the API.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: Task 5 (Routes) must be completed
- **Tools**: swagger-jsdoc and swagger-ui-express (already installed)
- **Working Directory**: Project root (simple-api/)
- **Documentation URL**: Will be available at /api-docs
- **References**:
  - Architecture: .taskmaster/docs/architecture.md
  - API Endpoints: Defined in src/routes/

## Your Mission

Create comprehensive OpenAPI documentation for all API endpoints, including schemas, examples, and interactive Swagger UI. The documentation should be complete, accurate, and helpful for developers using the API.

## Detailed Implementation Steps

1. **Create Swagger Configuration** (`src/middleware/swagger.js`)
   - Import swaggerJsdoc and swaggerUi
   - Define OpenAPI 3.0.0 specification object:
     - API info (title, version, description)
     - Server configurations (development and production)
     - Component schemas for all data types:
       - Todo (complete schema)
       - TodoInput (for creation)
       - TodoUpdate (for updates)
       - Error (standard error format)
       - HealthStatus
       - TodoStats
   - Configure JSDoc options to scan route files
   - Export serve and setup functions

2. **Document Todo Routes** (Update `src/routes/todos.js`)
   Add JSDoc comments above each route:
   - GET /api/todos - Include query parameters documentation
   - GET /api/todos/stats - Statistics endpoint
   - GET /api/todos/:id - Path parameter documentation
   - POST /api/todos - Request body schema
   - PUT /api/todos/:id - Update schema
   - DELETE /api/todos/:id - No body required
   - Add @swagger tags for proper categorization

3. **Document Health Routes** (Update `src/routes/health.js`)
   - GET /api/health - Basic health response
   - GET /api/health/detailed - Detailed health with checks
   - Tag as "System" endpoints

4. **Document API Root** (Update `src/routes/index.js`)
   - GET /api - API information endpoint
   - Include available endpoints in response

5. **Integrate Swagger UI** (Update `src/app.js`)
   - Import swagger middleware
   - Mount Swagger UI at /api-docs
   - Ensure it's mounted before routes

## Schema Definitions

### Todo Schema
```yaml
Todo:
  type: object
  required: [title]
  properties:
    id: integer (auto-generated)
    title: string (1-200 chars)
    description: string (max 1000, nullable)
    completed: boolean (default false)
    createdAt: string (date-time)
    updatedAt: string (date-time)
```

### Error Schema
```yaml
Error:
  type: object
  properties:
    error:
      type: object
      properties:
        message: string
        code: string
        details: array (optional)
```

## JSDoc Comment Format

```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: Short description
 *     description: Detailed description
 *     tags: [Todos]
 *     parameters:
 *       - in: query
 *         name: paramName
 *         schema:
 *           type: string
 *         description: Parameter description
 *     responses:
 *       200:
 *         description: Success response
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Todo'
 */
```

## Important Documentation Guidelines

1. **Completeness**: Document all parameters, request bodies, and responses
2. **Examples**: Include realistic example values
3. **Validation**: Document constraints (min/max lengths, required fields)
4. **Error Responses**: Document all possible error scenarios
5. **Consistency**: Use consistent terminology and formatting

## Success Criteria
- ✅ Swagger UI accessible at /api-docs
- ✅ All endpoints documented with descriptions
- ✅ Request/response schemas defined
- ✅ Interactive "Try it out" functionality works
- ✅ Proper categorization with tags
- ✅ Example values provided
- ✅ All parameters documented
- ✅ Error responses documented

## Testing Your Documentation

1. Start the server and navigate to http://localhost:3000/api-docs
2. Verify all endpoints are listed and grouped correctly
3. Test "Try it out" functionality for each endpoint
4. Check that schemas display correctly
5. Ensure examples are helpful and realistic

## Common Pitfalls to Avoid
1. Forgetting to add @swagger tag to JSDoc comments
2. Incorrect $ref paths to schemas
3. Missing required parameters in documentation
4. Not restarting server after adding documentation
5. Inconsistent schema definitions

## Expected Result
When complete, developers should be able to:
- View all available endpoints
- Understand request/response formats
- Test API calls directly from documentation
- See example requests and responses
- Understand validation requirements

Remember: Good documentation is as important as good code. Make it comprehensive, clear, and helpful for anyone using your API.