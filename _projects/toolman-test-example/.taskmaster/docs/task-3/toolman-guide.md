# Toolman Guide for Task 3: User Authentication System

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 3, which focuses on building a complete user authentication system with JWT tokens, refresh tokens, password reset functionality, and protected routes.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the authentication system directory structure

**When to Use**: 
- At the beginning to organize authentication components
- When separating concerns (controllers, middleware, services)
- For organizing authentication utilities and helpers

**How to Use**:
```
# Create authentication structure
create_directory /chat-application/backend/src/auth
create_directory /chat-application/backend/src/auth/controllers
create_directory /chat-application/backend/src/auth/middleware
create_directory /chat-application/backend/src/auth/services
create_directory /chat-application/backend/src/auth/utils
create_directory /chat-application/backend/src/auth/dto
create_directory /chat-application/backend/src/auth/validators
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create authentication controllers, middleware, services, and utilities

**When to Use**: 
- To implement authentication endpoints
- To create JWT middleware
- To implement token generation utilities
- To create password reset services

**How to Use**:
```
# Create authentication controller
write_file /chat-application/backend/src/auth/controllers/authController.ts <controller-content>

# Create JWT middleware
write_file /chat-application/backend/src/auth/middleware/authMiddleware.ts <middleware-content>

# Create token service
write_file /chat-application/backend/src/auth/services/tokenService.ts <token-service>

# Create email service
write_file /chat-application/backend/src/auth/services/emailService.ts <email-service>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review database models and existing configurations

**When to Use**: 
- To check User model from Task 2
- To review Redis configuration
- To verify existing route setup

**How to Use**:
```
# Read User model
read_file /chat-application/backend/src/database/models/User.ts

# Check Redis config
read_file /chat-application/backend/src/database/config/redis.config.ts

# Review server setup
read_file /chat-application/backend/src/index.ts
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update existing files to integrate authentication

**When to Use**: 
- To add authentication routes to main server
- To update package.json with new dependencies
- To modify environment configuration

**How to Use**:
```
# Add authentication dependencies
edit_file /chat-application/backend/package.json
# Add: jsonwebtoken, bcrypt, @types/jsonwebtoken, @types/bcrypt

# Update server routes
edit_file /chat-application/backend/src/index.ts
# Add authentication routes and middleware
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify created structure and files

**When to Use**: 
- After creating authentication directories
- To confirm all components are in place
- Before testing implementation

**How to Use**:
```
# Verify auth structure
list_directory /chat-application/backend/src/auth

# Check controllers
list_directory /chat-application/backend/src/auth/controllers
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Directory Setup Phase**
   - Use `create_directory` to establish auth structure
   - Organize by controllers, middleware, services, utils

2. **Core Authentication Phase**
   - Use `write_file` to create authController.ts with 5 endpoints:
     - POST /api/auth/register
     - POST /api/auth/login
     - POST /api/auth/refresh
     - GET /api/auth/profile
     - POST /api/auth/reset-password

3. **Token Management Phase**
   - Use `write_file` to create tokenService.ts
   - Implement generateTokens function
   - Create token validation utilities
   - Set up Redis refresh token storage

4. **Middleware Implementation Phase**
   - Use `write_file` to create authMiddleware.ts
   - Implement JWT verification
   - Create route protection logic
   - Handle token expiration

5. **Password Management Phase**
   - Use `write_file` to create password utilities
   - Implement bcrypt hashing
   - Create password reset service
   - Set up email service integration

6. **Integration Phase**
   - Use `edit_file` to update main server file
   - Add authentication routes
   - Apply middleware to protected routes
   - Update environment variables

## Best Practices

1. **Security First**: Never store plain passwords, use bcrypt rounds >= 10
2. **Token Expiration**: Keep access tokens short (15m), refresh tokens longer (7d)
3. **Error Handling**: Don't reveal if email exists during registration
4. **Validation**: Validate all inputs before processing
5. **Environment Variables**: Store all secrets in .env file

## Task-Specific Implementation Details

### JWT Token Generation Pattern
```typescript
// tokenService.ts
import jwt from 'jsonwebtoken';
import { redisClient } from '../database/config/redis.config';

export const generateTokens = async (userId: string) => {
  const accessToken = jwt.sign(
    { userId, type: 'access' },
    process.env.JWT_SECRET!,
    { expiresIn: '15m' }
  );
  
  const refreshToken = jwt.sign(
    { userId, type: 'refresh' },
    process.env.JWT_REFRESH_SECRET!,
    { expiresIn: '7d' }
  );
  
  // Store in Redis with expiration
  await redisClient.setex(
    `refresh_token:${userId}`,
    60 * 60 * 24 * 7,
    refreshToken
  );
  
  return { accessToken, refreshToken };
};
```

### Authentication Middleware Pattern
```typescript
// authMiddleware.ts
export const authenticate = async (req, res, next) => {
  const token = req.headers.authorization?.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }
  
  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET!);
    req.userId = decoded.userId;
    next();
  } catch (error) {
    return res.status(401).json({ error: 'Invalid token' });
  }
};
```

### Password Hashing Pattern
```typescript
// Password hashing
const hashedPassword = await bcrypt.hash(password, 10);

// Password verification
const isValid = await bcrypt.compare(password, hashedPassword);
```

## Troubleshooting

- **Token Expiration**: Handle both access and refresh token expiration
- **Redis Connection**: Ensure Redis is running before starting server
- **CORS Issues**: Configure CORS for authentication headers
- **Rate Limiting**: Implement rate limiting on auth endpoints
- **Email Delivery**: Use proper SMTP configuration for password reset

## Testing Approach

1. **Unit Tests**:
   - Test token generation and validation
   - Test password hashing and comparison
   - Test individual auth endpoints

2. **Integration Tests**:
   - Test complete registration flow
   - Test login and token refresh cycle
   - Test password reset flow

3. **Security Tests**:
   - Test invalid tokens
   - Test expired tokens
   - Test SQL injection attempts
   - Test brute force protection