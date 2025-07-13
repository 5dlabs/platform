# Autonomous Prompt for Task 4: Implement User Management Service

## Context

You are implementing the user management service for the Task Board API, which handles user registration, authentication, and JWT token generation. This service is critical for securing the application and managing user access to boards and tasks.

### Project State
- **Completed**: Task 1 (Project Setup), Task 2 (gRPC Service Contracts), Task 3 (Database Schema)
- **Current Task**: Implement the UserService gRPC endpoints with secure authentication
- **Tech Stack**: Rust, Tonic (gRPC), PostgreSQL with Diesel ORM, JWT authentication

### Service Responsibilities
1. User registration with email/password
2. User login with JWT token generation
3. Token validation for protected endpoints
4. User profile retrieval
5. Password security with proper hashing

## Task Requirements

### Functional Requirements

1. **User Registration**
   - Accept email, password, and full name
   - Validate email format and uniqueness
   - Enforce password strength requirements
   - Hash passwords before storage
   - Return JWT token on successful registration

2. **User Login**
   - Verify email and password combination
   - Generate JWT token with appropriate claims
   - Handle invalid credentials gracefully
   - Implement rate limiting for brute force protection

3. **Token Management**
   - Generate tokens with user ID, email, role claims
   - Set reasonable expiration (24 hours recommended)
   - Validate tokens on protected endpoints
   - Handle expired tokens appropriately

4. **Security Requirements**
   - Use Argon2 or bcrypt for password hashing
   - Never store or log plain text passwords
   - Implement proper error messages (no information leakage)
   - Validate all inputs to prevent injection attacks

### Technical Requirements

1. **gRPC Implementation**
   - Implement all methods in UserService trait
   - Use proper Status codes for errors
   - Handle async operations correctly
   - Integrate with database pool from Task 3

2. **Dependencies**
   - jsonwebtoken = "8.3" for JWT handling
   - argon2 = "0.5" for password hashing
   - validator = "0.16" for input validation
   - thiserror = "1.0" for error types

## Implementation Instructions

### Step 1: Set Up Project Structure
```
src/
├── services/
│   └── user_service.rs
├── auth/
│   ├── mod.rs
│   ├── password.rs
│   └── jwt.rs
├── validation/
│   └── user.rs
└── errors/
    └── auth.rs
```

### Step 2: Implement Password Manager
```rust
// src/auth/password.rs
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

pub struct PasswordManager;

impl PasswordManager {
    pub fn hash_password(password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::HashingError(e.to_string()))?;
        
        Ok(password_hash.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::InvalidHash(e.to_string()))?;
        
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}
```

### Step 3: Implement JWT Manager
- Create Claims struct with necessary fields
- Implement token generation with expiration
- Implement token validation with proper error handling
- Use environment variable for JWT secret

### Step 4: Create Input Validation
```rust
// src/validation/user.rs
use validator::{Validate, ValidationError};

#[derive(Debug, Validate)]
pub struct RegisterInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[validate(custom = "validate_password_strength")]
    pub password: String,
    
    #[validate(length(min = 1, max = 255))]
    pub full_name: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Check for uppercase, lowercase, digit, and special character
}
```

### Step 5: Implement UserService
1. Create UserServiceImpl struct with db_pool and jwt_manager
2. Implement register_user method:
   - Validate input
   - Check for existing user
   - Hash password
   - Create database record
   - Generate JWT token
   - Return response

3. Implement login_user method:
   - Validate credentials
   - Verify password hash
   - Generate JWT token
   - Return response

4. Implement validate_token method:
   - Parse and validate JWT
   - Return user claims

5. Implement get_user_profile method:
   - Validate authorization
   - Fetch user from database
   - Return profile data

### Step 6: Error Handling
- Create custom AuthError enum with thiserror
- Implement conversion to tonic::Status
- Ensure errors don't leak sensitive information
- Log errors appropriately with tracing

### Step 7: Database Integration
```rust
// Example database query wrapper
async fn check_user_exists(&self, email: &str) -> Result<bool, Status> {
    run_async_query(&self.db_pool, move |conn| {
        use crate::schema::users::dsl::*;
        
        users
            .filter(email.eq(email))
            .select(id)
            .first::<Uuid>(conn)
            .optional()
            .map(|u| u.is_some())
    })
    .await
    .map_err(|e| Status::internal(e.to_string()))
}
```

### Step 8: Write Comprehensive Tests
1. Unit tests for password hashing
2. Unit tests for JWT generation/validation
3. Integration tests for registration flow
4. Integration tests for login flow
5. Security tests for SQL injection
6. Tests for rate limiting

## Success Criteria

### Must Complete
- [ ] All UserService gRPC methods implemented
- [ ] Password hashing with Argon2 working
- [ ] JWT generation and validation functional
- [ ] Input validation prevents malformed data
- [ ] Database integration for user CRUD
- [ ] Proper error handling with no info leakage
- [ ] All tests passing

### Quality Checks
- [ ] No plain text passwords in logs or storage
- [ ] JWT secret loaded from environment
- [ ] Async operations don't block runtime
- [ ] Error messages are user-friendly
- [ ] Code follows Rust idioms
- [ ] Proper use of Result types

### Security Validation
- [ ] SQL injection attempts blocked
- [ ] Weak passwords rejected
- [ ] Duplicate emails handled correctly
- [ ] Expired tokens rejected
- [ ] Invalid tokens return 401 Unauthorized
- [ ] Rate limiting prevents brute force

## Common Pitfalls to Avoid

1. **Password Security**
   - Never log passwords
   - Always hash before storage
   - Use constant-time comparison

2. **JWT Handling**
   - Don't hardcode secrets
   - Validate expiration
   - Include necessary claims only

3. **Error Messages**
   - Don't reveal if email exists
   - Use generic "invalid credentials"
   - Log details, return generic errors

4. **Database Operations**
   - Use prepared statements
   - Handle connection pool properly
   - Don't block async runtime

5. **Input Validation**
   - Validate before processing
   - Sanitize all inputs
   - Check field lengths

## Testing Commands

```bash
# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test user_service_test

# Test specific endpoint
cargo test test_user_registration_flow

# Run with logging
RUST_LOG=debug cargo test
```

## Expected Deliverables

1. Complete UserService implementation
2. Password hashing utilities
3. JWT token management
4. Input validation structs
5. Custom error types
6. Comprehensive test suite
7. Database query functions
8. Logging instrumentation

Remember to follow Rust best practices, use proper error handling, and ensure all async operations are non-blocking. The service should be secure, performant, and maintainable.
