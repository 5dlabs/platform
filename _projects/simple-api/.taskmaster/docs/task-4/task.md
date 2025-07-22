# Task 4: Develop User Management Endpoints with Mock Data

## Overview
Implement GET `/api/users` and POST `/api/users` endpoints using in-memory mock data and input validation. This provides core CRUD functionality for user management.

## Task Details
- **Priority**: High
- **Dependencies**: Task 3 (Create Health and Welcome Endpoints)
- **Status**: Pending

## Implementation Guide

### 1. Create Mock Data Store
`/src/data/users.js`:
```javascript
// In-memory user storage
let users = [
  {
    id: 1,
    name: 'John Doe',
    email: 'john@example.com',
    createdAt: '2025-01-01T00:00:00.000Z'
  },
  {
    id: 2,
    name: 'Jane Smith',
    email: 'jane@example.com',
    createdAt: '2025-01-01T00:00:00.000Z'
  }
];

let nextId = 3;

export const getUsers = () => users;

export const createUser = (userData) => {
  const newUser = {
    id: nextId++,
    ...userData,
    createdAt: new Date().toISOString()
  };
  users.push(newUser);
  return newUser;
};
```

### 2. Install Validation Library
```bash
npm install validator@13
```

### 3. Create User Controller
`/src/controllers/users.js`:
```javascript
import validator from 'validator';
import { getUsers, createUser } from '../data/users.js';

export const listUsers = (req, res) => {
  const users = getUsers();
  res.json(users);
};

export const addUser = (req, res) => {
  const { name, email } = req.body;
  
  // Validation
  if (!name || !email) {
    return res.status(400).json({
      error: 'Bad Request',
      message: 'Name and email are required'
    });
  }
  
  if (!validator.isEmail(email)) {
    return res.status(400).json({
      error: 'Bad Request',
      message: 'Invalid email format'
    });
  }
  
  // Create user
  const newUser = createUser({ name, email });
  res.status(201).json(newUser);
};
```

### 4. Create User Routes
`/src/routes/users.js`:
```javascript
import { Router } from 'express';
import { listUsers, addUser } from '../controllers/users.js';

const router = Router();

router.get('/api/users', listUsers);
router.post('/api/users', addUser);

export default router;
```

### 5. Update Main Server
Add to `/src/index.js`:
```javascript
import userRoutes from './routes/users.js';

// After other routes
app.use('/', userRoutes);
```

## User Object Schema
```json
{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "createdAt": "2025-01-22T10:00:00.000Z"
}
```

## Acceptance Criteria
- [ ] GET `/api/users` returns array of users
- [ ] POST `/api/users` creates new user with unique ID
- [ ] Email validation using validator library
- [ ] Required field validation (name, email)
- [ ] Proper HTTP status codes (200, 201, 400)
- [ ] ISO 8601 timestamp for createdAt
- [ ] Mock data persists during server runtime

## Test Strategy
1. List users:
   ```bash
   curl http://localhost:3000/api/users
   ```

2. Create valid user:
   ```bash
   curl -X POST http://localhost:3000/api/users \
     -H "Content-Type: application/json" \
     -d '{"name": "Test User", "email": "test@example.com"}'
   ```

3. Test validation errors:
   - Missing fields
   - Invalid email format
   - Empty values

4. Verify ID auto-increment
5. Check persistence within session