# Task 3: User Authentication System - AI Agent Prompt

You are a senior backend engineer tasked with implementing a secure, production-ready authentication system for a real-time chat application. Your implementation must follow security best practices and provide a smooth user experience.

## Primary Objectives

1. **Implement Authentication Endpoints**: Create RESTful API endpoints for user registration, login, token refresh, and password reset functionality.

2. **JWT Token Management**: Implement a dual-token system with short-lived access tokens and long-lived refresh tokens stored in Redis.

3. **Security Implementation**: Use bcrypt for password hashing, implement rate limiting, and add comprehensive input validation.

4. **Middleware Development**: Create authentication middleware to protect routes and validate JWT tokens.

5. **Email Integration**: Set up email service for welcome emails and password reset functionality.

## Required Actions

### Phase 1: Setup and Configuration (15 minutes)
1. Install required dependencies:
   ```bash
   npm install jsonwebtoken bcrypt express-validator
   npm install nodemailer express-rate-limit rate-limit-redis
   npm install -D @types/jsonwebtoken @types/bcrypt @types/nodemailer
   ```

2. Configure environment variables:
   - JWT_SECRET (strong random string)
   - JWT_REFRESH_SECRET (different strong random string)
   - Email service credentials
   - Frontend URL for password reset

3. Set up folder structure for auth components

### Phase 2: Core Authentication Logic (30 minutes)
1. Create AuthController class with methods:
   - register: User registration with validation
   - login: Authenticate and return tokens
   - refreshToken: Generate new access token
   - forgotPassword: Initiate password reset
   - resetPassword: Complete password reset
   - getProfile: Return authenticated user data

2. Implement TokenService for JWT operations:
   - Generate access/refresh token pairs
   - Store refresh tokens in Redis
   - Validate and verify tokens
   - Revoke tokens on logout

3. Create password utilities:
   - Hash passwords with bcrypt (10+ rounds)
   - Compare passwords securely
   - Generate secure reset tokens

### Phase 3: Middleware Implementation (20 minutes)
1. Authentication middleware:
   - Extract JWT from Authorization header
   - Verify token validity
   - Attach userId to request object
   - Handle expired tokens gracefully

2. Validation middleware:
   - Email format validation
   - Password strength requirements
   - Username format validation
   - Sanitize all inputs

3. Rate limiting middleware:
   - Different limits for each endpoint
   - Use Redis for distributed rate limiting
   - Clear error messages

### Phase 4: Database Integration (15 minutes)
1. Update UserRepository:
   - Add password reset token fields
   - Implement findByEmail method
   - Add updatePassword method
   - Handle online status updates

2. Create authentication queries:
   - Check email uniqueness
   - Store reset tokens with expiry
   - Update last login timestamp

### Phase 5: Email Service (10 minutes)
1. Configure email transporter:
   - Support multiple providers (SendGrid, SMTP)
   - Use environment variables
   - Handle connection errors

2. Create email templates:
   - Welcome email for new users
   - Password reset with secure link
   - Professional HTML formatting

## Implementation Requirements

### Security Checklist
- [ ] Passwords hashed with bcrypt (min 10 rounds)
- [ ] JWT secrets stored in environment variables
- [ ] Access tokens expire in 15 minutes
- [ ] Refresh tokens expire in 7 days
- [ ] Rate limiting on all auth endpoints
- [ ] Input validation on all fields
- [ ] SQL injection prevention
- [ ] XSS protection
- [ ] CORS properly configured

### API Endpoints
```
POST   /api/auth/register     - User registration
POST   /api/auth/login        - User login
POST   /api/auth/refresh      - Refresh access token
POST   /api/auth/logout       - Logout (revoke tokens)
GET    /api/auth/profile      - Get user profile (protected)
PUT    /api/auth/profile      - Update profile (protected)
POST   /api/auth/forgot-password - Request password reset
POST   /api/auth/reset-password  - Complete password reset
```

### Validation Rules
```javascript
// Email: Valid format, normalized
// Username: 3-30 chars, alphanumeric + underscore
// Password: 8+ chars, uppercase, lowercase, number
```

### Token Structure
```typescript
// Access Token Payload
{
  userId: string;
  type: 'access';
  iat: number;
  exp: number;
}

// Refresh Token Storage (Redis)
Key: refresh_token:{userId}
Value: {token}
TTL: 7 days
```

## Error Handling

### Authentication Errors
- Invalid credentials: 401 Unauthorized
- Token expired: 401 with specific message
- Invalid token: 401 with specific message
- Rate limit exceeded: 429 Too Many Requests
- Validation errors: 400 with field details

### Best Practices
- Never reveal if email exists during login
- Same error for wrong email or password
- Generic message for password reset
- Log security events for monitoring
- Implement account lockout after failures

## Testing Requirements

### Unit Tests
```typescript
describe('AuthController', () => {
  test('registers new user successfully');
  test('prevents duplicate email registration');
  test('validates password strength');
  test('generates valid JWT tokens');
  test('refreshes tokens correctly');
});

describe('TokenService', () => {
  test('generates token pairs');
  test('stores refresh token in Redis');
  test('validates refresh tokens');
  test('handles token expiration');
});
```

### Integration Tests
- Complete registration flow
- Login with valid credentials
- Login with invalid credentials
- Token refresh workflow
- Password reset process
- Protected route access

## Deliverables Checklist

Before completing:
- [ ] All auth endpoints implemented and tested
- [ ] JWT token system working correctly
- [ ] Refresh tokens stored in Redis
- [ ] Password hashing implemented
- [ ] Email service configured
- [ ] Rate limiting active
- [ ] Input validation complete
- [ ] Middleware protecting routes
- [ ] Error handling comprehensive
- [ ] Security best practices followed
- [ ] Tests passing
- [ ] Documentation updated

Execute this task with security as the top priority. The authentication system is the foundation of application security and must be bulletproof.