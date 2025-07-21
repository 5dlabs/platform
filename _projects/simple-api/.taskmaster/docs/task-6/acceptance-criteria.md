# Task 6: Implement API Documentation with Swagger - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 6: Implement API Documentation with Swagger. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Swagger Configuration ✓

**Given** the API needs documentation
**When** checking src/config/swagger.js
**Then** it must include:
- OpenAPI 3.0.0 specification
- API title, version, and description
- Contact and license information
- Development and production server URLs
- Complete component schemas (Todo, TodoInput, TodoUpdate, Error, HealthStatus)
- Reusable response definitions

**Test**:
```bash
node -e "const swagger = require('./src/config/swagger'); console.log(swagger.info ? 'Config OK' : 'Failed')"
```

### 2. Swagger UI Accessibility ✓

**Given** the server is running
**When** navigating to /api-docs
**Then**:
- Swagger UI loads successfully
- No console errors
- UI is interactive and responsive
- Custom styling applied
- Documentation title shows correctly

**Test**:
```bash
# Start server
npm run dev
# Open http://localhost:3000/api-docs in browser
# Should see "Simple Todo REST API" documentation
```

### 3. Endpoint Documentation Coverage ✓

**Given** all API endpoints exist
**When** viewing Swagger UI
**Then** all endpoints must be documented:

| Endpoint | Method | Tag | Documentation Required |
|----------|--------|-----|----------------------|
| /api/todos | GET | Todos | Summary, parameters, responses |
| /api/todos | POST | Todos | Summary, request body, responses |
| /api/todos/:id | GET | Todos | Summary, path param, responses |
| /api/todos/:id | PUT | Todos | Summary, path param, body, responses |
| /api/todos/:id | DELETE | Todos | Summary, path param, responses |
| /api/todos/stats/summary | GET | Todos | Summary, responses |
| /api/health | GET | Health | Summary, responses |
| /api/health/ready | GET | Health | Summary, responses |
| /api/health/live | GET | Health | Summary, responses |

**Test**: Count endpoints in Swagger UI - should be at least 9

### 4. Schema Definitions ✓

**Given** the need for consistent data models
**When** checking component schemas
**Then** these schemas must be defined:

```yaml
Todo:
  - id (integer, example: 1)
  - title (string, required, 1-200 chars)
  - description (string, nullable, max 1000)
  - completed (boolean, default: false)
  - createdAt (date-time)
  - updatedAt (date-time)

TodoInput:
  - title (string, required)
  - description (string, optional)

TodoUpdate:
  - title (string, optional)
  - description (string, optional)
  - completed (boolean, optional)

Error:
  - error (string)
  - message (string)
  - details (array, optional)
  - requestId (string)

HealthStatus:
  - status (enum: healthy/unhealthy)
  - timestamp (date-time)
  - service (string)
  - version (string)
  - database (enum: connected/disconnected)
```

**Test**: In Swagger UI, check "Schemas" section at bottom

### 5. Request Documentation ✓

**Given** endpoints accept parameters
**When** viewing endpoint details
**Then** all parameters must be documented:

- **Query parameters**: type, description, constraints, examples
- **Path parameters**: type, description, required flag
- **Request bodies**: schema reference, examples, required fields

**Test Cases**:
```yaml
GET /api/todos:
  - completed: boolean, optional, example: true
  - limit: integer, 1-100, example: 10
  - offset: integer, >= 0, example: 0

POST /api/todos:
  - body: TodoInput schema, example provided

PUT /api/todos/:id:
  - id: integer, required, example: 1
  - body: TodoUpdate schema, examples provided
```

### 6. Response Documentation ✓

**Given** endpoints return different responses
**When** viewing endpoint responses
**Then** all response codes must be documented:

| Status | Description | Schema |
|--------|-------------|---------|
| 200 | Success | Endpoint-specific |
| 201 | Created | With created resource |
| 204 | No Content | Empty (for DELETE) |
| 400 | Validation Error | Error schema |
| 404 | Not Found | Error schema |
| 500 | Server Error | Error schema |
| 503 | Service Unavailable | For health |

**Test**: Each endpoint shows all possible responses

### 7. Interactive Testing ✓

**Given** Swagger UI's "Try it out" feature
**When** testing any endpoint
**Then**:
- "Try it out" button is clickable
- Form fields appear for parameters
- "Execute" sends actual request
- Response is displayed with:
  - Status code
  - Response body
  - Response headers
  - Curl command

**Test**:
```
1. Click "Try it out" on GET /api/todos
2. Set completed=true, limit=5
3. Click "Execute"
4. Verify response shows filtered todos
```

### 8. Example Values ✓

**Given** the need for clarity
**When** viewing schemas and parameters
**Then** examples must be provided:
- Realistic todo titles and descriptions
- Valid parameter values
- Complete response examples
- Error response examples

**Test**: Check that example values are realistic and valid

### 9. Tag Organization ✓

**Given** multiple endpoint groups
**When** viewing Swagger UI
**Then** endpoints must be organized by tags:
- **Todos**: All todo CRUD operations
- **Health**: Health monitoring endpoints
- **General**: API information (optional)

**Test**: Verify endpoints are grouped logically

### 10. Middleware Integration ✓

**Given** Swagger middleware
**When** checking src/app.js
**Then**:
- Swagger middleware imported correctly
- Mounted at /api-docs path
- Mounted BEFORE API routes
- Root endpoint includes documentation URL

**Test**:
```bash
curl http://localhost:3000/
# Response should include "documentation": "http://localhost:3000/api-docs"
```

## OpenAPI Validation

**Given** the OpenAPI specification
**When** validating the spec
**Then** it must be valid OpenAPI 3.0:

```bash
# Install validator
npm install -g @apidevtools/swagger-cli

# Validate (may need to export specs first)
swagger-cli validate <spec-file>
```

## User Experience Criteria

### Documentation Quality
- Clear, concise descriptions
- No technical jargon in summaries
- Helpful descriptions for parameters
- Meaningful example values

### UI Customization
- Company/project branding (if applicable)
- Hidden unnecessary UI elements
- Persistent authorization (if added later)
- Request duration display enabled

### Consistency
- Uniform description style
- Consistent parameter naming
- Matching schemas across endpoints
- Aligned with actual API behavior

## Test Scenarios

### Scenario 1: Full Documentation Review
1. Open /api-docs
2. Expand each endpoint
3. Verify all information present
4. Check examples make sense
5. Verify schemas are complete

### Scenario 2: Interactive Testing
1. Create a todo using Swagger UI
2. List todos with filters
3. Update the created todo
4. Delete the todo
5. Check health status

### Scenario 3: Error Documentation
1. Try invalid requests
2. Verify error responses match documentation
3. Check validation error details
4. Confirm error schema usage

## Definition of Done

- [ ] Swagger configuration created with OpenAPI 3.0
- [ ] All endpoints documented with descriptions
- [ ] Request parameters documented with types and examples
- [ ] Response schemas defined for all status codes
- [ ] Component schemas created and referenced
- [ ] Interactive testing functional in Swagger UI
- [ ] Examples provided for all operations
- [ ] Tags organize endpoints logically
- [ ] Swagger UI accessible at /api-docs
- [ ] Documentation matches actual API behavior
- [ ] No broken schema references
- [ ] Custom UI options configured

## Performance Criteria

- Swagger UI loads in < 3 seconds
- No impact on API performance
- Documentation generated at startup only

## Security Criteria

- No sensitive data in examples
- Production server URL uses HTTPS
- No authentication tokens in documentation
- Error messages don't reveal internals

## Notes

- Documentation is generated from JSDoc comments
- Changes to API should be reflected in docs
- Keep examples realistic but not real data
- Documentation serves as API contract
- Consider versioning strategy for future