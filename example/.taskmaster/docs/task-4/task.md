# Task 4: Implement User Management Service

## Overview

This task implements the core user management functionality for the Task Board API, including user registration, authentication, and JWT token generation. The service provides secure authentication mechanisms and integrates with the gRPC interface defined in Task 2, using the database schema from Task 3.

## Architecture Context

Based on the system architecture:

### Authentication Stack
- **JWT Tokens**: jsonwebtoken crate (v8.3+) for token generation and validation
- **Password Hashing**: Argon2 or bcrypt for secure password storage
- **Token Structure**: Contains user ID, email, role, and expiration claims
- **Token Expiration**: Short-lived tokens (configurable, typically 24 hours)

### Authorization Model
- **Roles**: Admin and Member roles with different permissions
- **Token Validation**: Middleware for protecting endpoints
- **Secure Storage**: Recommendations for client-side token storage
- **Rate Limiting**: Protection against brute force attacks

### Service Integration
- **gRPC Interface**: Implements UserService trait from proto definitions
- **Database Layer**: Uses Diesel models for user persistence
- **Async Runtime**: Full Tokio integration for non-blocking operations
- **Error Handling**: Custom error types for authentication failures

## Implementation Details

### 1. gRPC Service Implementation

```rust
// src/services/user_service.rs
use tonic::{Request, Response, Status};
use crate::proto::user_service_server::UserService;
use crate::proto::{
    RegisterRequest, RegisterResponse,
    LoginRequest, LoginResponse,
    TokenRequest, TokenResponse,
    UserRequest, UserResponse
};

pub struct UserServiceImpl {
    db_pool: DbPool,
    jwt_secret: String,
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        // Implementation
    }

    async fn login_user(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        // Implementation
    }

    async fn validate_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        // Implementation
    }

    async fn get_user_profile(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // Implementation
    }
}
```

### 2. Password Hashing Implementation

```rust
// src/auth/password.rs
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

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

### 3. JWT Token Management

```rust
// src/auth/jwt.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub email: String,
    pub role: String,
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
}

pub struct JwtManager {
    secret: String,
    expiration_hours: i64,
}

impl JwtManager {
    pub fn new(secret: String, expiration_hours: i64) -> Self {
        Self { secret, expiration_hours }
    }

    pub fn generate_token(&self, user_id: Uuid, email: &str, role: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);
        
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }
}
```

### 4. Input Validation

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
    
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub full_name: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Check for at least one uppercase, lowercase, number, and special character
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        Err(ValidationError::new("Password must contain uppercase, lowercase, number, and special character"))
    }
}
```

### 5. Service Method Implementations

```rust
// Registration endpoint
async fn register_user(
    &self,
    request: Request<RegisterRequest>,
) -> Result<Response<RegisterResponse>, Status> {
    let req = request.into_inner();
    
    // Validate input
    let input = RegisterInput {
        email: req.email.clone(),
        password: req.password.clone(),
        full_name: req.full_name.clone(),
    };
    
    input.validate()
        .map_err(|e| Status::invalid_argument(e.to_string()))?;
    
    // Check if user exists
    let existing_user = self.check_user_exists(&req.email).await?;
    if existing_user {
        return Err(Status::already_exists("User with this email already exists"));
    }
    
    // Hash password
    let password_hash = PasswordManager::hash_password(&req.password)
        .map_err(|e| Status::internal(e.to_string()))?;
    
    // Create user in database
    let new_user = NewUser {
        email: &req.email,
        password_hash: &password_hash,
        full_name: &req.full_name,
        role: "member",
    };
    
    let user = self.create_user_in_db(new_user).await
        .map_err(|e| Status::internal(e.to_string()))?;
    
    // Generate JWT token
    let token = self.jwt_manager.generate_token(user.id, &user.email, &user.role)
        .map_err(|e| Status::internal(e.to_string()))?;
    
    Ok(Response::new(RegisterResponse {
        user_id: user.id.to_string(),
        token,
        email: user.email,
        full_name: user.full_name,
        role: user.role,
    }))
}
```

### 6. Error Handling

```rust
// src/errors/auth.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Password hashing failed: {0}")]
    HashingError(String),
    
    #[error("Invalid password hash: {0}")]
    InvalidHash(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
}

impl From<AuthError> for Status {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials | AuthError::UserNotFound => {
                Status::unauthenticated(err.to_string())
            }
            AuthError::TokenExpired => Status::unauthenticated("Token has expired"),
            AuthError::InvalidToken(_) => Status::unauthenticated("Invalid token"),
            _ => Status::internal(err.to_string()),
        }
    }
}
```

## Dependencies

### Task Dependencies
- **Task 2**: gRPC Service Contracts (UserService proto definition)
- **Task 3**: Database Schema and ORM (User model and database operations)

### Cargo Dependencies
```toml
[dependencies]
# Authentication
jsonwebtoken = "8.3"
argon2 = "0.5"
validator = { version = "0.16", features = ["derive"] }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"

# Error handling
thiserror = "1.0"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Tracing
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = PasswordManager::hash_password(password).unwrap();
        
        assert!(PasswordManager::verify_password(password, &hash).unwrap());
        assert!(!PasswordManager::verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let manager = JwtManager::new("test-secret".to_string(), 24);
        let user_id = Uuid::new_v4();
        
        let token = manager.generate_token(user_id, "test@example.com", "member").unwrap();
        let claims = manager.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "member");
    }

    #[test]
    fn test_input_validation() {
        let valid_input = RegisterInput {
            email: "test@example.com".to_string(),
            password: "ValidPass123!".to_string(),
            full_name: "Test User".to_string(),
        };
        
        assert!(valid_input.validate().is_ok());
        
        let invalid_email = RegisterInput {
            email: "invalid-email".to_string(),
            ..valid_input.clone()
        };
        
        assert!(invalid_email.validate().is_err());
    }
}
```

### 2. Integration Tests

```rust
#[tokio::test]
async fn test_user_registration_flow() {
    let service = create_test_service().await;
    
    let request = Request::new(RegisterRequest {
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
        full_name: "New User".to_string(),
    });
    
    let response = service.register_user(request).await.unwrap();
    let res = response.into_inner();
    
    assert!(!res.user_id.is_empty());
    assert!(!res.token.is_empty());
    assert_eq!(res.email, "newuser@example.com");
}

#[tokio::test]
async fn test_duplicate_registration_fails() {
    let service = create_test_service().await;
    
    // First registration
    let request1 = Request::new(RegisterRequest {
        email: "duplicate@example.com".to_string(),
        password: "SecurePass123!".to_string(),
        full_name: "User One".to_string(),
    });
    
    service.register_user(request1).await.unwrap();
    
    // Duplicate registration
    let request2 = Request::new(RegisterRequest {
        email: "duplicate@example.com".to_string(),
        password: "DifferentPass123!".to_string(),
        full_name: "User Two".to_string(),
    });
    
    let result = service.register_user(request2).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}
```

### 3. Security Tests

```rust
#[tokio::test]
async fn test_invalid_token_rejected() {
    let service = create_test_service().await;
    
    let request = Request::new(TokenRequest {
        token: "invalid.token.here".to_string(),
    });
    
    let result = service.validate_token(request).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn test_expired_token_rejected() {
    // Create token with past expiration
    // Validate it fails with appropriate error
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let service = create_test_service().await;
    
    let request = Request::new(LoginRequest {
        email: "admin@example.com' OR '1'='1".to_string(),
        password: "password".to_string(),
    });
    
    let result = service.login_user(request).await;
    assert!(result.is_err());
}
```

## Subtask Breakdown

1. **Design Registration Endpoint** - Define API schema and validation rules
2. **Design Login Endpoint** - Implement credential verification flow
3. **Implement Password Hashing** - Integrate Argon2 for secure storage
4. **Implement JWT Generation** - Create and sign tokens with claims
5. **Implement Input Validation** - Validate all user inputs
6. **Implement Error Handling** - Comprehensive error types and responses
7. **Write Automated Tests** - Unit and integration test coverage

## Success Metrics

- All authentication endpoints respond within 200ms
- Password hashing uses industry-standard algorithms
- JWT tokens include all required claims
- Input validation prevents malformed data
- Error messages don't leak sensitive information
- 100% test coverage for authentication logic
- Rate limiting prevents brute force attacks
