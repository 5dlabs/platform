# Task 3: User Authentication System

## Overview
Implement a comprehensive authentication system with JWT-based authorization, featuring user registration, secure login, token refresh mechanism, and password reset functionality. This system forms the security foundation for the chat application.

## Technical Architecture

### Authentication Stack
- **Token Management**: JWT for stateless authentication
- **Password Security**: bcrypt for hashing (min 10 rounds)
- **Session Storage**: Redis for refresh tokens
- **Email Service**: Nodemailer or SendGrid for password reset
- **Validation**: Express-validator for input validation

### Security Features
- Dual token system (access + refresh tokens)
- Secure password hashing with salt
- Rate limiting on auth endpoints
- Email verification for registration
- Secure password reset flow

## Implementation Details

### 1. Authentication Routes

#### Route Structure
```typescript
// backend/src/routes/auth.ts
import { Router } from 'express';
import { AuthController } from '../controllers/authController';
import { validateRequest } from '../middleware/validation';
import { rateLimiter } from '../middleware/rateLimiter';

const router = Router();
const authController = new AuthController();

// Public routes
router.post('/register', 
  rateLimiter('register'),
  validateRequest('register'),
  authController.register
);

router.post('/login',
  rateLimiter('login'),
  validateRequest('login'),
  authController.login
);

router.post('/refresh',
  authController.refreshToken
);

router.post('/forgot-password',
  rateLimiter('password-reset'),
  validateRequest('email'),
  authController.forgotPassword
);

router.post('/reset-password',
  validateRequest('reset-password'),
  authController.resetPassword
);

// Protected routes
router.get('/profile',
  authenticate,
  authController.getProfile
);

router.put('/profile',
  authenticate,
  validateRequest('update-profile'),
  authController.updateProfile
);

export default router;
```

### 2. JWT Implementation

#### Token Service
```typescript
// backend/src/services/tokenService.ts
import jwt from 'jsonwebtoken';
import { redis } from '../config/redis';
import { TokenPayload, TokenPair } from '../types/auth';

export class TokenService {
  private readonly ACCESS_TOKEN_EXPIRY = '15m';
  private readonly REFRESH_TOKEN_EXPIRY = '7d';
  private readonly REFRESH_TOKEN_EXPIRY_SECONDS = 7 * 24 * 60 * 60;

  generateTokenPair(userId: string): TokenPair {
    const payload: TokenPayload = { userId, type: 'access' };
    
    const accessToken = jwt.sign(
      payload,
      process.env.JWT_SECRET!,
      { expiresIn: this.ACCESS_TOKEN_EXPIRY }
    );

    const refreshPayload: TokenPayload = { userId, type: 'refresh' };
    const refreshToken = jwt.sign(
      refreshPayload,
      process.env.JWT_REFRESH_SECRET!,
      { expiresIn: this.REFRESH_TOKEN_EXPIRY }
    );

    return { accessToken, refreshToken };
  }

  async storeRefreshToken(userId: string, refreshToken: string): Promise<void> {
    const key = `refresh_token:${userId}`;
    await redis.setex(key, this.REFRESH_TOKEN_EXPIRY_SECONDS, refreshToken);
  }

  async validateRefreshToken(userId: string, token: string): Promise<boolean> {
    const key = `refresh_token:${userId}`;
    const storedToken = await redis.get(key);
    return storedToken === token;
  }

  async revokeRefreshToken(userId: string): Promise<void> {
    const key = `refresh_token:${userId}`;
    await redis.del(key);
  }

  verifyAccessToken(token: string): TokenPayload {
    return jwt.verify(token, process.env.JWT_SECRET!) as TokenPayload;
  }

  verifyRefreshToken(token: string): TokenPayload {
    return jwt.verify(token, process.env.JWT_REFRESH_SECRET!) as TokenPayload;
  }
}
```

### 3. Authentication Controller

```typescript
// backend/src/controllers/authController.ts
import { Request, Response } from 'express';
import bcrypt from 'bcrypt';
import { UserRepository } from '../repositories/userRepository';
import { TokenService } from '../services/tokenService';
import { EmailService } from '../services/emailService';
import { ValidationError, UnauthorizedError } from '../utils/errors';

export class AuthController {
  private userRepository = new UserRepository();
  private tokenService = new TokenService();
  private emailService = new EmailService();

  register = async (req: Request, res: Response): Promise<void> => {
    const { email, username, password } = req.body;

    // Check if user exists
    const existingUser = await this.userRepository.findByEmail(email);
    if (existingUser) {
      throw new ValidationError('Email already registered');
    }

    // Hash password
    const passwordHash = await bcrypt.hash(password, 10);

    // Create user
    const user = await this.userRepository.create({
      email,
      username,
      passwordHash,
      isOnline: false
    });

    // Generate tokens
    const { accessToken, refreshToken } = this.tokenService.generateTokenPair(user.id);
    await this.tokenService.storeRefreshToken(user.id, refreshToken);

    // Send welcome email
    await this.emailService.sendWelcomeEmail(email, username);

    res.status(201).json({
      user: {
        id: user.id,
        email: user.email,
        username: user.username
      },
      accessToken,
      refreshToken
    });
  };

  login = async (req: Request, res: Response): Promise<void> => {
    const { email, password } = req.body;

    // Find user
    const user = await this.userRepository.findByEmail(email);
    if (!user) {
      throw new UnauthorizedError('Invalid credentials');
    }

    // Verify password
    const isValidPassword = await bcrypt.compare(password, user.passwordHash);
    if (!isValidPassword) {
      throw new UnauthorizedError('Invalid credentials');
    }

    // Generate tokens
    const { accessToken, refreshToken } = this.tokenService.generateTokenPair(user.id);
    await this.tokenService.storeRefreshToken(user.id, refreshToken);

    // Update online status
    await this.userRepository.updateOnlineStatus(user.id, true);

    res.json({
      user: {
        id: user.id,
        email: user.email,
        username: user.username,
        avatarUrl: user.avatarUrl
      },
      accessToken,
      refreshToken
    });
  };

  refreshToken = async (req: Request, res: Response): Promise<void> => {
    const { refreshToken } = req.body;

    if (!refreshToken) {
      throw new UnauthorizedError('Refresh token required');
    }

    // Verify refresh token
    const payload = this.tokenService.verifyRefreshToken(refreshToken);
    
    // Validate stored token
    const isValid = await this.tokenService.validateRefreshToken(payload.userId, refreshToken);
    if (!isValid) {
      throw new UnauthorizedError('Invalid refresh token');
    }

    // Generate new token pair
    const tokens = this.tokenService.generateTokenPair(payload.userId);
    await this.tokenService.storeRefreshToken(payload.userId, tokens.refreshToken);

    res.json(tokens);
  };

  forgotPassword = async (req: Request, res: Response): Promise<void> => {
    const { email } = req.body;

    const user = await this.userRepository.findByEmail(email);
    if (!user) {
      // Don't reveal if user exists
      res.json({ message: 'If the email exists, a reset link has been sent' });
      return;
    }

    // Generate reset token
    const resetToken = this.generateResetToken();
    const resetExpiry = new Date(Date.now() + 3600000); // 1 hour

    // Store reset token
    await this.userRepository.setPasswordResetToken(user.id, resetToken, resetExpiry);

    // Send reset email
    await this.emailService.sendPasswordResetEmail(email, resetToken);

    res.json({ message: 'If the email exists, a reset link has been sent' });
  };

  private generateResetToken(): string {
    return crypto.randomBytes(32).toString('hex');
  }
}
```

### 4. Authentication Middleware

```typescript
// backend/src/middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import { TokenService } from '../services/tokenService';
import { UnauthorizedError } from '../utils/errors';

export interface AuthRequest extends Request {
  userId?: string;
}

const tokenService = new TokenService();

export const authenticate = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const authHeader = req.headers.authorization;
    
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      throw new UnauthorizedError('No token provided');
    }

    const token = authHeader.substring(7);
    const payload = tokenService.verifyAccessToken(token);
    
    req.userId = payload.userId;
    next();
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      throw new UnauthorizedError('Token expired');
    }
    if (error instanceof jwt.JsonWebTokenError) {
      throw new UnauthorizedError('Invalid token');
    }
    throw error;
  }
};
```

### 5. Input Validation

```typescript
// backend/src/validators/authValidators.ts
import { body, ValidationChain } from 'express-validator';

export const authValidators = {
  register: [
    body('email')
      .isEmail()
      .normalizeEmail()
      .withMessage('Valid email required'),
    body('username')
      .isLength({ min: 3, max: 30 })
      .matches(/^[a-zA-Z0-9_]+$/)
      .withMessage('Username must be 3-30 characters, alphanumeric and underscore only'),
    body('password')
      .isLength({ min: 8 })
      .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/)
      .withMessage('Password must be at least 8 characters with uppercase, lowercase and number')
  ],

  login: [
    body('email').isEmail().normalizeEmail(),
    body('password').notEmpty()
  ],

  resetPassword: [
    body('token').notEmpty(),
    body('password')
      .isLength({ min: 8 })
      .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/)
  ]
};
```

### 6. Rate Limiting

```typescript
// backend/src/middleware/rateLimiter.ts
import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import { redis } from '../config/redis';

export const rateLimiter = (name: string) => {
  const limits = {
    register: { windowMs: 15 * 60 * 1000, max: 5 },
    login: { windowMs: 15 * 60 * 1000, max: 10 },
    'password-reset': { windowMs: 60 * 60 * 1000, max: 3 }
  };

  const config = limits[name] || { windowMs: 15 * 60 * 1000, max: 100 };

  return rateLimit({
    store: new RedisStore({
      client: redis,
      prefix: `rate_limit:${name}:`
    }),
    ...config,
    message: 'Too many requests, please try again later'
  });
};
```

## Security Best Practices

### Password Security
- Minimum 8 characters with complexity requirements
- bcrypt with minimum 10 rounds
- Password history to prevent reuse
- Account lockout after failed attempts

### Token Security
- Short-lived access tokens (15 minutes)
- Refresh tokens stored in Redis with expiry
- Tokens invalidated on logout
- Secure random generation for reset tokens

### API Security
- Rate limiting on all auth endpoints
- Input validation and sanitization
- CORS properly configured
- HTTPS enforced in production

## Email Integration

### Email Service
```typescript
// backend/src/services/emailService.ts
import nodemailer from 'nodemailer';
import { config } from '../config/email';

export class EmailService {
  private transporter;

  constructor() {
    this.transporter = nodemailer.createTransport(config);
  }

  async sendWelcomeEmail(email: string, username: string): Promise<void> {
    await this.transporter.sendMail({
      from: config.from,
      to: email,
      subject: 'Welcome to Chat App!',
      html: this.getWelcomeTemplate(username)
    });
  }

  async sendPasswordResetEmail(email: string, token: string): Promise<void> {
    const resetUrl = `${process.env.FRONTEND_URL}/reset-password?token=${token}`;
    
    await this.transporter.sendMail({
      from: config.from,
      to: email,
      subject: 'Password Reset Request',
      html: this.getResetTemplate(resetUrl)
    });
  }
}
```

## Testing Strategy

### Unit Tests
- Token generation and validation
- Password hashing and comparison
- Input validation rules
- Repository methods

### Integration Tests
- Complete registration flow
- Login with valid/invalid credentials
- Token refresh mechanism
- Password reset flow

### Security Tests
- Rate limiting effectiveness
- SQL injection attempts
- Token manipulation
- Brute force protection