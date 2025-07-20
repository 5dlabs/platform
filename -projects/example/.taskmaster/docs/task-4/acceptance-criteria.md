# Acceptance Criteria for Task 4: Implement User Management Service

## Functional Requirements

### User Registration
1. **Input Validation**
   - [ ] Email must be valid format (RFC 5322)
   - [ ] Email must be unique in database
   - [ ] Password minimum 8 characters
   - [ ] Password contains uppercase, lowercase, digit, and special character
   - [ ] Full name between 1-255 characters

2. **Registration Flow**
   - [ ] New user created in database with hashed password
   - [ ] JWT token generated and returned
   - [ ] Response includes user ID, email, full name, and role
   - [ ] Default role set to "member"
   - [ ] Timestamps set automatically

### User Login
1. **Authentication**
   - [ ] Email and password verified against database
   - [ ] Invalid credentials return generic error message
   - [ ] Successful login returns JWT token
   - [ ] Token includes user ID, email, and role claims

2. **Error Handling**
   - [ ] Non-existent email returns same error as wrong password
   - [ ] Empty fields return validation error
   - [ ] SQL injection attempts blocked

### Token Management
1. **JWT Tokens**
   - [ ] Tokens signed with secret from environment variable
   - [ ] Expiration set to 24 hours
   - [ ] Contains sub (user ID), email, role, exp, iat claims
   - [ ] Invalid tokens return 401 Unauthorized

2. **Token Validation**
   - [ ] Expired tokens rejected with appropriate error
   - [ ] Malformed tokens rejected
   - [ ] Valid tokens return user claims

### User Profile
1. **Profile Retrieval**
   - [ ] Requires valid authentication token
   - [ ] Returns user details without password hash
   - [ ] Handles non-existent user IDs gracefully

## Technical Requirements

### Security
1. **Password Handling**
   - [ ] Passwords hashed using Argon2 algorithm
   - [ ] Salt generated for each password
   - [ ] Plain text passwords never logged or stored
   - [ ] Password verification uses constant-time comparison

2. **JWT Security**
   - [ ] Secret key at least 256 bits
   - [ ] HS256 algorithm for signing
   - [ ] Token validation checks signature and expiration
   - [ ] No sensitive data in JWT payload

### Performance
1. **Response Times**
   - [ ] Registration endpoint < 200ms
   - [ ] Login endpoint < 200ms
   - [ ] Token validation < 50ms
   - [ ] Database queries use connection pool

2. **Concurrency**
   - [ ] Handles 100 concurrent requests
   - [ ] No race conditions in user creation
   - [ ] Database operations don't block async runtime

### Error Handling
1. **gRPC Status Codes**
   - [ ] InvalidArgument for validation errors
   - [ ] AlreadyExists for duplicate email
   - [ ] Unauthenticated for invalid credentials
   - [ ] Internal for server errors

2. **Error Messages**
   - [ ] User-friendly error descriptions
   - [ ] No sensitive information in errors
   - [ ] Consistent error format
   - [ ] Errors logged with context

## Test Cases

### Unit Tests

```rust
#[test]
fn test_password_hashing() {
    // Test: Hash password
    // Assert: Hash is different from original
    // Assert: Verify returns true for correct password
    // Assert: Verify returns false for wrong password
}

#[test]
fn test_jwt_generation() {
    // Test: Generate token with claims
    // Assert: Token is valid JWT format
    // Assert: Claims match input
    // Assert: Expiration is set correctly
}

#[test]
fn test_email_validation() {
    // Test valid emails: user@example.com, test.user+tag@domain.co.uk
    // Test invalid emails: notanemail, @example.com, user@
    // Assert: Validation results match expected
}

#[test]
fn test_password_strength_validation() {
    // Test weak passwords: "password", "12345678", "Password"
    // Test strong passwords: "Pass123!", "Str0ng!Pass"
    // Assert: Only strong passwords pass validation
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_complete_registration_flow() {
    // Setup: Clean test database
    // Test: Register new user
    // Assert: User created in database
    // Assert: Password is hashed
    // Assert: JWT token valid
    // Test: Login with same credentials
    // Assert: Login successful
}

#[tokio::test]
async fn test_duplicate_email_registration() {
    // Setup: Create user with email
    // Test: Register with same email
    // Assert: Returns AlreadyExists error
    // Assert: Only one user in database
}

#[tokio::test]
async fn test_invalid_login_attempts() {
    // Test: Login with non-existent email
    // Assert: Returns Unauthenticated
    // Test: Login with wrong password
    // Assert: Returns same error message
    // Test: Login with SQL injection attempt
    // Assert: Returns validation error
}

#[tokio::test]
async fn test_token_lifecycle() {
    // Test: Generate token
    // Assert: Token validates successfully
    // Test: Use expired token
    // Assert: Returns token expired error
    // Test: Use malformed token
    // Assert: Returns invalid token error
}
```

### Security Tests

```rust
#[tokio::test]
async fn test_no_password_leakage() {
    // Test: Register and check logs
    // Assert: No plain text password in logs
    // Test: Error responses
    // Assert: No password in error messages
}

#[tokio::test]
async fn test_timing_attack_resistance() {
    // Test: Time login with valid email/wrong password
    // Test: Time login with invalid email
    // Assert: Response times are similar
}

#[tokio::test]
async fn test_concurrent_registrations() {
    // Test: Register same email from 10 concurrent tasks
    // Assert: Only one succeeds
    // Assert: Others get AlreadyExists error
}
```

## Verification Steps

### Manual Testing

1. **Registration Testing**
   ```bash
   # Test valid registration
   grpcurl -plaintext -d '{
     "email": "test@example.com",
     "password": "SecurePass123!",
     "full_name": "Test User"
   }' localhost:50051 taskboard.UserService/RegisterUser
   
   # Verify JWT token in response
   # Verify user in database
   ```

2. **Login Testing**
   ```bash
   # Test valid login
   grpcurl -plaintext -d '{
     "email": "test@example.com",
     "password": "SecurePass123!"
   }' localhost:50051 taskboard.UserService/LoginUser
   
   # Test invalid credentials
   # Verify error messages don't leak info
   ```

3. **Token Validation**
   ```bash
   # Test valid token
   grpcurl -plaintext -d '{
     "token": "eyJ..."
   }' localhost:50051 taskboard.UserService/ValidateToken
   
   # Test expired/invalid tokens
   ```

### Database Verification

```sql
-- Check user creation
SELECT id, email, full_name, role, created_at 
FROM users 
WHERE email = 'test@example.com';

-- Verify password is hashed
SELECT password_hash 
FROM users 
WHERE email = 'test@example.com';
-- Should show Argon2 hash format

-- Check for duplicates
SELECT email, COUNT(*) 
FROM users 
GROUP BY email 
HAVING COUNT(*) > 1;
-- Should return empty
```

### Performance Verification

1. **Load Testing**
   ```bash
   # Run concurrent registration attempts
   ab -n 1000 -c 100 -p register.json http://localhost:50051/
   
   # Check response times
   # Verify no errors under load
   ```

2. **Database Pool Monitoring**
   ```rust
   // Log pool stats during tests
   info!("Active connections: {}", pool.state().connections);
   info!("Idle connections: {}", pool.state().idle_connections);
   ```

## Success Metrics

- **Functional Coverage**: 100% of user stories implemented
- **Test Coverage**: >95% code coverage for auth module
- **Security**: Zero security vulnerabilities in OWASP top 10
- **Performance**: All endpoints respond <200ms at p99
- **Reliability**: Zero crashes under normal load
- **Maintainability**: All functions documented, <10 cognitive complexity
