# Autonomous Task Prompt: Develop User Management Endpoints with Mock Data

You are tasked with implementing user management endpoints with in-memory storage and proper validation for the Express API.

## Context
- Basic Express server with health/welcome endpoints exists
- Need to add user CRUD operations
- Use mock data (in-memory) for storage
- Implement proper input validation

## Your Mission
Create RESTful endpoints for user management with validation and mock data persistence.

## Steps to Complete

1. **Set up data layer**
   - Create `/src/data/users.js` for mock storage
   - Initialize with 2 sample users
   - Implement getUsers and createUser functions
   - Manage auto-incrementing IDs

2. **Install dependencies**
   - Add validator@13 for email validation
   - Update package.json

3. **Create user controller**
   - Implement listUsers for GET /api/users
   - Implement addUser for POST /api/users
   - Add input validation:
     - Required fields: name, email
     - Email format validation
   - Return proper error responses

4. **Set up routes**
   - Create `/src/routes/users.js`
   - Define GET and POST routes
   - Connect to controller methods

5. **Integration**
   - Import and mount user routes in main server
   - Test all endpoints thoroughly
   - Verify data persistence during runtime

## API Specifications

### GET /api/users
Response: 200 OK
```json
[
  {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "createdAt": "2025-01-01T00:00:00.000Z"
  }
]
```

### POST /api/users
Request:
```json
{
  "name": "New User",
  "email": "new@example.com"
}
```

Response: 201 Created
```json
{
  "id": 3,
  "name": "New User",
  "email": "new@example.com",
  "createdAt": "2025-01-22T10:00:00.000Z"
}
```

### Validation Errors
Response: 400 Bad Request
```json
{
  "error": "Bad Request",
  "message": "Name and email are required"
}
```

## Success Criteria
- All endpoints work as specified
- Validation prevents invalid data
- IDs auto-increment correctly
- Timestamps use ISO 8601 format
- Data persists during server session
- Proper HTTP status codes used

## Technical Requirements
- Use validator library for email validation
- Keep data layer separate from controllers
- Follow RESTful conventions
- Maintain clean code structure
- Handle edge cases gracefully