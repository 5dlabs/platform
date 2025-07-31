# Task 3: User Authentication System

## Overview
Implement a secure, scalable authentication system using JWT tokens with refresh token rotation, password hashing with bcrypt, and email-based password reset functionality.

## Technical Implementation Guide

### Phase 1: Authentication Controllers

#### User Registration Controller
```typescript
// backend/src/controllers/authController.ts
import bcrypt from 'bcrypt';
import { Request, Response } from 'express';
import { UserRepository } from '../repositories/userRepository';
import { generateTokens } from '../utils/jwt';
import { validateEmail, validatePassword } from '../utils/validators';

export const register = async (req: Request, res: Response) => {
  try {
    const { email, username, password } = req.body;
    
    // Validation
    if (!validateEmail(email)) {
      return res.status(400).json({ error: 'Invalid email format' });
    }
    
    if (!validatePassword(password)) {
      return res.status(400).json({ 
        error: 'Password must be at least 8 characters with uppercase, lowercase, and number' 
      });
    }
    
    // Check if user exists
    const existingUser = await userRepository.findByEmail(email);
    if (existingUser) {
      return res.status(409).json({ error: 'Email already registered' });
    }
    
    // Hash password
    const passwordHash = await bcrypt.hash(password, 12);
    
    // Create user
    const user = await userRepository.create({
      email,
      username,
      passwordHash,
      isOnline: false
    });
    
    // Generate tokens
    const tokens = await generateTokens(user.id);
    
    res.status(201).json({
      user: {
        id: user.id,
        email: user.email,
        username: user.username
      },
      ...tokens
    });
  } catch (error) {
    console.error('Registration error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};
```

#### User Login Controller
```typescript
export const login = async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body;
    
    // Find user
    const user = await userRepository.findByEmail(email);
    if (!user) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    // Verify password
    const isValidPassword = await bcrypt.compare(password, user.passwordHash);
    if (!isValidPassword) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    // Update online status
    await userRepository.updateOnlineStatus(user.id, true);
    
    // Generate tokens
    const tokens = await generateTokens(user.id);
    
    res.json({
      user: {
        id: user.id,
        email: user.email,
        username: user.username,
        avatarUrl: user.avatarUrl
      },
      ...tokens
    });
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};
```

### Phase 2: JWT Token Management

#### Token Generation and Validation
```typescript
// backend/src/utils/jwt.ts
import jwt from 'jsonwebtoken';
import { randomBytes } from 'crypto';
import redis from '../config/redis';

interface TokenPayload {
  userId: string;
  type: 'access' | 'refresh';
}

export const generateTokens = async (userId: string) => {
  // Generate access token (15 minutes)
  const accessToken = jwt.sign(
    { userId, type: 'access' } as TokenPayload,
    process.env.JWT_SECRET!,
    { expiresIn: '15m' }
  );
  
  // Generate refresh token (7 days)
  const refreshTokenId = randomBytes(32).toString('hex');
  const refreshToken = jwt.sign(
    { userId, tokenId: refreshTokenId, type: 'refresh' },
    process.env.JWT_REFRESH_SECRET!,
    { expiresIn: '7d' }
  );
  
  // Store refresh token in Redis with metadata
  await redis.setex(
    `refresh_token:${userId}:${refreshTokenId}`,
    7 * 24 * 60 * 60, // 7 days in seconds
    JSON.stringify({
      createdAt: new Date().toISOString(),
      userAgent: 'user-agent-here', // Should be from request
      ipAddress: 'ip-here' // Should be from request
    })
  );
  
  return { accessToken, refreshToken };
};

export const verifyAccessToken = (token: string): TokenPayload => {
  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as TokenPayload;
    if (decoded.type !== 'access') {
      throw new Error('Invalid token type');
    }
    return decoded;
  } catch (error) {
    throw new Error('Invalid access token');
  }
};

export const verifyRefreshToken = async (token: string): Promise<string> => {
  try {
    const decoded = jwt.verify(token, process.env.JWT_REFRESH_SECRET!) as any;
    if (decoded.type !== 'refresh') {
      throw new Error('Invalid token type');
    }
    
    // Check if token exists in Redis
    const tokenData = await redis.get(`refresh_token:${decoded.userId}:${decoded.tokenId}`);
    if (!tokenData) {
      throw new Error('Refresh token revoked');
    }
    
    return decoded.userId;
  } catch (error) {
    throw new Error('Invalid refresh token');
  }
};
```

#### Token Refresh Endpoint
```typescript
export const refreshTokens = async (req: Request, res: Response) => {
  try {
    const { refreshToken } = req.body;
    
    if (!refreshToken) {
      return res.status(400).json({ error: 'Refresh token required' });
    }
    
    // Verify refresh token
    const userId = await verifyRefreshToken(refreshToken);
    
    // Revoke old refresh token
    const decoded = jwt.decode(refreshToken) as any;
    await redis.del(`refresh_token:${userId}:${decoded.tokenId}`);
    
    // Generate new tokens
    const tokens = await generateTokens(userId);
    
    res.json(tokens);
  } catch (error) {
    res.status(401).json({ error: 'Invalid refresh token' });
  }
};
```

### Phase 3: Authentication Middleware

#### Protected Route Middleware
```typescript
// backend/src/middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import { verifyAccessToken } from '../utils/jwt';

export interface AuthRequest extends Request {
  userId?: string;
}

export const authenticate = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction
) => {
  try {
    const authHeader = req.headers.authorization;
    
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return res.status(401).json({ error: 'No token provided' });
    }
    
    const token = authHeader.substring(7);
    const payload = verifyAccessToken(token);
    
    req.userId = payload.userId;
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid token' });
  }
};

// Optional authentication (for public routes that can be enhanced for logged-in users)
export const optionalAuth = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction
) => {
  try {
    const authHeader = req.headers.authorization;
    
    if (authHeader && authHeader.startsWith('Bearer ')) {
      const token = authHeader.substring(7);
      const payload = verifyAccessToken(token);
      req.userId = payload.userId;
    }
    
    next();
  } catch (error) {
    // Continue without authentication
    next();
  }
};
```

### Phase 4: Password Reset System

#### Password Reset Request
```typescript
// backend/src/controllers/passwordResetController.ts
import crypto from 'crypto';
import { sendPasswordResetEmail } from '../services/emailService';

export const requestPasswordReset = async (req: Request, res: Response) => {
  try {
    const { email } = req.body;
    
    const user = await userRepository.findByEmail(email);
    if (!user) {
      // Don't reveal if email exists
      return res.json({ message: 'If email exists, reset link has been sent' });
    }
    
    // Generate reset token
    const resetToken = crypto.randomBytes(32).toString('hex');
    const resetTokenHash = crypto
      .createHash('sha256')
      .update(resetToken)
      .digest('hex');
    
    // Store in Redis with 1 hour expiry
    await redis.setex(
      `password_reset:${resetTokenHash}`,
      3600,
      user.id
    );
    
    // Send email
    const resetUrl = `${process.env.FRONTEND_URL}/reset-password/${resetToken}`;
    await sendPasswordResetEmail(user.email, resetUrl);
    
    res.json({ message: 'If email exists, reset link has been sent' });
  } catch (error) {
    console.error('Password reset error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};

export const resetPassword = async (req: Request, res: Response) => {
  try {
    const { token, newPassword } = req.body;
    
    // Hash token to match stored version
    const resetTokenHash = crypto
      .createHash('sha256')
      .update(token)
      .digest('hex');
    
    // Get user ID from Redis
    const userId = await redis.get(`password_reset:${resetTokenHash}`);
    if (!userId) {
      return res.status(400).json({ error: 'Invalid or expired reset token' });
    }
    
    // Validate new password
    if (!validatePassword(newPassword)) {
      return res.status(400).json({ 
        error: 'Password must be at least 8 characters with uppercase, lowercase, and number' 
      });
    }
    
    // Hash new password
    const passwordHash = await bcrypt.hash(newPassword, 12);
    
    // Update user password
    await userRepository.updatePassword(userId, passwordHash);
    
    // Delete reset token
    await redis.del(`password_reset:${resetTokenHash}`);
    
    // Revoke all refresh tokens for security
    const keys = await redis.keys(`refresh_token:${userId}:*`);
    if (keys.length > 0) {
      await redis.del(...keys);
    }
    
    res.json({ message: 'Password reset successful' });
  } catch (error) {
    console.error('Password reset error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};
```

### Phase 5: User Profile Management

#### Profile Endpoints
```typescript
export const getProfile = async (req: AuthRequest, res: Response) => {
  try {
    const user = await userRepository.findById(req.userId!);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }
    
    res.json({
      id: user.id,
      email: user.email,
      username: user.username,
      avatarUrl: user.avatarUrl,
      createdAt: user.createdAt,
      updatedAt: user.updatedAt
    });
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
};

export const updateProfile = async (req: AuthRequest, res: Response) => {
  try {
    const { username, avatarUrl } = req.body;
    
    // Validate username uniqueness if changed
    if (username) {
      const existingUser = await userRepository.findByUsername(username);
      if (existingUser && existingUser.id !== req.userId) {
        return res.status(409).json({ error: 'Username already taken' });
      }
    }
    
    const updatedUser = await userRepository.updateProfile(req.userId!, {
      username,
      avatarUrl
    });
    
    res.json({
      id: updatedUser.id,
      email: updatedUser.email,
      username: updatedUser.username,
      avatarUrl: updatedUser.avatarUrl
    });
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
};
```

### Phase 6: Route Configuration

#### Authentication Routes
```typescript
// backend/src/routes/auth.ts
import { Router } from 'express';
import { authenticate } from '../middleware/auth';
import * as authController from '../controllers/authController';

const router = Router();

// Public routes
router.post('/register', authController.register);
router.post('/login', authController.login);
router.post('/refresh', authController.refreshTokens);
router.post('/password-reset/request', authController.requestPasswordReset);
router.post('/password-reset/confirm', authController.resetPassword);

// Protected routes
router.get('/profile', authenticate, authController.getProfile);
router.put('/profile', authenticate, authController.updateProfile);
router.post('/logout', authenticate, authController.logout);

export default router;
```

### Phase 7: Security Enhancements

#### Rate Limiting
```typescript
// backend/src/middleware/rateLimiter.ts
import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import redis from '../config/redis';

export const authRateLimiter = rateLimit({
  store: new RedisStore({
    client: redis,
    prefix: 'rate_limit:auth:'
  }),
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5, // 5 requests per window
  message: 'Too many authentication attempts, please try again later'
});

export const generalRateLimiter = rateLimit({
  store: new RedisStore({
    client: redis,
    prefix: 'rate_limit:general:'
  }),
  windowMs: 15 * 60 * 1000,
  max: 100
});
```

#### Input Validation
```typescript
// backend/src/utils/validators.ts
export const validateEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

export const validatePassword = (password: string): boolean => {
  // At least 8 characters, one uppercase, one lowercase, one number
  const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/;
  return passwordRegex.test(password);
};

export const validateUsername = (username: string): boolean => {
  // Alphanumeric and underscore, 3-20 characters
  const usernameRegex = /^[a-zA-Z0-9_]{3,20}$/;
  return usernameRegex.test(username);
};
```

## Environment Variables

```env
# JWT Secrets
JWT_SECRET=your-super-secret-jwt-key
JWT_REFRESH_SECRET=your-super-secret-refresh-key

# Email Service
EMAIL_HOST=smtp.gmail.com
EMAIL_PORT=587
EMAIL_USER=your-email@gmail.com
EMAIL_PASSWORD=your-app-password
EMAIL_FROM=noreply@chatapp.com

# Frontend URL (for password reset links)
FRONTEND_URL=http://localhost:3000
```

## Success Metrics

- Registration endpoint creates users with hashed passwords
- Login returns valid JWT tokens
- Protected routes reject invalid tokens
- Refresh token rotation works correctly
- Password reset emails sent successfully
- Rate limiting prevents brute force attacks
- All authentication flows pass integration tests