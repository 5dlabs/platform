# Task 3: Implement JWT Authentication - Autonomous AI Agent Prompt

You are tasked with implementing a complete JWT-based authentication system for the Express application. This includes secure password hashing, user registration, login functionality, token generation/validation, and middleware for protecting routes.

## Your Mission

Build a secure authentication system using JWT tokens and bcrypt for password hashing. Create registration and login endpoints, implement token-based authentication middleware, and ensure proper error handling throughout the authentication flow.

## Prerequisites

Ensure Tasks 1 and 2 are complete:
- Express server is running with middleware configured
- SQLite database is set up with User model
- bcrypt is already installed (from Task 2)

## Step-by-Step Instructions

### 1. Install JWT Package

```bash
npm install jsonwebtoken@^9.0.2
```

### 2. Create JWT Utility Functions

Create `src/utils/jwt.js`:

```javascript
const jwt = require('jsonwebtoken');

const JWT_SECRET = process.env.JWT_SECRET || 'default-secret-change-in-production';
const JWT_EXPIRES_IN = '24h';
const REFRESH_TOKEN_EXPIRES_IN = '7d';

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
    expiresIn: REFRESH_TOKEN_EXPIRES_IN
  });
};

const decodeToken = (token) => {
  return jwt.decode(token);
};

module.exports = {
  generateToken,
  verifyToken,
  generateRefreshToken,
  decodeToken
};
```

### 3. Create Password Utility Functions

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
  const errors = [];
  
  if (!password) {
    return {
      valid: false,
      message: 'Password is required'
    };
  }
  
  if (password.length < 8) {
    errors.push('Password must be at least 8 characters long');
  }
  
  // Optional: Add more validation rules
  // if (!/\d/.test(password)) {
  //   errors.push('Password must contain at least one number');
  // }
  // if (!/[A-Z]/.test(password)) {
  //   errors.push('Password must contain at least one uppercase letter');
  // }
  
  if (errors.length > 0) {
    return {
      valid: false,
      message: errors.join(', ')
    };
  }
  
  return { valid: true };
};

module.exports = {
  hashPassword,
  comparePassword,
  validatePasswordStrength
};
```

### 4. Create Authentication Middleware

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
    console.error('Authentication error:', error.message);
    
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

// Optional middleware for routes that can work with or without auth
const optionalAuth = (req, res, next) => {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return next(); // Continue without user
  }
  
  try {
    const decoded = verifyToken(token);
    req.user = {
      id: decoded.userId,
      email: decoded.email
    };
  } catch (error) {
    // Invalid token, but continue anyway
    console.log('Optional auth - invalid token:', error.message);
  }
  
  next();
};

module.exports = {
  authenticateToken,
  optionalAuth
};
```

### 5. Create Authentication Routes

Create `src/routes/auth.js`:

```javascript
const express = require('express');
const router = express.Router();
const User = require('../models/User');
const { hashPassword, comparePassword, validatePasswordStrength } = require('../utils/password');
const { generateToken, generateRefreshToken, verifyToken } = require('../utils/jwt');

// Email validation regex
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

// POST /auth/register
router.post('/register', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    // Validate input presence
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
    const existingUser = User.findByEmail(email.toLowerCase());
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
    const user = User.create(email.toLowerCase(), hashedPassword);
    
    // Generate tokens
    const tokenPayload = {
      userId: user.id,
      email: user.email
    };
    
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    // Log successful registration
    console.log(`New user registered: ${user.email}`);
    
    res.status(201).json({
      message: 'User registered successfully',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken,
        expiresIn: '24h'
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
    const user = User.findByEmail(email.toLowerCase());
    if (!user) {
      // Generic error to avoid revealing user existence
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
    
    // Log successful login
    console.log(`User logged in: ${user.email}`);
    
    res.json({
      message: 'Login successful',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken,
        expiresIn: '24h'
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
    let decoded;
    try {
      decoded = verifyToken(refreshToken);
    } catch (error) {
      return res.status(401).json({
        error: {
          message: 'Invalid refresh token',
          code: 'INVALID_REFRESH_TOKEN'
        }
      });
    }
    
    // Verify user still exists
    const user = User.findById(decoded.userId);
    if (!user) {
      return res.status(401).json({
        error: {
          message: 'User not found',
          code: 'USER_NOT_FOUND'
        }
      });
    }
    
    // Generate new access token
    const tokenPayload = {
      userId: decoded.userId,
      email: decoded.email
    };
    
    const accessToken = generateToken(tokenPayload);
    
    res.json({
      accessToken,
      expiresIn: '24h'
    });
  } catch (error) {
    console.error('Token refresh error:', error);
    res.status(500).json({
      error: {
        message: 'Token refresh failed',
        code: 'REFRESH_ERROR'
      }
    });
  }
});

// GET /auth/me - Get current user info
router.get('/me', require('../middleware/auth').authenticateToken, (req, res) => {
  const user = User.findById(req.user.id);
  
  if (!user) {
    return res.status(404).json({
      error: {
        message: 'User not found',
        code: 'USER_NOT_FOUND'
      }
    });
  }
  
  res.json({
    user: {
      id: user.id,
      email: user.email,
      createdAt: user.created_at
    }
  });
});

module.exports = router;
```

### 6. Integrate Authentication Routes into Express App

Update `src/app.js` to include authentication routes:

```javascript
// Add after other requires
const authRoutes = require('./routes/auth');

// Add after body parsing middleware but before error handling
app.use('/auth', authRoutes);

// Add a protected test route
const { authenticateToken } = require('./middleware/auth');

app.get('/api/protected', authenticateToken, (req, res) => {
  res.json({
    message: 'This is a protected route',
    user: req.user
  });
});
```

### 7. Update Environment Variables

Ensure `.env` has a secure JWT_SECRET:
```
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
```

## Verification Steps

1. **Start the server**:
   ```bash
   npm run dev
   ```

2. **Test Registration**:
   ```bash
   curl -X POST http://localhost:3000/auth/register \
     -H "Content-Type: application/json" \
     -d '{"email":"newuser@example.com","password":"password123"}'
   ```
   Expected: 201 status with user info and tokens

3. **Test Registration Validation**:
   - Weak password:
     ```bash
     curl -X POST http://localhost:3000/auth/register \
       -H "Content-Type: application/json" \
       -d '{"email":"test@example.com","password":"123"}'
     ```
     Expected: 400 status with "Password must be at least 8 characters long"
   
   - Invalid email:
     ```bash
     curl -X POST http://localhost:3000/auth/register \
       -H "Content-Type: application/json" \
       -d '{"email":"notanemail","password":"password123"}'
     ```
     Expected: 400 status with "Invalid email format"

4. **Test Login**:
   ```bash
   curl -X POST http://localhost:3000/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"newuser@example.com","password":"password123"}'
   ```
   Expected: 200 status with tokens

5. **Test Protected Route**:
   ```bash
   # Use the accessToken from login response
   TOKEN="your-access-token-here"
   curl http://localhost:3000/api/protected \
     -H "Authorization: Bearer $TOKEN"
   ```
   Expected: 200 status with protected message

6. **Test Token Refresh**:
   ```bash
   # Use the refreshToken from login response
   REFRESH_TOKEN="your-refresh-token-here"
   curl -X POST http://localhost:3000/auth/refresh \
     -H "Content-Type: application/json" \
     -d "{\"refreshToken\":\"$REFRESH_TOKEN\"}"
   ```
   Expected: New access token

7. **Test Current User Endpoint**:
   ```bash
   curl http://localhost:3000/auth/me \
     -H "Authorization: Bearer $TOKEN"
   ```
   Expected: User information

## Success Criteria

- User registration creates new users with hashed passwords
- Duplicate email registration returns 409 error
- Login with correct credentials returns JWT tokens
- Login with incorrect credentials returns 401 error
- Protected routes require valid JWT token
- Expired tokens are rejected with appropriate error
- Refresh token generates new access token
- Password validation enforces minimum length
- All error responses follow consistent format

## Security Considerations

1. **Always hash passwords** - Never store plain text
2. **Use environment variables** - Keep JWT_SECRET secure
3. **Validate all inputs** - Email format, password strength
4. **Generic error messages** - Don't reveal if email exists
5. **Token expiration** - 24 hours for access, 7 days for refresh
6. **HTTPS in production** - Protect tokens in transit

## Important Notes

- Passwords are hashed with bcrypt using 10 salt rounds
- JWT tokens include userId and email in payload
- Authorization header format: `Bearer <token>`
- Tokens expire after 24 hours (configurable)
- Error responses maintain consistent structure
- Email addresses are normalized to lowercase

You have now successfully implemented JWT authentication. The application has secure user registration, login, and route protection ready for the Task Management API in Task 4.