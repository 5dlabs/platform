# Task 3: Implement JWT Authentication System

## Overview

This task implements a secure JWT-based authentication system for the Express application. It includes user registration, login functionality, token generation and validation, and middleware for protecting routes. The implementation builds upon the User model from Task 2 and provides the foundation for securing API endpoints in subsequent tasks.

## Objectives

- Implement secure password hashing with bcrypt
- Create JWT token generation and validation utilities
- Build registration endpoint with email/password validation
- Implement login endpoint with credential verification
- Create authentication middleware for route protection
- Handle authentication errors appropriately
- Optional: Implement refresh token mechanism

## Technical Requirements

### Dependencies
- **jsonwebtoken** (^9.0.2): JWT token generation and verification
- **bcrypt** (^5.1.1): Password hashing (already installed in Task 2)

### JWT Configuration
- Token expiration: 24 hours for access tokens
- Payload includes: user ID and email
- Secret key from environment variable
- Bearer token format in Authorization header

### Security Requirements
- Passwords hashed with bcrypt (10 salt rounds)
- Email validation for proper format
- Password strength requirements (minimum 8 characters)
- Proper error messages without leaking user existence
- Token validation on protected routes

## Implementation Steps

### 1. JWT Utility Functions (Subtask 3.1)

Install jsonwebtoken:
```bash
npm install jsonwebtoken@^9.0.2
```

Create `src/utils/jwt.js`:
```javascript
const jwt = require('jsonwebtoken');

const JWT_SECRET = process.env.JWT_SECRET || 'default-secret-change-in-production';
const JWT_EXPIRES_IN = '24h';

const generateToken = (payload) => {
  return jwt.sign(payload, JWT_SECRET, {
    expiresIn: JWT_EXPIRES_IN
  });
};

const verifyToken = (token) => {
  try {
    return jwt.verify(token, JWT_SECRET);
  } catch (error) {
    if (error.name === 'TokenExpiredError') {
      throw new Error('Token expired');
    } else if (error.name === 'JsonWebTokenError') {
      throw new Error('Invalid token');
    }
    throw error;
  }
};

const generateRefreshToken = (payload) => {
  return jwt.sign(payload, JWT_SECRET, {
    expiresIn: '7d'
  });
};

module.exports = {
  generateToken,
  verifyToken,
  generateRefreshToken
};
```

### 2. Password Utilities (Subtask 3.2)

Create `src/utils/password.js`:
```javascript
const bcrypt = require('bcrypt');

const SALT_ROUNDS = 10;

const hashPassword = async (password) => {
  return bcrypt.hash(password, SALT_ROUNDS);
};

const comparePassword = async (password, hash) => {
  return bcrypt.compare(password, hash);
};

const validatePasswordStrength = (password) => {
  if (!password || password.length < 8) {
    return {
      valid: false,
      message: 'Password must be at least 8 characters long'
    };
  }
  
  // Additional checks can be added here
  // e.g., require numbers, special characters, etc.
  
  return { valid: true };
};

module.exports = {
  hashPassword,
  comparePassword,
  validatePasswordStrength
};
```

### 3. Authentication Middleware (Subtask 3.3)

Create `src/middleware/auth.js`:
```javascript
const { verifyToken } = require('../utils/jwt');
const User = require('../models/User');

const authenticateToken = (req, res, next) => {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1]; // Bearer TOKEN
  
  if (!token) {
    return res.status(401).json({
      error: {
        message: 'Access token required',
        code: 'TOKEN_REQUIRED'
      }
    });
  }
  
  try {
    const decoded = verifyToken(token);
    
    // Optionally verify user still exists
    const user = User.findById(decoded.userId);
    if (!user) {
      return res.status(401).json({
        error: {
          message: 'User not found',
          code: 'USER_NOT_FOUND'
        }
      });
    }
    
    // Attach user info to request
    req.user = {
      id: decoded.userId,
      email: decoded.email
    };
    
    next();
  } catch (error) {
    if (error.message === 'Token expired') {
      return res.status(401).json({
        error: {
          message: 'Token expired',
          code: 'TOKEN_EXPIRED'
        }
      });
    }
    
    return res.status(401).json({
      error: {
        message: 'Invalid token',
        code: 'INVALID_TOKEN'
      }
    });
  }
};

module.exports = {
  authenticateToken
};
```

### 4. Registration Endpoint (Subtask 3.4)

Create `src/routes/auth.js`:
```javascript
const express = require('express');
const router = express.Router();
const User = require('../models/User');
const { hashPassword, comparePassword, validatePasswordStrength } = require('../utils/password');
const { generateToken, generateRefreshToken } = require('../utils/jwt');

// Email validation regex
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

// POST /auth/register
router.post('/register', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    // Validate input
    if (!email || !password) {
      return res.status(400).json({
        error: {
          message: 'Email and password are required',
          code: 'MISSING_CREDENTIALS'
        }
      });
    }
    
    // Validate email format
    if (!EMAIL_REGEX.test(email)) {
      return res.status(400).json({
        error: {
          message: 'Invalid email format',
          code: 'INVALID_EMAIL'
        }
      });
    }
    
    // Validate password strength
    const passwordValidation = validatePasswordStrength(password);
    if (!passwordValidation.valid) {
      return res.status(400).json({
        error: {
          message: passwordValidation.message,
          code: 'WEAK_PASSWORD'
        }
      });
    }
    
    // Check if user already exists
    const existingUser = User.findByEmail(email);
    if (existingUser) {
      return res.status(409).json({
        error: {
          message: 'Email already registered',
          code: 'EMAIL_EXISTS'
        }
      });
    }
    
    // Hash password and create user
    const hashedPassword = await hashPassword(password);
    const user = User.create(email, hashedPassword);
    
    // Generate tokens
    const tokenPayload = {
      userId: user.id,
      email: user.email
    };
    
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    res.status(201).json({
      message: 'User registered successfully',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken
      }
    });
  } catch (error) {
    console.error('Registration error:', error);
    res.status(500).json({
      error: {
        message: 'Registration failed',
        code: 'REGISTRATION_ERROR'
      }
    });
  }
});

module.exports = router;
```

### 5. Login Endpoint (Subtask 3.5)

Add to `src/routes/auth.js`:
```javascript
// POST /auth/login
router.post('/login', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    // Validate input
    if (!email || !password) {
      return res.status(400).json({
        error: {
          message: 'Email and password are required',
          code: 'MISSING_CREDENTIALS'
        }
      });
    }
    
    // Find user by email
    const user = User.findByEmail(email);
    if (!user) {
      return res.status(401).json({
        error: {
          message: 'Invalid credentials',
          code: 'INVALID_CREDENTIALS'
        }
      });
    }
    
    // Verify password
    const isValidPassword = await comparePassword(password, user.password);
    if (!isValidPassword) {
      return res.status(401).json({
        error: {
          message: 'Invalid credentials',
          code: 'INVALID_CREDENTIALS'
        }
      });
    }
    
    // Generate tokens
    const tokenPayload = {
      userId: user.id,
      email: user.email
    };
    
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    res.json({
      message: 'Login successful',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken
      }
    });
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({
      error: {
        message: 'Login failed',
        code: 'LOGIN_ERROR'
      }
    });
  }
});
```

### 6. Optional: Refresh Token Endpoint (Subtask 3.7)

Add to `src/routes/auth.js`:
```javascript
// POST /auth/refresh
router.post('/refresh', (req, res) => {
  try {
    const { refreshToken } = req.body;
    
    if (!refreshToken) {
      return res.status(400).json({
        error: {
          message: 'Refresh token required',
          code: 'MISSING_REFRESH_TOKEN'
        }
      });
    }
    
    // Verify refresh token
    const decoded = verifyToken(refreshToken);
    
    // Generate new access token
    const tokenPayload = {
      userId: decoded.userId,
      email: decoded.email
    };
    
    const accessToken = generateToken(tokenPayload);
    
    res.json({
      accessToken
    });
  } catch (error) {
    res.status(401).json({
      error: {
        message: 'Invalid refresh token',
        code: 'INVALID_REFRESH_TOKEN'
      }
    });
  }
});
```

### 7. Integrate Authentication Routes

Update `src/app.js` to include auth routes:
```javascript
// Add after other requires
const authRoutes = require('./routes/auth');

// Add before error handling middleware
app.use('/auth', authRoutes);

// Example of protecting a route
const { authenticateToken } = require('./middleware/auth');

app.get('/api/protected', authenticateToken, (req, res) => {
  res.json({
    message: 'This is a protected route',
    user: req.user
  });
});
```

## Testing

### Manual Testing

1. **Test Registration**:
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

2. **Test Login**:
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

3. **Test Protected Route**:
```bash
# Use the token from login response
curl http://localhost:3000/api/protected \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

### Automated Tests

Create `tests/auth.test.js`:
```javascript
const request = require('supertest');
const { app } = require('../src/app');
const { resetDatabase } = require('../src/db/init');

beforeEach(() => {
  resetDatabase();
});

describe('Authentication', () => {
  describe('POST /auth/register', () => {
    test('registers new user', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'newuser@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(201);
      expect(response.body).toHaveProperty('tokens.accessToken');
      expect(response.body.user.email).toBe('newuser@example.com');
    });
    
    test('rejects duplicate email', async () => {
      await request(app)
        .post('/auth/register')
        .send({
          email: 'duplicate@example.com',
          password: 'password123'
        });
      
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'duplicate@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(409);
      expect(response.body.error.code).toBe('EMAIL_EXISTS');
    });
  });
  
  describe('POST /auth/login', () => {
    beforeEach(async () => {
      await request(app)
        .post('/auth/register')
        .send({
          email: 'user@example.com',
          password: 'password123'
        });
    });
    
    test('logs in with valid credentials', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'user@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('tokens.accessToken');
    });
    
    test('rejects invalid password', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'user@example.com',
          password: 'wrongpassword'
        });
      
      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('INVALID_CREDENTIALS');
    });
  });
});
```

## Common Issues and Solutions

### Issue: JWT_SECRET not set
**Solution**: Ensure JWT_SECRET is in .env file and loaded before jwt utils

### Issue: Token expired immediately
**Solution**: Check server time and JWT_EXPIRES_IN configuration

### Issue: Password comparison always fails
**Solution**: Ensure passwords are hashed before storing in database

## Security Considerations

1. **Never store plain text passwords**
2. **Use strong JWT secret in production**
3. **Implement rate limiting on auth endpoints** (Task 5)
4. **Consider implementing account lockout after failed attempts**
5. **Use HTTPS in production to protect tokens in transit**
6. **Implement token blacklisting for logout functionality**

## Next Steps

After completing this task:
- Authentication system is fully functional
- Users can register and login
- JWT tokens are generated and validated
- Routes can be protected with authentication middleware
- Foundation is ready for Task 4: Create Task Management API

The authentication system provides the security layer needed to protect user data and ensure only authorized users can access their tasks.