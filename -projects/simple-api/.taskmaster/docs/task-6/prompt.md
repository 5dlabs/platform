# Task 6: Implement API Documentation with Swagger - Autonomous Prompt

You are an AI agent tasked with implementing comprehensive API documentation using OpenAPI/Swagger for the Simple Todo REST API. Your goal is to create interactive documentation that accurately describes all endpoints, schemas, and responses.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- Task 5 (Routes) is complete - all endpoints are implemented
- Dependencies swagger-ui-express and swagger-jsdoc are installed

## Your Mission
Implement Swagger documentation that provides a complete OpenAPI 3.0 specification for the API, including interactive testing capabilities through Swagger UI. Document all endpoints, request/response schemas, and error formats.

## Required Actions

### 1. Create Swagger Configuration
Create `src/config/swagger.js`:
- Define OpenAPI 3.0 specification
- Set API title, version, description
- Configure servers (development/production)
- Define reusable components:
  - Schemas: Todo, TodoInput, TodoUpdate, Error, HealthCheck
  - Responses: NotFound, ValidationError, InternalError
- Set API paths to scan for annotations

### 2. Create Swagger Middleware
Create `src/middleware/swagger.js`:
- Import swagger-ui-express
- Import swagger config
- Configure Swagger UI options
- Add custom CSS to hide top bar
- Set custom site title
- Export serve and setup functions

### 3. Add JSDoc Annotations to Routes
Update `src/routes/todoRoutes.js` with @swagger comments:

For each endpoint, document:
- Summary and description
- Tags for grouping
- Parameters (path, query, body)
- Request body schema
- Response schemas and status codes
- Error responses

Example structure:
```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: Get all todos
 *     tags: [Todos]
 *     parameters: [...]
 *     responses: {...}
 */
```

### 4. Document Health Routes
Update `src/routes/healthRoutes.js`:
- Document basic health check
- Document detailed health check
- Include response schemas

### 5. Update Application
Update `src/app.js`:
- Import swagger middleware
- Mount Swagger UI at `/api-docs`
- Place before API routes

### 6. Schema Definitions

**Todo Schema**:
- id: integer (auto-generated)
- title: string (1-200 chars)
- description: string (max 1000)
- completed: boolean
- createdAt: datetime
- updatedAt: datetime

**TodoInput Schema**:
- title: required string
- description: optional string

**TodoUpdate Schema**:
- title: optional string
- description: optional string
- completed: optional boolean

**Error Schema**:
- error.message: string
- error.code: string
- error.details: array (optional)

### 7. Documentation Requirements
- All endpoints must be documented
- Include example values for all fields
- Document all possible error responses
- Group endpoints by tags (Todos, Health)
- Provide clear descriptions
- Include query parameter constraints

## Validation Criteria
- Swagger UI loads at /api-docs
- All endpoints appear in documentation
- Interactive testing works
- Schemas accurately reflect data models
- Examples match actual responses
- Error responses are documented
- OpenAPI spec validates correctly
- Try-it-out feature functions properly

## Important Notes
- Use OpenAPI 3.0 specification
- Place swagger annotations above routes
- Include realistic example data
- Document all status codes
- Ensure schemas match actual models
- Use proper types (integer, not number)
- Include format for datetime fields

## Testing the Documentation
After implementation, verify:
1. Navigate to http://localhost:3000/api-docs
2. All endpoints are listed
3. Schemas show correct properties
4. Try-it-out works for each endpoint
5. Examples are realistic
6. Parameters show constraints
7. Response codes are documented
8. Models can be viewed separately

## Expected Outcome
Complete API documentation with:
- Interactive Swagger UI interface
- Full OpenAPI 3.0 specification
- All endpoints documented
- Accurate schema definitions
- Working try-it-out functionality
- Professional documentation appearance
- Ready for developer consumption

Execute all steps and ensure the documentation accurately reflects the implemented API functionality.