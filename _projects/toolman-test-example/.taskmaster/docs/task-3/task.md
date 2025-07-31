# Task 3: User Authentication System - Technical Implementation Guide

## Overview

This task implements a comprehensive user authentication system with JWT-based authentication, refresh tokens, password reset functionality, and secure user management. The system follows industry best practices for security and scalability.

## Architecture

### Components

1. **Authentication Controller** - Handles all auth-related HTTP endpoints
2. **JWT Service** - Manages token generation, validation, and refresh
3. **User Service** - Handles user data management and validation
4. **Email Service** - Sends password reset and verification emails
5. **Redis Cache** - Stores refresh tokens and temporary data
6. **Authentication Middleware** - Protects routes and validates tokens

### Security Layers

- Password hashing with bcrypt (10+ salt rounds)
- JWT tokens with separate secrets for access and refresh
- Refresh token rotation on use
- Rate limiting on authentication endpoints
- Email verification for password resets
- HTTPS-only cookie storage for tokens

## Implementation Steps

### 1. Install Required Dependencies

```bash
npm install jsonwebtoken bcrypt redis ioredis nodemailer
npm install --save-dev @types/jsonwebtoken @types/bcrypt @types/nodemailer
```

### 2. Environment Configuration

Create or update `.env` file:

```env
JWT_SECRET=your-super-secret-jwt-key-minimum-32-chars
JWT_REFRESH_SECRET=your-refresh-token-secret-minimum-32-chars
REDIS_URL=redis://localhost:6379
EMAIL_HOST=smtp.gmail.com
EMAIL_PORT=587
EMAIL_USER=your-email@gmail.com
EMAIL_PASS=your-app-password
FRONTEND_URL=http://localhost:3000
```

### 3. JWT Service Implementation

```typescript
// src/services/jwt.service.ts
import jwt from 'jsonwebtoken';
import { redisClient } from '../config/redis';

interface TokenPayload {
  userId: string;
  email: string;
}

export class JWTService {
  private static ACCESS_TOKEN_EXPIRY = '15m';
  private static REFRESH_TOKEN_EXPIRY = '7d';
  private static REFRESH_TOKEN_EXPIRY_SECONDS = 7 * 24 * 60 * 60;

  static generateTokens(payload: TokenPayload) {
    const accessToken = jwt.sign(
      payload,
      process.env.JWT_SECRET!,
      { expiresIn: this.ACCESS_TOKEN_EXPIRY }
    );

    const refreshToken = jwt.sign(
      payload,
      process.env.JWT_REFRESH_SECRET!,
      { expiresIn: this.REFRESH_TOKEN_EXPIRY }
    );

    return { accessToken, refreshToken };
  }

  static async storeRefreshToken(userId: string, refreshToken: string) {
    await redisClient.set(
      `refresh_token:${userId}`,
      refreshToken,
      'EX',
      this.REFRESH_TOKEN_EXPIRY_SECONDS
    );
  }

  static async validateRefreshToken(userId: string, token: string) {
    const storedToken = await redisClient.get(`refresh_token:${userId}`);
    return storedToken === token;
  }

  static async revokeRefreshToken(userId: string) {
    await redisClient.del(`refresh_token:${userId}`);
  }

  static verifyAccessToken(token: string): TokenPayload {
    return jwt.verify(token, process.env.JWT_SECRET!) as TokenPayload;
  }

  static verifyRefreshToken(token: string): TokenPayload {
    return jwt.verify(token, process.env.JWT_REFRESH_SECRET!) as TokenPayload;
  }
}
```

### 4. Authentication Controller

```typescript
// src/controllers/auth.controller.ts
import { Request, Response } from 'express';
import bcrypt from 'bcrypt';
import { UserService } from '../services/user.service';
import { JWTService } from '../services/jwt.service';
import { EmailService } from '../services/email.service';
import { validateRegistration, validateLogin } from '../validators/auth.validator';

export class AuthController {
  static async register(req: Request, res: Response) {
    try {
      const { error } = validateRegistration(req.body);
      if (error) {
        return res.status(400).json({ error: error.details[0].message });
      }

      const { email, password, name } = req.body;

      // Check if user exists
      const existingUser = await UserService.findByEmail(email);
      if (existingUser) {
        return res.status(409).json({ error: 'User already exists' });
      }

      // Hash password
      const hashedPassword = await bcrypt.hash(password, 12);

      // Create user
      const user = await UserService.create({
        email,
        password: hashedPassword,
        name
      });

      // Generate tokens
      const { accessToken, refreshToken } = JWTService.generateTokens({
        userId: user.id,
        email: user.email
      });

      // Store refresh token
      await JWTService.storeRefreshToken(user.id, refreshToken);

      // Set cookies
      res.cookie('refreshToken', refreshToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 7 * 24 * 60 * 60 * 1000 // 7 days
      });

      res.status(201).json({
        message: 'User registered successfully',
        user: {
          id: user.id,
          email: user.email,
          name: user.name
        },
        accessToken
      });
    } catch (error) {
      console.error('Registration error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }

  static async login(req: Request, res: Response) {
    try {
      const { error } = validateLogin(req.body);
      if (error) {
        return res.status(400).json({ error: error.details[0].message });
      }

      const { email, password } = req.body;

      // Find user
      const user = await UserService.findByEmail(email);
      if (!user) {
        return res.status(401).json({ error: 'Invalid credentials' });
      }

      // Verify password
      const isPasswordValid = await bcrypt.compare(password, user.password);
      if (!isPasswordValid) {
        return res.status(401).json({ error: 'Invalid credentials' });
      }

      // Generate tokens
      const { accessToken, refreshToken } = JWTService.generateTokens({
        userId: user.id,
        email: user.email
      });

      // Store refresh token
      await JWTService.storeRefreshToken(user.id, refreshToken);

      // Set cookies
      res.cookie('refreshToken', refreshToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 7 * 24 * 60 * 60 * 1000
      });

      res.json({
        message: 'Login successful',
        user: {
          id: user.id,
          email: user.email,
          name: user.name
        },
        accessToken
      });
    } catch (error) {
      console.error('Login error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }

  static async refresh(req: Request, res: Response) {
    try {
      const { refreshToken } = req.cookies;

      if (!refreshToken) {
        return res.status(401).json({ error: 'Refresh token not provided' });
      }

      // Verify refresh token
      const payload = JWTService.verifyRefreshToken(refreshToken);

      // Validate stored token
      const isValid = await JWTService.validateRefreshToken(payload.userId, refreshToken);
      if (!isValid) {
        return res.status(401).json({ error: 'Invalid refresh token' });
      }

      // Generate new tokens
      const tokens = JWTService.generateTokens({
        userId: payload.userId,
        email: payload.email
      });

      // Rotate refresh token
      await JWTService.revokeRefreshToken(payload.userId);
      await JWTService.storeRefreshToken(payload.userId, tokens.refreshToken);

      // Update cookie
      res.cookie('refreshToken', tokens.refreshToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 7 * 24 * 60 * 60 * 1000
      });

      res.json({
        accessToken: tokens.accessToken
      });
    } catch (error) {
      console.error('Token refresh error:', error);
      res.status(401).json({ error: 'Invalid refresh token' });
    }
  }

  static async profile(req: Request, res: Response) {
    try {
      const userId = req.user!.userId;
      const user = await UserService.findById(userId);

      if (!user) {
        return res.status(404).json({ error: 'User not found' });
      }

      res.json({
        user: {
          id: user.id,
          email: user.email,
          name: user.name,
          createdAt: user.createdAt
        }
      });
    } catch (error) {
      console.error('Profile error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }

  static async resetPassword(req: Request, res: Response) {
    try {
      const { email } = req.body;

      const user = await UserService.findByEmail(email);
      if (!user) {
        // Don't reveal if user exists
        return res.json({ message: 'If the email exists, a reset link has been sent' });
      }

      // Generate reset token
      const resetToken = await UserService.generatePasswordResetToken(user.id);

      // Send email
      await EmailService.sendPasswordResetEmail(user.email, resetToken);

      res.json({ message: 'If the email exists, a reset link has been sent' });
    } catch (error) {
      console.error('Password reset error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }

  static async logout(req: Request, res: Response) {
    try {
      const userId = req.user!.userId;
      
      // Revoke refresh token
      await JWTService.revokeRefreshToken(userId);

      // Clear cookie
      res.clearCookie('refreshToken');

      res.json({ message: 'Logged out successfully' });
    } catch (error) {
      console.error('Logout error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }
}
```

### 5. Authentication Middleware

```typescript
// src/middleware/auth.middleware.ts
import { Request, Response, NextFunction } from 'express';
import { JWTService } from '../services/jwt.service';

interface AuthRequest extends Request {
  user?: {
    userId: string;
    email: string;
  };
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
    const payload = JWTService.verifyAccessToken(token);

    req.user = {
      userId: payload.userId,
      email: payload.email
    };

    next();
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      return res.status(401).json({ error: 'Token expired' });
    }
    if (error instanceof jwt.JsonWebTokenError) {
      return res.status(401).json({ error: 'Invalid token' });
    }
    return res.status(500).json({ error: 'Internal server error' });
  }
};
```

### 6. Email Service

```typescript
// src/services/email.service.ts
import nodemailer from 'nodemailer';

export class EmailService {
  private static transporter = nodemailer.createTransport({
    host: process.env.EMAIL_HOST,
    port: parseInt(process.env.EMAIL_PORT || '587'),
    secure: false,
    auth: {
      user: process.env.EMAIL_USER,
      pass: process.env.EMAIL_PASS
    }
  });

  static async sendPasswordResetEmail(email: string, resetToken: string) {
    const resetUrl = `${process.env.FRONTEND_URL}/reset-password?token=${resetToken}`;

    const mailOptions = {
      from: process.env.EMAIL_USER,
      to: email,
      subject: 'Password Reset Request',
      html: `
        <h1>Password Reset</h1>
        <p>You requested a password reset. Click the link below to reset your password:</p>
        <a href="${resetUrl}" style="display: inline-block; padding: 10px 20px; background-color: #007bff; color: white; text-decoration: none; border-radius: 5px;">Reset Password</a>
        <p>This link will expire in 1 hour.</p>
        <p>If you didn't request this, please ignore this email.</p>
      `
    };

    await this.transporter.sendMail(mailOptions);
  }
}
```

### 7. Routes Configuration

```typescript
// src/routes/auth.routes.ts
import { Router } from 'express';
import { AuthController } from '../controllers/auth.controller';
import { authenticate } from '../middleware/auth.middleware';
import { rateLimiter } from '../middleware/rateLimiter';

const router = Router();

// Public routes
router.post('/register', rateLimiter.registration, AuthController.register);
router.post('/login', rateLimiter.login, AuthController.login);
router.post('/refresh', AuthController.refresh);
router.post('/reset-password', rateLimiter.passwordReset, AuthController.resetPassword);

// Protected routes
router.get('/profile', authenticate, AuthController.profile);
router.post('/logout', authenticate, AuthController.logout);

export default router;
```

## Security Best Practices

1. **Password Security**
   - Minimum 8 characters, require complexity
   - Hash with bcrypt using 12+ salt rounds
   - Never store plaintext passwords

2. **Token Security**
   - Use strong, unique secrets for JWT
   - Short expiry for access tokens (15 minutes)
   - Rotate refresh tokens on use
   - Store refresh tokens in HttpOnly cookies

3. **Rate Limiting**
   - Limit registration attempts (5 per hour per IP)
   - Limit login attempts (10 per 15 minutes)
   - Limit password reset requests (3 per hour)

4. **Input Validation**
   - Validate all inputs using Joi or similar
   - Sanitize email addresses
   - Prevent SQL injection

5. **HTTPS Only**
   - Force HTTPS in production
   - Use secure cookies
   - Implement HSTS headers

## Error Handling

- Never expose internal error details
- Log errors for debugging
- Return generic error messages to clients
- Implement proper error recovery

## Testing Considerations

1. Unit test all service methods
2. Integration test authentication flows
3. Test token expiration and refresh
4. Test rate limiting
5. Security testing for common vulnerabilities
6. Load testing for authentication endpoints

## Performance Optimizations

1. Cache user data in Redis
2. Use connection pooling for database
3. Implement request batching
4. Use async/await properly
5. Index database on email field

## Monitoring

1. Track authentication success/failure rates
2. Monitor token refresh patterns
3. Alert on suspicious activity
4. Log all authentication events
5. Monitor Redis memory usage