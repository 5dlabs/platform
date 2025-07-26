# Task 8: Create Main Server File

## Overview
This task implements the core Express.js server in `src/index.js`, establishing the foundation for the Hello World API. It creates a basic HTTP server with request logging middleware and proper startup configuration.

## Objectives
- Create the main Express.js server file
- Implement basic request logging middleware
- Configure server to listen on port 3000
- Add environment variable support for port configuration
- Include error handling for server startup

## Technical Approach

### 1. Server Initialization
- Import Express.js framework
- Create Express application instance
- Configure port with environment variable fallback

### 2. Middleware Implementation
- Create custom logging middleware
- Log timestamp, HTTP method, and URL for each request
- Ensure middleware properly calls next() for request pipeline

### 3. Server Startup
- Configure app.listen() with port binding
- Add startup confirmation logging
- Implement error handling for startup failures

### 4. Environment Configuration
- Support PORT environment variable
- Default to port 3000 if not specified
- Allow flexible deployment configurations

## Dependencies
- Task 6: Node.js project must be initialized
- Task 7: Express.js must be installed
- Express.js package available in node_modules

## Expected Outcomes
1. Functional Express.js server running on port 3000
2. All requests logged to console with timestamp
3. Environment variable support for port configuration
4. Graceful error handling for startup issues
5. Server ready for endpoint implementation

## Code Structure
```javascript
// Main components:
// 1. Express import and app creation
// 2. Port configuration with env support
// 3. Request logging middleware
// 4. Server startup with error handling
```

## Related Tasks
- Depends on: Task 7 (Install Express.js)
- Required for: Task 9 (Root Endpoint), Task 10 (Health Endpoint)
- Foundation for: All API endpoint implementations