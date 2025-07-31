# Task 3: User Authentication System - Autonomous AI Agent Prompt

## Objective

You are tasked with implementing a complete user authentication system for a web application. This system must provide secure user registration, login, token-based authentication using JWT, refresh token management, and password reset functionality. The implementation must follow security best practices and be production-ready.

## System Requirements

### Authentication Endpoints

Implement the following REST API endpoints:

1. **POST /api/auth/register**
   - Accept: email, password, name
   - Validate input data
   - Hash password using bcrypt (minimum 12 salt rounds)
   - Create user in database
   - Generate JWT access and refresh tokens
   - Store refresh token in Redis
   - Return: user data and access token
   - Set refresh token as HttpOnly cookie

2. **POST /api/auth/login**
   - Accept: email, password
   - Validate credentials
   - Verify password against hash
   - Generate new JWT tokens
   - Store refresh token in Redis
   - Return: user data and access token
   - Set refresh token as HttpOnly cookie

3. **POST /api/auth/refresh**
   - Accept: refresh token from cookie
   - Validate refresh token
   - Check token exists in Redis
   - Generate new access and refresh tokens
   - Rotate refresh token (delete old, store new)
   - Return: new access token
   - Update refresh token cookie

4. **GET /api/auth/profile**
   - Require: valid access token
   - Extract user ID from token
   - Fetch user data from database
   - Return: user profile information
   - Protected by authentication middleware

5. **POST /api/auth/reset-password**
   - Accept: email address
   - Generate unique reset token
   - Store token in Redis with 1-hour expiry
   - Send reset email with link
   - Return generic success message

6. **POST /api/auth/logout**
   - Require: valid access token
   - Revoke refresh token from Redis
   - Clear refresh token cookie
   - Return: success message

### JWT Token Management

1. **Access Tokens**
   - Expiry: 15 minutes
   - Contains: userId, email
   - Signed with JWT_SECRET
   - Sent in Authorization header

2. **Refresh Tokens**
   - Expiry: 7 days
   - Contains: userId, email
   - Signed with JWT_REFRESH_SECRET
   - Stored in HttpOnly cookie
   - Tracked in Redis

### Security Requirements

1. **Password Security**
   - Minimum 8 characters
   - Require uppercase, lowercase, number, special character
   - Hash with bcrypt (12+ rounds)
   - Never log or expose passwords

2. **Token Security**
   - Use environment variables for secrets
   - Minimum 32 character secrets
   - Different secrets for access and refresh
   - Implement token rotation

3. **Rate Limiting**
   - Registration: 5 attempts per hour per IP
   - Login: 10 attempts per 15 minutes
   - Password reset: 3 attempts per hour

4. **Data Validation**
   - Validate all inputs with Joi or similar
   - Sanitize email addresses
   - Prevent injection attacks
   - Return appropriate error messages

### Email Service Configuration

1. **Setup Requirements**
   - Configure SMTP transporter
   - Use environment variables for credentials
   - Support HTML email templates
   - Handle send failures gracefully

2. **Password Reset Email**
   - Include secure reset link
   - Set 1-hour expiration
   - Professional email template
   - Clear instructions for user

### Middleware Implementation

1. **Authentication Middleware**
   - Extract token from Authorization header
   - Verify JWT signature and expiry
   - Attach user data to request
   - Handle various error cases

2. **Error Handling**
   - Catch all exceptions
   - Log errors for debugging
   - Return generic error messages
   - Maintain security

### Database Schema

```typescript
interface User {
  id: string;
  email: string;
  password: string; // bcrypt hash
  name: string;
  createdAt: Date;
  updatedAt: Date;
}
```

### Redis Storage

- Key format: `refresh_token:{userId}`
- Value: refresh token string
- Expiry: 7 days
- Password reset tokens: `reset_token:{token}` -> userId

## Implementation Steps

1. Install required dependencies
2. Set up environment configuration
3. Create database models and migrations
4. Implement JWT service with token generation
5. Create authentication controller with all endpoints
6. Implement authentication middleware
7. Set up email service for password resets
8. Configure routes with proper middleware
9. Add input validation for all endpoints
10. Implement rate limiting
11. Create comprehensive tests
12. Document API endpoints

## Testing Requirements

1. **Unit Tests**
   - Test all service methods
   - Test token generation and validation
   - Test password hashing and verification
   - Test email sending functionality

2. **Integration Tests**
   - Complete registration flow
   - Login with valid/invalid credentials
   - Token refresh mechanism
   - Password reset flow
   - Protected route access

3. **Security Tests**
   - SQL injection attempts
   - XSS prevention
   - Rate limiting effectiveness
   - Token security

## Success Criteria

The authentication system is complete when:

1. All endpoints function correctly
2. Tokens are properly generated and validated
3. Refresh token rotation works
4. Password reset emails are sent
5. All security measures are in place
6. Comprehensive tests pass
7. Documentation is complete
8. Code follows best practices

## Additional Considerations

- Use TypeScript for type safety
- Follow RESTful conventions
- Implement proper logging
- Consider scalability
- Prepare for production deployment
- Monitor authentication metrics
- Plan for future OAuth integration

Begin by setting up the project structure and installing necessary dependencies. Ensure all environment variables are properly configured before implementation.