# Simple Express API - Architecture

## Overview
A lightweight REST API built with Express.js providing user management capabilities with a focus on simplicity and maintainability.

## Architecture Components

### Application Layer
- **Express.js Server**: Main application server handling HTTP requests
- **Middleware Stack**: JSON parsing, error handling, request logging
- **Route Handlers**: Modular route definitions for different API endpoints

### API Structure
```
/                    - Welcome/info endpoint
/health             - Health check endpoint
/api/users          - User management endpoints
  GET /api/users    - List users
  POST /api/users   - Create user
```

### Data Layer
- **Mock Data**: In-memory user storage for initial implementation
- **Data Models**: Simple user object with id, name, email, createdAt

### Error Handling
- **Global Error Handler**: Centralized error processing
- **404 Handler**: Route not found handling
- **Validation**: Input validation with proper error responses

## File Structure
```
simple-api/
├── index.js          # Main server file
├── package.json      # Dependencies and scripts
└── node_modules/     # Dependencies
```

## Key Design Decisions
- **Stateless**: No session management, pure REST API
- **JSON Only**: All requests/responses in JSON format
- **Mock Data**: In-memory storage for simplicity
- **Express Middleware**: Standard middleware pattern for cross-cutting concerns