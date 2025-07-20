# Task 5: Implement API Routes - Autonomous Prompt

You are an AI agent tasked with implementing all API routes for the Simple Todo REST API. Your goal is to create route handlers that connect the Express application to the controllers with proper validation middleware integration.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- Task 3 (Express/Middleware) is complete - app and validation available
- Task 4 (Controllers) is complete - controller functions ready
- Validation middleware and asyncHandler are available

## Your Mission
Create all API routes with proper RESTful structure, integrate validation middleware, and connect routes to controllers. Update the Express application to mount these routes and ensure all endpoints are accessible.

## Required Actions

### 1. Create Todo Routes
Create `src/routes/todoRoutes.js`:
- Import express Router
- Import todoController from controllers
- Import todoValidation from middleware
- Import asyncHandler from errorHandler

Define routes:
- `GET /` - List todos (with validation.list)
- `POST /` - Create todo (with validation.create)
- `GET /stats` - Get statistics (no validation)
- `GET /:id` - Get single todo (with validation.getOne)
- `PUT /:id` - Update todo (with validation.update)
- `DELETE /:id` - Delete todo (with validation.delete)

**Important**: Place `/stats` route BEFORE `/:id` to avoid conflicts

### 2. Create Health Routes
Create `src/routes/healthRoutes.js`:
- Basic health check at `GET /`
- Check database connectivity
- Return status, timestamp, environment, version
- Add detailed health check at `GET /detailed`
- Include memory usage and system info

### 3. Create Main Routes Index
Create `src/routes/index.js`:
- Create main router
- Mount todo routes at `/todos`
- Mount health routes at `/health`
- Add root endpoint at `/` with API info
- List all available endpoints

### 4. Update Application
Update `src/app.js`:
- Import routes from './routes'
- Mount routes at `/api`
- Ensure routes are added before 404 handler
- Keep error handler as last middleware

Final route structure:
```
/api               - API information
/api/todos         - Todo endpoints
/api/health        - Health checks
/api-docs          - Swagger (if implemented)
```

### 5. Route Implementation Details

For each route:
- Use appropriate HTTP method
- Apply validation middleware
- Wrap controller with asyncHandler
- Add descriptive comments

Example pattern:
```javascript
router.get(
  '/',
  todoValidation.list,
  asyncHandler(todoController.getAllTodos)
);
```

## Validation Criteria
- All endpoints are accessible
- Validation middleware executes correctly
- Routes delegate to correct controllers
- Async errors are caught properly
- Route parameters work correctly
- Query parameters are passed through
- Health endpoints provide accurate info
- API root lists all endpoints

## Important Notes
- Use express.Router() for modular routing
- Apply validation before controller
- Use asyncHandler for all async routes
- Maintain RESTful URL patterns
- Order matters - specific routes before params
- Don't forget to export routers
- Update app.js to use the routes

## Testing the Routes
After implementation, verify:
1. GET /api returns endpoint list
2. GET /api/todos returns todo list
3. POST /api/todos creates new todo
4. GET /api/todos/1 returns specific todo
5. PUT /api/todos/1 updates todo
6. DELETE /api/todos/1 deletes todo
7. GET /api/todos/stats returns statistics
8. GET /api/health returns health status
9. Invalid routes return 404
10. Validation errors return 400

## Expected Outcome
Complete route implementation with:
- All API endpoints functioning
- Proper route organization
- Validation middleware integrated
- Clean route/controller separation
- Health monitoring endpoints
- Ready for API documentation in Task 6

Execute all steps and test each endpoint to ensure proper routing and middleware execution.