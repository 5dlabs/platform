# Autonomous Agent Prompt: User Authentication System

You are tasked with implementing a comprehensive authentication system for a chat application using JWT tokens, bcrypt for password hashing, and Redis for session management.

## Objective
Build a secure, scalable authentication system with user registration, login, JWT token management with refresh tokens, and password reset functionality.

## Detailed Requirements

### 1. Authentication Endpoints
Create the following API endpoints:
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `POST /api/auth/refresh` - Refresh access token
- `GET /api/auth/profile` - Get current user profile (protected)
- `PUT /api/auth/profile` - Update user profile (protected)
- `POST /api/auth/password-reset/request` - Request password reset
- `POST /api/auth/password-reset/confirm` - Confirm password reset
- `POST /api/auth/logout` - Logout user (protected)

### 2. User Registration
Implement registration with:
- Email and username validation
- Check for existing users
- Password strength requirements (8+ chars, uppercase, lowercase, number)
- Bcrypt password hashing (salt rounds: 12)
- Return user object and JWT tokens

### 3. JWT Token Implementation
Create dual-token system:
- **Access Token**: 15-minute expiry, contains userId
- **Refresh Token**: 7-day expiry, stored in Redis
- Store refresh tokens with metadata (IP, user agent, timestamp)
- Implement token rotation on refresh

Token structure:
```typescript
interface TokenPayload {
  userId: string;
  type: 'access' | 'refresh';
}
```

### 4. Authentication Middleware
Create middleware for:
- Extracting and validating Bearer tokens
- Setting userId on request object
- Optional authentication for public routes
- Proper error responses for invalid tokens

### 5. Password Reset Flow
Implement secure password reset:
1. Generate cryptographically secure reset token
2. Hash token before storing in Redis (1-hour expiry)
3. Send reset email with frontend URL
4. Validate token and update password
5. Revoke all refresh tokens after reset

### 6. User Profile Management
Allow users to:
- View their profile information
- Update username (with uniqueness check)
- Update avatar URL
- Cannot change email directly

### 7. Security Measures
Implement:
- Rate limiting on auth endpoints (5 attempts per 15 minutes)
- Input validation for all fields
- Secure password requirements
- Token revocation on logout
- No sensitive data in responses

### 8. Redis Integration
Use Redis for:
- Refresh token storage: `refresh_token:{userId}:{tokenId}`
- Password reset tokens: `password_reset:{hashedToken}`
- Rate limiting data: `rate_limit:auth:{identifier}`

### 9. Error Handling
Provide appropriate error responses:
- 400: Validation errors
- 401: Invalid credentials/tokens
- 409: Duplicate email/username
- 429: Rate limit exceeded
- 500: Server errors (log but don't expose details)

### 10. Email Service
Set up email functionality:
- Configure SMTP settings from environment
- HTML email template for password reset
- Async email sending
- Error handling for failed sends

## Expected Deliverables

1. Authentication controller with all endpoints
2. JWT utility functions for token management
3. Authentication middleware
4. Password reset controller
5. Email service implementation
6. Input validators
7. Rate limiting middleware
8. Updated routes configuration
9. TypeScript interfaces for auth types

## Implementation Standards

### Code Organization
```
backend/src/
├── controllers/
│   ├── authController.ts
│   └── passwordResetController.ts
├── middleware/
│   ├── auth.ts
│   └── rateLimiter.ts
├── services/
│   └── emailService.ts
├── utils/
│   ├── jwt.ts
│   └── validators.ts
└── routes/
    └── auth.ts
```

### Security Checklist
- [ ] Passwords hashed with bcrypt (12 rounds)
- [ ] JWT secrets in environment variables
- [ ] Refresh tokens stored securely in Redis
- [ ] Rate limiting on sensitive endpoints
- [ ] Input validation on all user inputs
- [ ] No sensitive data in error messages
- [ ] Tokens expire appropriately
- [ ] Password reset tokens are single-use

## Testing Requirements

Write tests for:
1. User registration with valid/invalid data
2. Login with correct/incorrect credentials
3. Token refresh flow
4. Protected route access
5. Password reset request and confirmation
6. Rate limiting behavior
7. Token expiration handling

## Verification Steps

1. Test registration: Create new user account
2. Test login: Authenticate with credentials
3. Test token refresh: Exchange refresh token for new access token
4. Test protected routes: Access with/without valid token
5. Test password reset: Complete full reset flow
6. Test rate limiting: Exceed request limits
7. Verify Redis storage: Check token storage patterns
8. Test logout: Ensure token revocation

Begin by setting up the authentication routes and controllers, then implement JWT token management, followed by middleware and security features.