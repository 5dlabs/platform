# Task 3: User Authentication System - Acceptance Criteria

## Functional Requirements

### 1. User Registration ✓
- [ ] POST /api/auth/register endpoint functional
- [ ] Email uniqueness enforced
- [ ] Username uniqueness enforced
- [ ] Password properly hashed with bcrypt
- [ ] Welcome email sent successfully
- [ ] JWT tokens returned on successful registration
- [ ] Proper validation error messages

### 2. User Login ✓
- [ ] POST /api/auth/login endpoint functional
- [ ] Email/password authentication works
- [ ] Invalid credentials return 401 error
- [ ] JWT access token generated (15 min expiry)
- [ ] Refresh token generated (7 day expiry)
- [ ] User online status updated
- [ ] Last login timestamp recorded

### 3. Token Management ✓
- [ ] Access tokens expire after 15 minutes
- [ ] Refresh tokens stored in Redis
- [ ] POST /api/auth/refresh endpoint works
- [ ] Old refresh token invalidated on refresh
- [ ] Token structure includes userId and type
- [ ] Invalid tokens properly rejected

### 4. Password Reset ✓
- [ ] POST /api/auth/forgot-password sends email
- [ ] Reset token generated and stored
- [ ] Reset token expires after 1 hour
- [ ] POST /api/auth/reset-password updates password
- [ ] Old password no longer works after reset
- [ ] Reset token single-use only

### 5. Protected Routes ✓
- [ ] Authentication middleware validates tokens
- [ ] GET /api/auth/profile requires valid token
- [ ] Expired tokens return 401 error
- [ ] Missing tokens return 401 error
- [ ] userId attached to authenticated requests

## Security Validation

### Password Security
```bash
# Test 1: Password hashing
Password: "Test123!" 
✓ Hashed with bcrypt (10+ rounds)
✓ Hash different each time (salt working)
✓ Original password not stored

# Test 2: Password validation
"weak" → ✗ Too short
"weakpassword" → ✗ No uppercase/number
"StrongPass123!" → ✓ Accepted
```

### Token Security
```javascript
// Test 1: Access token validation
const token = generateAccessToken(userId);
✓ Valid for 15 minutes
✓ Contains userId and type
✓ Signed with JWT_SECRET

// Test 2: Refresh token storage
await storeRefreshToken(userId, refreshToken);
✓ Stored in Redis with key refresh_token:{userId}
✓ TTL set to 7 days
✓ Previous token overwritten
```

### Rate Limiting
```bash
# Test 1: Registration rate limit
for i in 1..6; do
  curl -X POST /api/auth/register
done
✓ First 5 requests succeed
✓ 6th request returns 429 Too Many Requests

# Test 2: Login rate limit
✓ 10 attempts allowed per 15 minutes
✓ Rate limit per IP address
```

## API Endpoint Tests

### Registration Endpoint
```bash
# Valid registration
POST /api/auth/register
{
  "email": "user@example.com",
  "username": "testuser",
  "password": "SecurePass123!"
}

Response 201:
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "testuser"
  },
  "accessToken": "jwt...",
  "refreshToken": "jwt..."
}

# Duplicate email
Response 400:
{
  "errors": [
    { "field": "email", "message": "Email already registered" }
  ]
}
```

### Login Endpoint
```bash
# Valid login
POST /api/auth/login
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}

Response 200:
{
  "user": { ... },
  "accessToken": "jwt...",
  "refreshToken": "jwt..."
}

# Invalid credentials
Response 401:
{
  "error": "Invalid credentials"
}
```

### Token Refresh
```bash
# Valid refresh
POST /api/auth/refresh
{
  "refreshToken": "valid-refresh-token"
}

Response 200:
{
  "accessToken": "new-jwt...",
  "refreshToken": "new-refresh-jwt..."
}

# Invalid/expired token
Response 401:
{
  "error": "Invalid refresh token"
}
```

## Validation Requirements

### Input Validation Tests
- [ ] Email must be valid format
- [ ] Email normalized (lowercase, trimmed)
- [ ] Username 3-30 characters
- [ ] Username alphanumeric + underscore only
- [ ] Password minimum 8 characters
- [ ] Password requires upper, lower, number
- [ ] All fields sanitized for XSS
- [ ] SQL injection prevention active

### Validation Examples
```javascript
// Email validation
"test@example.com" → ✓ Valid
"TEST@EXAMPLE.COM" → ✓ Normalized to lowercase
"invalid-email" → ✗ Invalid format

// Username validation  
"user123" → ✓ Valid
"a" → ✗ Too short
"user@name" → ✗ Invalid characters

// Password validation
"Abc123!@" → ✓ Valid
"password" → ✗ No uppercase/number
"Pass123" → ✗ Too short
```

## Email Service Tests

### Welcome Email
- [ ] Sent on successful registration
- [ ] Contains username in greeting
- [ ] Professional HTML template
- [ ] Plain text fallback included
- [ ] From address configured correctly

### Password Reset Email
- [ ] Sent when requested
- [ ] Contains secure reset link
- [ ] Link includes valid token
- [ ] Link expires after 1 hour
- [ ] Clear instructions provided

## Performance Criteria

### Response Times
- [ ] Registration < 500ms (including email)
- [ ] Login < 100ms
- [ ] Token refresh < 50ms
- [ ] Profile fetch < 50ms

### Concurrency
- [ ] Handle 100 concurrent registrations
- [ ] Handle 1000 concurrent logins
- [ ] No race conditions in token storage
- [ ] Redis connections pooled properly

## Integration Tests

### Full Authentication Flow
```javascript
// 1. Register new user
const registerRes = await api.post('/auth/register', userData);
✓ User created in database
✓ Tokens returned
✓ Welcome email sent

// 2. Login with credentials
const loginRes = await api.post('/auth/login', credentials);
✓ Tokens match user
✓ Online status updated

// 3. Access protected route
const profileRes = await api.get('/auth/profile', {
  headers: { Authorization: `Bearer ${accessToken}` }
});
✓ User data returned
✓ Matches logged in user

// 4. Refresh token
const refreshRes = await api.post('/auth/refresh', { refreshToken });
✓ New tokens generated
✓ Old refresh token invalid

// 5. Logout
const logoutRes = await api.post('/auth/logout');
✓ Tokens revoked
✓ Redis entries cleared
```

## Error Handling Tests

### Authentication Errors
- [ ] Wrong password → 401 "Invalid credentials"
- [ ] Non-existent email → 401 "Invalid credentials"  
- [ ] Expired token → 401 "Token expired"
- [ ] Malformed token → 401 "Invalid token"
- [ ] Missing token → 401 "No token provided"

### Validation Errors
- [ ] Invalid email → 400 with field details
- [ ] Weak password → 400 with requirements
- [ ] Duplicate username → 400 with message
- [ ] Missing fields → 400 with field list

## Security Checklist

### Implementation Security
- [ ] No passwords in logs
- [ ] Secrets in environment variables
- [ ] HTTPS enforced in production
- [ ] CORS configured correctly
- [ ] Helmet.js security headers
- [ ] No user enumeration possible
- [ ] Timing attack prevention

### Monitoring & Logging
- [ ] Failed login attempts logged
- [ ] Successful logins logged
- [ ] Password resets logged
- [ ] Rate limit hits logged
- [ ] Token refresh events logged

## Final Verification

### Complete System Test
1. Register new user account
2. Receive welcome email
3. Login with credentials
4. Access protected endpoints
5. Let token expire
6. Refresh token successfully
7. Request password reset
8. Complete password reset
9. Login with new password
10. Logout successfully

**All steps must complete without errors for task acceptance.**

### Production Readiness
- [ ] All endpoints return consistent format
- [ ] Error messages don't leak information
- [ ] Performance meets requirements
- [ ] Security best practices followed
- [ ] Monitoring and logging active
- [ ] Documentation complete
- [ ] Tests achieve 90%+ coverage

**Task is complete when all authentication flows work securely and efficiently.**