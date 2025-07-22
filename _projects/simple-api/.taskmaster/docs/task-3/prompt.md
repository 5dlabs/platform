# Autonomous Task Prompt: Create Health and Welcome Endpoints

You are tasked with implementing health check and welcome endpoints for the Express API, following MVC patterns with separated routes and controllers.

## Context
- Express server is set up and running
- Need monitoring and discovery endpoints
- Follow separation of concerns with routes and controllers
- API should provide version info and health status

## Your Mission
Create well-structured endpoints for API discovery (/) and health monitoring (/health) with proper separation of concerns.

## Steps to Complete

1. **Create directory structure**
   - Set up routes directory with modular route files
   - Set up controllers directory for business logic
   - Keep routes thin, controllers handle logic

2. **Implement Welcome endpoint (GET /)**
   - Create welcome controller that reads package.json
   - Return API name, version, and current timestamp
   - Handle file reading errors gracefully

3. **Implement Health endpoint (GET /health)**
   - Create health controller with server status
   - Include server uptime using process.uptime()
   - Return status "ok" and timestamp

4. **Set up routing**
   - Create separate route files for each endpoint
   - Use Express Router for modular routes
   - Import and mount routes in main server file

5. **Integration and testing**
   - Wire routes into Express app
   - Test both endpoints return correct JSON
   - Verify proper HTTP status codes (200)
   - Ensure clean separation of concerns

## Expected Responses

### GET /
```json
{
  "message": "Welcome to Simple Express API",
  "version": "1.0.0",
  "timestamp": "2025-01-22T10:00:00.000Z"
}
```

### GET /health
```json
{
  "status": "ok",
  "uptime": 123.456,
  "timestamp": "2025-01-22T10:00:00.000Z"
}
```

## Success Criteria
- Endpoints respond with correct JSON structure
- Version dynamically read from package.json
- Uptime accurately reflects server runtime
- Clean code separation (routes vs controllers)
- No hardcoded values
- Proper error handling

## Technical Notes
- Use ES6 module imports
- Follow RESTful conventions
- Keep controllers pure functions when possible
- Use Express Router for modularity
- Return appropriate content-type headers