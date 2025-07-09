# Autonomous Prompt: Implement User Routes

## Task Context
You are an AI assistant tasked with implementing user management routes for an Express.js TypeScript application. This includes creating RESTful endpoints for user retrieval and creation with in-memory storage.

## Objective
Create comprehensive user management endpoints with proper validation, error handling, and TypeScript type safety.

## Required Actions

### 1. Install UUID Package
First, install the UUID package for generating unique user IDs:

```bash
npm install uuid
npm install --save-dev @types/uuid
```

### 2. Create User Routes File
Create `src/routes/users.ts` with the following implementation:

```typescript
import { Router, Request, Response } from 'express';
import { User, CreateUserRequest, UpdateUserRequest, UserResponse, userToResponse, isValidCreateUserRequest, isValidEmail } from '../types/user';
import { v4 as uuidv4 } from 'uuid';

const router = Router();

// In-memory storage for users
let users: User[] = [
  {
    id: '123e4567-e89b-12d3-a456-426614174000',
    name: 'John Doe',
    email: 'john@example.com',
    createdAt: new Date('2023-07-09T15:30:00.000Z')
  },
  {
    id: '987fcdeb-51a2-43d1-9b12-345678901234',
    name: 'Jane Smith',
    email: 'jane@example.com',
    createdAt: new Date('2023-07-09T15:31:00.000Z')
  }
];

/**
 * GET /users - Retrieve all users
 */
router.get('/users', (req: Request, res: Response) => {
  try {
    const userResponses: UserResponse[] = users.map(userToResponse);
    res.json(userResponses);
  } catch (error) {
    res.status(500).json({ 
      error: 'Internal server error while retrieving users' 
    });
  }
});

/**
 * GET /users/:id - Retrieve a specific user
 */
router.get('/users/:id', (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    const user = users.find(u => u.id === id);
    
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }
    
    res.json(userToResponse(user));
  } catch (error) {
    res.status(500).json({ 
      error: 'Internal server error while retrieving user' 
    });
  }
});

/**
 * POST /users - Create a new user
 */
router.post('/users', (req: Request, res: Response) => {
  try {
    const createRequest: CreateUserRequest = req.body;
    
    // Validate request body
    if (!isValidCreateUserRequest(createRequest)) {
      return res.status(400).json({ 
        error: 'Invalid user data',
        details: {
          name: 'Name is required and must be non-empty',
          email: 'Email is required and must be valid'
        }
      });
    }
    
    const { name, email } = createRequest;
    
    // Check for duplicate email
    const existingUser = users.find(user => user.email.toLowerCase() === email.toLowerCase());
    if (existingUser) {
      return res.status(409).json({ 
        error: 'Email already exists',
        details: 'A user with this email address already exists'
      });
    }
    
    // Create new user
    const newUser: User = {
      id: uuidv4(),
      name: name.trim(),
      email: email.toLowerCase().trim(),
      createdAt: new Date()
    };
    
    users.push(newUser);
    
    res.status(201).json(userToResponse(newUser));
  } catch (error) {
    res.status(500).json({ 
      error: 'Internal server error while creating user' 
    });
  }
});

/**
 * PUT /users/:id - Update a user
 */
router.put('/users/:id', (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    const updateRequest: UpdateUserRequest = req.body;
    
    const userIndex = users.findIndex(u => u.id === id);
    if (userIndex === -1) {
      return res.status(404).json({ error: 'User not found' });
    }
    
    const user = users[userIndex];
    
    // Validate update data
    if (updateRequest.name !== undefined) {
      if (!updateRequest.name || updateRequest.name.trim().length === 0) {
        return res.status(400).json({ 
          error: 'Invalid name',
          details: 'Name must be non-empty'
        });
      }
      user.name = updateRequest.name.trim();
    }
    
    if (updateRequest.email !== undefined) {
      if (!isValidEmail(updateRequest.email)) {
        return res.status(400).json({ 
          error: 'Invalid email',
          details: 'Email must be in valid format'
        });
      }
      
      // Check for duplicate email (excluding current user)
      const existingUser = users.find(u => 
        u.id !== id && u.email.toLowerCase() === updateRequest.email!.toLowerCase()
      );
      if (existingUser) {
        return res.status(409).json({ 
          error: 'Email already exists',
          details: 'Another user already has this email address'
        });
      }
      
      user.email = updateRequest.email.toLowerCase().trim();
    }
    
    users[userIndex] = user;
    
    res.json(userToResponse(user));
  } catch (error) {
    res.status(500).json({ 
      error: 'Internal server error while updating user' 
    });
  }
});

/**
 * DELETE /users/:id - Delete a user
 */
router.delete('/users/:id', (req: Request, res: Response) => {
  try {
    const { id } = req.params;
    const userIndex = users.findIndex(u => u.id === id);
    
    if (userIndex === -1) {
      return res.status(404).json({ error: 'User not found' });
    }
    
    users.splice(userIndex, 1);
    
    res.status(204).send();
  } catch (error) {
    res.status(500).json({ 
      error: 'Internal server error while deleting user' 
    });
  }
});

export default router;
```

### 3. Update Main Application
Modify `src/index.ts` to include the user routes:

```typescript
import express from 'express';
import healthRoutes from './routes/health';
import userRoutes from './routes/users';

const app = express();
const port = process.env.PORT || 3000;

// Middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Routes
app.use('/api', healthRoutes);
app.use('/api', userRoutes);

// Default route
app.get('/', (req, res) => {
  res.json({ 
    message: 'Express TypeScript API Server',
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV || 'development',
    endpoints: {
      health: '/api/health',
      users: '/api/users'
    }
  });
});

// Error handling middleware
app.use((err: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
  console.error(err.stack);
  res.status(500).json({ 
    error: 'Internal server error',
    message: 'Something went wrong!'
  });
});

// Start server
app.listen(port, () => {
  console.log(`⚡️ Server is running at http://localhost:${port}`);
  console.log(`📊 Health check: http://localhost:${port}/api/health`);
  console.log(`👥 Users API: http://localhost:${port}/api/users`);
});

export default app;
```

### 4. Add Validation Middleware (Optional)
Create `src/middleware/validation.ts` for enhanced validation:

```typescript
import { Request, Response, NextFunction } from 'express';
import { isValidCreateUserRequest, isValidEmail } from '../types/user';

export function validateCreateUser(req: Request, res: Response, next: NextFunction) {
  if (!isValidCreateUserRequest(req.body)) {
    return res.status(400).json({
      error: 'Invalid user data',
      details: {
        name: 'Name is required and must be non-empty',
        email: 'Email is required and must be valid'
      }
    });
  }
  next();
}

export function validateUserExists(req: Request, res: Response, next: NextFunction) {
  const { id } = req.params;
  
  if (!id || id.trim().length === 0) {
    return res.status(400).json({
      error: 'Invalid user ID',
      details: 'User ID is required'
    });
  }
  
  next();
}
```

## Validation Steps
1. **Build Test**: Run `npm run build` to ensure TypeScript compiles
2. **Server Start**: Run `npm run dev` to start the development server
3. **GET Users Test**: Test retrieving all users
4. **POST User Test**: Test creating a new user
5. **Validation Test**: Test with invalid data
6. **Duplicate Email Test**: Test duplicate email prevention

## Testing Commands

### Basic API Testing
```bash
# Start the server
npm run dev

# Test GET /users
curl -X GET http://localhost:3000/api/users

# Test POST /users
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Johnson","email":"alice@example.com"}'

# Test GET /users/:id
curl -X GET http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000

# Test PUT /users/:id
curl -X PUT http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{"name":"John Updated"}'

# Test DELETE /users/:id
curl -X DELETE http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000
```

### Validation Testing
```bash
# Test missing name
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com"}'

# Test invalid email
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test User","email":"invalid-email"}'

# Test duplicate email
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Duplicate User","email":"john@example.com"}'
```

### Load Testing
```bash
# Test concurrent requests
for i in {1..10}; do
  curl -X POST http://localhost:3000/api/users \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"User $i\",\"email\":\"user$i@example.com\"}" &
done
wait
```

## Success Criteria
- [ ] GET /users endpoint returns all users
- [ ] POST /users endpoint creates new users
- [ ] PUT /users/:id endpoint updates existing users
- [ ] DELETE /users/:id endpoint removes users
- [ ] Proper validation for all input data
- [ ] Duplicate email prevention
- [ ] Appropriate HTTP status codes
- [ ] TypeScript type safety throughout
- [ ] Error handling for all scenarios
- [ ] Data persistence during application lifecycle

## Error Handling Scenarios
- **400 Bad Request**: Invalid input data
- **404 Not Found**: User not found
- **409 Conflict**: Duplicate email
- **500 Internal Server Error**: Server errors

## Response Format Examples

### GET /users
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "John Doe",
    "email": "john@example.com",
    "createdAt": "2023-07-09T15:30:00.000Z"
  }
]
```

### POST /users (Success)
```json
{
  "id": "456e7890-a12b-34c5-d678-901234567890",
  "name": "Alice Johnson",
  "email": "alice@example.com",
  "createdAt": "2023-07-09T15:32:00.000Z"
}
```

### Error Response
```json
{
  "error": "Email already exists",
  "details": "A user with this email address already exists"
}
```

## Performance Considerations
- In-memory storage is suitable for development/testing
- Consider pagination for large user lists
- Implement rate limiting for production
- Add caching for frequently accessed data

## Security Considerations
- Input validation and sanitization
- Email case normalization
- SQL injection prevention (future database integration)
- Rate limiting for user creation

## Future Enhancements
- Database integration (PostgreSQL, MongoDB)
- Authentication and authorization
- User search and filtering
- Bulk operations
- Audit logging
- Password management

## Final Deliverables
- [ ] `src/routes/users.ts` with all CRUD endpoints
- [ ] Updated `src/index.ts` with route integration
- [ ] UUID package installed and configured
- [ ] Comprehensive validation and error handling
- [ ] TypeScript type safety throughout
- [ ] In-memory storage implementation
- [ ] Proper HTTP status codes and responses