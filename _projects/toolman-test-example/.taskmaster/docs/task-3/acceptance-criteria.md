# Acceptance Criteria: User Authentication System

## Overview
This document defines the acceptance criteria for the user authentication system implementation.

## API Endpoints Criteria

### ✅ Registration Endpoint
- [ ] `POST /api/auth/register` endpoint exists
- [ ] Accepts email, username, and password
- [ ] Returns user object and JWT tokens
- [ ] HTTP 201 status on success
- [ ] HTTP 409 for duplicate email/username
- [ ] HTTP 400 for validation errors

### ✅ Login Endpoint
- [ ] `POST /api/auth/login` endpoint exists
- [ ] Accepts email and password
- [ ] Returns user object and JWT tokens
- [ ] Updates user online status
- [ ] HTTP 200 on success
- [ ] HTTP 401 for invalid credentials

### ✅ Token Refresh Endpoint
- [ ] `POST /api/auth/refresh` endpoint exists
- [ ] Accepts refresh token in body
- [ ] Returns new access and refresh tokens
- [ ] Revokes old refresh token
- [ ] HTTP 401 for invalid/expired token

### ✅ Profile Endpoints
- [ ] `GET /api/auth/profile` requires authentication
- [ ] Returns current user data
- [ ] `PUT /api/auth/profile` updates user data
- [ ] Username uniqueness validated on update
- [ ] HTTP 401 without valid token

### ✅ Password Reset Endpoints
- [ ] `POST /api/auth/password-reset/request` accepts email
- [ ] Sends reset email with valid link
- [ ] `POST /api/auth/password-reset/confirm` accepts token and new password
- [ ] Successfully updates password
- [ ] Revokes all refresh tokens after reset

## Security Implementation Criteria

### ✅ Password Hashing
- [ ] Bcrypt used with 12 salt rounds
- [ ] Plain passwords never stored
- [ ] Password comparison works correctly
- [ ] Timing-safe comparison used

### ✅ JWT Token Security
- [ ] Access tokens expire in 15 minutes
- [ ] Refresh tokens expire in 7 days
- [ ] Tokens signed with secure secrets
- [ ] Different secrets for access/refresh
- [ ] Token type validation implemented

### ✅ Input Validation
- [ ] Email format validated
- [ ] Password strength enforced (8+ chars, mixed case, number)
- [ ] Username format validated (3-20 chars, alphanumeric)
- [ ] SQL injection prevention
- [ ] XSS prevention in inputs

## Redis Integration Criteria

### ✅ Refresh Token Storage
- [ ] Tokens stored with pattern `refresh_token:{userId}:{tokenId}`
- [ ] 7-day TTL set automatically
- [ ] Token metadata stored (IP, user agent)
- [ ] Tokens retrievable for validation
- [ ] Tokens deletable on logout

### ✅ Password Reset Tokens
- [ ] Stored with pattern `password_reset:{hashedToken}`
- [ ] 1-hour TTL enforced
- [ ] Token deleted after use
- [ ] Hashed before storage

## Middleware Functionality Criteria

### ✅ Authentication Middleware
- [ ] Extracts Bearer token from Authorization header
- [ ] Validates access token
- [ ] Sets userId on request object
- [ ] Returns 401 for invalid tokens
- [ ] Handles missing tokens gracefully

### ✅ Rate Limiting
- [ ] Limits auth endpoints to 5 requests per 15 minutes
- [ ] Uses Redis for distributed rate limiting
- [ ] Returns 429 when limit exceeded
- [ ] Provides retry-after header
- [ ] Different limits for different endpoints

## Email Service Criteria

### ✅ Password Reset Emails
- [ ] SMTP configuration from environment
- [ ] HTML email template created
- [ ] Reset link includes valid token
- [ ] Email sent asynchronously
- [ ] Errors logged but don't crash app

## Error Handling Criteria

### ✅ Error Responses
- [ ] Consistent error format: `{ error: "message" }`
- [ ] Appropriate HTTP status codes
- [ ] No sensitive data in errors
- [ ] Internal errors logged
- [ ] User-friendly error messages

## Testing Checklist

### Unit Tests
```javascript
describe('Authentication', () => {
  describe('Registration', () => {
    it('creates user with valid data');
    it('rejects duplicate email');
    it('validates password strength');
    it('returns JWT tokens');
  });
  
  describe('Login', () => {
    it('authenticates valid credentials');
    it('rejects invalid password');
    it('updates online status');
  });
  
  describe('Token Management', () => {
    it('generates valid tokens');
    it('refreshes tokens correctly');
    it('validates token expiry');
    it('revokes tokens on logout');
  });
});
```

### Integration Tests
1. **Complete Registration Flow**
   ```bash
   curl -X POST http://localhost:3001/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","username":"testuser","password":"Test123!"}'
   ```

2. **Login and Token Usage**
   ```bash
   # Login
   curl -X POST http://localhost:3001/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","password":"Test123!"}'
   
   # Use access token
   curl -X GET http://localhost:3001/api/auth/profile \
     -H "Authorization: Bearer <access_token>"
   ```

3. **Token Refresh**
   ```bash
   curl -X POST http://localhost:3001/api/auth/refresh \
     -H "Content-Type: application/json" \
     -d '{"refreshToken":"<refresh_token>"}'
   ```

## Definition of Done

The task is complete when:
1. All authentication endpoints functional
2. JWT tokens generated and validated correctly
3. Passwords hashed with bcrypt
4. Refresh token rotation implemented
5. Password reset flow works end-to-end
6. Rate limiting prevents abuse
7. All security measures in place
8. Comprehensive error handling
9. All tests passing

## Common Issues to Avoid

- ❌ Storing plain passwords
- ❌ Using same secret for all tokens
- ❌ Not validating token type
- ❌ Exposing sensitive errors
- ❌ Missing rate limiting
- ❌ Synchronous bcrypt operations
- ❌ Tokens without expiry
- ❌ Not revoking tokens on password reset

## Security Verification

```bash
# Test rate limiting
for i in {1..10}; do
  curl -X POST http://localhost:3001/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"wrong"}'
done

# Verify token expiry
# Wait 16 minutes after login, then try to use access token
sleep 960 && curl -X GET http://localhost:3001/api/auth/profile \
  -H "Authorization: Bearer <old_access_token>"

# Check Redis storage
docker exec -it redis_container redis-cli
> KEYS refresh_token:*
> TTL refresh_token:userId:tokenId
```