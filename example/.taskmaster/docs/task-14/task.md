# Task 14: Implement User Routes

## Overview
Create GET /users and POST /users endpoints with in-memory storage to provide basic user management functionality for the Express TypeScript application.

## Description
This task involves implementing RESTful user endpoints that allow clients to retrieve all users and create new users. The implementation uses in-memory storage for simplicity and includes basic validation for user creation.

## Priority
High

## Dependencies
- Task 11: Initialize Express TypeScript Project (must be completed first)
- Task 12: Create User Type Definition (must be completed first)

## Implementation Steps

### 1. Create users route file
- Create `src/routes/users.ts` with Express router setup
- Set up in-memory storage array for users
- Import User types from type definitions
- Configure router with proper middleware

### 2. Implement GET /users handler
- Create handler to return all users from in-memory array
- Format response as JSON array
- Include proper error handling
- Add pagination support (optional)

### 3. Implement POST /users handler
- Create handler to add new users with validation
- Generate unique IDs for new users
- Validate required fields (name, email)
- Check for duplicate emails
- Add user to in-memory storage

### 4. Wire user routes to app
- Import user router in main Express app
- Mount user routes at appropriate path
- Ensure proper middleware order

## Implementation Details

### User Route Structure
```typescript
import { Router, Request, Response } from 'express';
import { User, CreateUserRequest, UserResponse, userToResponse } from '../types/user';
import { v4 as uuidv4 } from 'uuid';

const router = Router();

// In-memory storage
let users: User[] = [];

// GET /users - Retrieve all users
router.get('/users', (req: Request, res: Response) => {
  const userResponses: UserResponse[] = users.map(userToResponse);
  res.json(userResponses);
});

// POST /users - Create new user
router.post('/users', (req: Request, res: Response) => {
  const { name, email }: CreateUserRequest = req.body;
  
  // Validation
  if (!name || !email) {
    return res.status(400).json({ error: 'Name and email are required' });
  }
  
  // Check for duplicate email
  if (users.some(user => user.email === email)) {
    return res.status(409).json({ error: 'Email already exists' });
  }
  
  // Create new user
  const newUser: User = {
    id: uuidv4(),
    name,
    email,
    createdAt: new Date()
  };
  
  users.push(newUser);
  res.status(201).json(userToResponse(newUser));
});

export default router;
```

### Integration with Main App
```typescript
// In src/index.ts
import userRoutes from './routes/users';

app.use('/api', userRoutes);
```

### Request/Response Examples

#### GET /users
**Response (200 OK)**:
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "John Doe",
    "email": "john@example.com",
    "createdAt": "2023-07-09T15:30:00.000Z"
  },
  {
    "id": "987fcdeb-51a2-43d1-9b12-345678901234",
    "name": "Jane Smith",
    "email": "jane@example.com",
    "createdAt": "2023-07-09T15:31:00.000Z"
  }
]
```

#### POST /users
**Request Body**:
```json
{
  "name": "Alice Johnson",
  "email": "alice@example.com"
}
```

**Response (201 Created)**:
```json
{
  "id": "456e7890-a12b-34c5-d678-901234567890",
  "name": "Alice Johnson",
  "email": "alice@example.com",
  "createdAt": "2023-07-09T15:32:00.000Z"
}
```

## File Structure
```
src/
├── routes/
│   ├── users.ts
│   └── health.ts
├── types/
│   └── user.ts
└── index.ts (updated)
```

## Test Strategy
- Test both endpoints with curl/Postman
- Verify data persists between requests
- Test validation for missing fields
- Test duplicate email prevention
- Test response format and status codes
- Load test with multiple concurrent requests

## Expected Outcomes
- Functional GET /users endpoint returning all users
- Functional POST /users endpoint creating new users
- Proper validation and error handling
- In-memory data persistence during application lifecycle
- RESTful response codes and formats

## Common Issues
- **Memory persistence**: Data is lost on server restart
- **Validation**: Ensure all required fields are validated
- **Duplicate checking**: Prevent duplicate email addresses
- **Error handling**: Return appropriate HTTP status codes
- **Type safety**: Use proper TypeScript types throughout

## Enhanced Features (Optional)
- GET /users/:id endpoint for single user retrieval
- PUT /users/:id endpoint for user updates
- DELETE /users/:id endpoint for user deletion
- Query parameters for filtering and pagination
- Input sanitization and advanced validation
- Rate limiting for user creation

## Next Steps
After completion, this implementation provides:
- Basic user CRUD operations
- Foundation for authentication systems
- Data structure for future database integration
- RESTful API pattern for other resources