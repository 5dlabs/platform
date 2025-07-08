# User API Service - Design Specification

## Overview
A lightweight Express.js REST API service for basic user management operations. This service demonstrates proper Express.js patterns with TypeScript integration.

## Architecture

### Technology Stack
- **Runtime**: Node.js 20+
- **Framework**: Express.js 4.x
- **Language**: TypeScript 5.x
- **Storage**: In-memory (array-based for MVP)

### Project Structure
```
user-api/
├── src/
│   ├── index.ts          # Application entry point
│   ├── routes/
│   │   ├── health.ts     # Health check endpoint
│   │   └── users.ts      # User management endpoints
│   ├── middleware/
│   │   └── error.ts      # Global error handler
│   └── types/
│       └── user.ts       # User type definition
├── package.json
├── tsconfig.json
└── README.md
```

## API Endpoints

### Health Check
- **GET /health**
  - Response: `{ "status": "ok", "timestamp": "2025-07-08T..." }`
  - Status: 200 OK

### User Management
- **GET /users**
  - Response: Array of User objects
  - Status: 200 OK

- **POST /users**
  - Request Body: `{ "name": "string", "email": "string" }`
  - Response: Created User object with generated ID
  - Status: 201 Created
  - Validation: name and email are required

## Data Model
```typescript
interface User {
  id: string;        // UUID v4
  name: string;      // Required, non-empty
  email: string;     // Required, basic email format
  createdAt: Date;   // Auto-generated
}
```

## Implementation Details

### Error Handling
- Global error middleware catches all errors
- Consistent error response format
- Appropriate HTTP status codes
- Input validation errors return 400

### In-Memory Storage
- Simple array to store users
- Data persists during server runtime
- Resets on server restart
- Suitable for demo/development

## Development Workflow
1. Initialize TypeScript project
2. Set up Express server
3. Implement type definitions
4. Create route handlers
5. Add error handling
6. Write documentation