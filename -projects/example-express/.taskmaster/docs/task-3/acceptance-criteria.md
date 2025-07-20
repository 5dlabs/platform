# Task 3: Implement JWT Authentication - Acceptance Criteria

## Overview
This document defines the specific acceptance criteria for the JWT authentication system. Each criterion includes verification steps to ensure secure and functional authentication is properly implemented.

## Dependencies Criteria

### ✓ JWT Package Installed
- **Requirement**: jsonwebtoken package installed
- **Verification**:
  ```bash
  npm list jsonwebtoken
  ```
- **Expected Version**: ^9.0.2

### ✓ Bcrypt Already Available
- **Requirement**: bcrypt package available from Task 2
- **Verification**:
  ```bash
  npm list bcrypt
  ```
- **Expected Version**: ^5.1.1

## JWT Utility Criteria

### ✓ Token Generation Function
- **Requirement**: Can generate valid JWT tokens
- **Test**:
  ```javascript
  const { generateToken } = require('./src/utils/jwt');
  const token = generateToken({ userId: 1, email: 'test@example.com' });
  console.log(token.split('.').length === 3); // Should be true
  ```
- **Expected**: Valid JWT with three parts (header.payload.signature)

### ✓ Token Verification Function
- **Requirement**: Can verify and decode tokens
- **Test**:
  ```javascript
  const { generateToken, verifyToken } = require('./src/utils/jwt');
  const token = generateToken({ userId: 1 });
  const decoded = verifyToken(token);
  console.log(decoded.userId === 1); // Should be true
  ```
- **Expected**: Decoded payload matches original data

### ✓ Token Expiration Handling
- **Requirement**: Expired tokens are rejected
- **Test**: Create token with short expiration and verify after delay
- **Expected**: Throws "Token expired" error

### ✓ Invalid Token Handling
- **Requirement**: Malformed tokens are rejected
- **Test**:
  ```javascript
  const { verifyToken } = require('./src/utils/jwt');
  try {
    verifyToken('invalid.token.here');
  } catch (error) {
    console.log(error.message); // Should be "Invalid token"
  }
  ```
- **Expected**: Throws "Invalid token" error

## Password Utility Criteria

### ✓ Password Hashing
- **Requirement**: Passwords are hashed with bcrypt
- **Test**:
  ```javascript
  const { hashPassword } = require('./src/utils/password');
  const hash = await hashPassword('password123');
  console.log(hash.startsWith('$2b$')); // Should be true
  console.log(hash.length > 50); // Should be true
  ```
- **Expected**: Bcrypt hash format with salt

### ✓ Password Comparison
- **Requirement**: Can verify passwords against hashes
- **Test**:
  ```javascript
  const { hashPassword, comparePassword } = require('./src/utils/password');
  const hash = await hashPassword('password123');
  const valid = await comparePassword('password123', hash);
  const invalid = await comparePassword('wrongpassword', hash);
  console.log(valid === true && invalid === false); // Should be true
  ```
- **Expected**: Correct password returns true, wrong returns false

### ✓ Password Strength Validation
- **Requirement**: Enforces minimum password requirements
- **Test**:
  ```javascript
  const { validatePasswordStrength } = require('./src/utils/password');
  const weak = validatePasswordStrength('123');
  const strong = validatePasswordStrength('password123');
  console.log(!weak.valid && strong.valid); // Should be true
  ```
- **Expected**: 
  - Passwords < 8 chars are rejected
  - Passwords >= 8 chars are accepted

## Authentication Middleware Criteria

### ✓ Token Extraction from Header
- **Requirement**: Extracts Bearer token from Authorization header
- **Test**: Make request with "Authorization: Bearer TOKEN"
- **Expected**: Token is extracted and validated

### ✓ Missing Token Handling
- **Requirement**: Requests without token are rejected
- **Test**:
  ```bash
  curl http://localhost:3000/api/protected
  ```
- **Expected Response**:
  ```json
  {
    "error": {
      "message": "Access token required",
      "code": "TOKEN_REQUIRED"
    }
  }
  ```
- **Status Code**: 401

### ✓ Valid Token Processing
- **Requirement**: Valid tokens allow access
- **Test**: Use valid token from login
- **Expected**: 
  - Request proceeds
  - req.user contains userId and email

### ✓ User Verification
- **Requirement**: Middleware verifies user still exists
- **Test**: Use token for deleted user
- **Expected**: 401 with "User not found"

## Registration Endpoint Criteria

### ✓ Successful Registration
- **Requirement**: New users can register
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/auth/register \
    -H "Content-Type: application/json" \
    -d '{"email":"newuser@example.com","password":"password123"}'
  ```
- **Expected Response**:
  ```json
  {
    "message": "User registered successfully",
    "user": {
      "id": 1,
      "email": "newuser@example.com"
    },
    "tokens": {
      "accessToken": "...",
      "refreshToken": "...",
      "expiresIn": "24h"
    }
  }
  ```
- **Status Code**: 201

### ✓ Email Validation
- **Requirement**: Invalid emails are rejected
- **Test Cases**:
  - "notanemail" → 400 "Invalid email format"
  - "missing@" → 400 "Invalid email format"
  - "@example.com" → 400 "Invalid email format"
  - "valid@email.com" → Success

### ✓ Password Validation
- **Requirement**: Weak passwords are rejected
- **Test Cases**:
  - "" → 400 "Password is required"
  - "123" → 400 "Password must be at least 8 characters long"
  - "12345678" → Success

### ✓ Duplicate Email Prevention
- **Requirement**: Cannot register same email twice
- **Test**:
  1. Register user@example.com
  2. Try to register user@example.com again
- **Expected**: 409 "Email already registered"

### ✓ Email Case Normalization
- **Requirement**: Emails are stored lowercase
- **Test**:
  1. Register "Test@Example.com"
  2. Try to register "test@example.com"
- **Expected**: 409 duplicate error

## Login Endpoint Criteria

### ✓ Successful Login
- **Requirement**: Valid credentials return tokens
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"user@example.com","password":"password123"}'
  ```
- **Expected Response**:
  ```json
  {
    "message": "Login successful",
    "user": {
      "id": 1,
      "email": "user@example.com"
    },
    "tokens": {
      "accessToken": "...",
      "refreshToken": "...",
      "expiresIn": "24h"
    }
  }
  ```
- **Status Code**: 200

### ✓ Invalid Email Handling
- **Requirement**: Non-existent emails return generic error
- **Test**: Login with unregistered email
- **Expected**: 401 "Invalid credentials"
- **Note**: Same error as wrong password (security)

### ✓ Invalid Password Handling
- **Requirement**: Wrong passwords return generic error
- **Test**: Login with wrong password
- **Expected**: 401 "Invalid credentials"

### ✓ Missing Credentials
- **Requirement**: Empty fields are rejected
- **Test Cases**:
  - No email → 400 "Email and password are required"
  - No password → 400 "Email and password are required"
  - Empty body → 400 "Email and password are required"

## Token Refresh Criteria

### ✓ Successful Token Refresh
- **Requirement**: Valid refresh token generates new access token
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/auth/refresh \
    -H "Content-Type: application/json" \
    -d '{"refreshToken":"valid-refresh-token"}'
  ```
- **Expected Response**:
  ```json
  {
    "accessToken": "new-access-token",
    "expiresIn": "24h"
  }
  ```
- **Status Code**: 200

### ✓ Invalid Refresh Token
- **Requirement**: Invalid refresh tokens are rejected
- **Test**: Use expired or malformed refresh token
- **Expected**: 401 "Invalid refresh token"

### ✓ Missing Refresh Token
- **Requirement**: Request without token is rejected
- **Test**: Empty request body
- **Expected**: 400 "Refresh token required"

## Protected Route Criteria

### ✓ Protected Route Access
- **Requirement**: Valid token allows access
- **Test**:
  ```bash
  TOKEN="valid-access-token"
  curl http://localhost:3000/api/protected \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 200 with protected content

### ✓ Current User Endpoint
- **Requirement**: /auth/me returns user info
- **Test**: GET /auth/me with valid token
- **Expected Response**:
  ```json
  {
    "user": {
      "id": 1,
      "email": "user@example.com",
      "createdAt": "2025-07-20T12:00:00.000Z"
    }
  }
  ```

## Security Criteria

### ✓ Passwords Never Stored Plain Text
- **Requirement**: Database contains only hashes
- **Verification**:
  ```bash
  sqlite3 database.sqlite "SELECT password FROM users LIMIT 1;"
  ```
- **Expected**: Bcrypt hash starting with $2b$

### ✓ JWT Secret from Environment
- **Requirement**: JWT_SECRET used from .env
- **Test**: Change JWT_SECRET and restart
- **Expected**: Old tokens become invalid

### ✓ Consistent Error Messages
- **Requirement**: Errors don't leak user existence
- **Test**: Login with wrong email vs wrong password
- **Expected**: Both return identical "Invalid credentials"

### ✓ Token Expiration
- **Requirement**: Tokens expire after 24 hours
- **Verification**: Decode token and check exp claim
- **Expected**: exp is 24 hours from iat

## Integration Criteria

### ✓ Routes Integrated with Express
- **Requirement**: All auth routes accessible
- **Verification**:
  ```bash
  curl http://localhost:3000/auth/register -X POST
  curl http://localhost:3000/auth/login -X POST
  curl http://localhost:3000/auth/refresh -X POST
  curl http://localhost:3000/auth/me -X GET
  ```
- **Expected**: Routes respond (may be errors without proper data)

### ✓ Middleware Can Protect Any Route
- **Requirement**: authenticateToken works on any route
- **Test**: Add middleware to a test route
- **Expected**: Route requires valid token

## Test Summary Checklist

- [ ] JWT package installed (^9.0.2)
- [ ] Token generation creates valid JWTs
- [ ] Token verification decodes correctly
- [ ] Expired tokens are rejected
- [ ] Invalid tokens are rejected
- [ ] Passwords hash with bcrypt
- [ ] Password comparison works correctly
- [ ] Password strength enforced (8+ chars)
- [ ] Middleware extracts Bearer tokens
- [ ] Missing tokens return 401
- [ ] Valid tokens attach user to request
- [ ] Registration creates new users
- [ ] Email validation works
- [ ] Duplicate emails return 409
- [ ] Login with valid credentials returns tokens
- [ ] Login with invalid credentials returns 401
- [ ] Refresh token generates new access token
- [ ] Protected routes require authentication
- [ ] /auth/me returns current user
- [ ] Passwords stored as hashes only
- [ ] JWT secret from environment
- [ ] Error messages don't leak information
- [ ] Tokens expire in 24 hours

## Definition of Done

Task 3 is complete when:
1. All acceptance criteria above are met
2. Registration and login flows work end-to-end
3. JWT tokens are generated and validated correctly
4. Protected routes require valid authentication
5. Password security follows best practices
6. Error handling is consistent and secure
7. Integration with Express is seamless

## Notes

- Test with both valid and invalid data
- Verify security measures are in place
- Check that tokens work across server restarts
- Ensure error messages are user-friendly but secure
- Remember to use Authorization: Bearer TOKEN format