# User API Service PRD

## Overview
A simple Express.js REST API for basic user management operations.

## Goals
- Create a minimal but functional user management API
- Demonstrate proper Express.js patterns
- Include basic error handling and validation

## Technical Requirements

### API Endpoints
1. `GET /health` - Health check endpoint
   - Returns: `{ status: "ok", timestamp: "ISO-8601 date" }`

2. `GET /users` - List all users
   - Returns: Array of user objects
   - In-memory storage is fine (no database required)

3. `POST /users` - Create a new user
   - Body: `{ name: string, email: string }`
   - Returns: Created user with generated ID
   - Basic validation: name and email required

### Technical Stack
- Node.js with Express.js
- TypeScript
- Basic error handling middleware
- Input validation (can use express-validator or manual)
- Proper HTTP status codes

### Project Structure
```
user-api/
├── src/
│   ├── index.ts        # Entry point
│   ├── routes/
│   │   ├── health.ts   # Health check route
│   │   └── users.ts    # User routes
│   ├── middleware/
│   │   └── error.ts    # Error handling
│   └── types/
│       └── user.ts     # User type definition
├── package.json
├── tsconfig.json
└── README.md
```

## Success Criteria
- [ ] Express server runs on port 3000
- [ ] All three endpoints work correctly
- [ ] Proper error handling for invalid requests
- [ ] TypeScript types for User
- [ ] Basic README with API documentation