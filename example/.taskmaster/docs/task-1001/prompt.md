# AI Agent Prompt: Implement User Authentication System

## Task Context

You are tasked with implementing a comprehensive user authentication system for a microservice architecture. This system must provide secure authentication and authorization capabilities using JWT tokens, password hashing, and role-based access control.

## Primary Objective

Design and implement a complete user authentication system that includes:
- User registration with email verification
- Secure login/logout functionality
- JWT token-based authentication
- Role-based access control (RBAC)
- Password reset capabilities
- Security best practices implementation

## Technical Requirements

### Core Components to Implement

1. **Database Schema**
   - Users table with secure password storage
   - Roles and permissions tables
   - User-role and role-permission relationship tables
   - Audit trail tables for security events

2. **Authentication Service**
   - User registration with email verification
   - Login with credential validation
   - Password hashing using bcrypt (minimum 12 rounds)
   - JWT token generation and validation
   - Token refresh mechanism

3. **Authorization Middleware**
   - Route protection middleware
   - Role-based access control
   - Permission checking
   - Token validation and user context injection

4. **API Endpoints**
   - POST /auth/register - User registration
   - POST /auth/login - User login
   - POST /auth/logout - User logout
   - POST /auth/refresh - Token refresh
   - POST /auth/forgot-password - Password reset request
   - POST /auth/reset-password - Password reset completion

### Security Requirements

- Use bcrypt for password hashing with minimum 12 rounds
- Implement JWT tokens with RS256 algorithm
- Add rate limiting to prevent brute force attacks
- Implement account lockout after failed attempts
- Secure token storage and transmission
- Input validation and sanitization
- SQL injection prevention
- XSS protection

### Performance Requirements

- Authentication response time < 200ms
- Support for concurrent user sessions
- Efficient database queries with proper indexing
- Token caching for validation performance

## Implementation Approach

### Phase 1: Foundation Setup
1. Set up database schema with proper relationships
2. Create migration scripts and seed data
3. Implement core user model with validation
4. Set up password hashing utilities

### Phase 2: Core Authentication
1. Implement user registration with email verification
2. Build login functionality with credential validation
3. Create JWT token generation and validation
4. Add token refresh mechanism

### Phase 3: Authorization System
1. Implement role-based access control
2. Create permission checking middleware
3. Build route protection decorators
4. Add user context injection

### Phase 4: Security Hardening
1. Add rate limiting to authentication endpoints
2. Implement account lockout mechanisms
3. Create audit logging system
4. Add comprehensive input validation

## Code Structure Expectations

```
src/
├── auth/
│   ├── controllers/
│   │   ├── auth.controller.js
│   │   └── user.controller.js
│   ├── middleware/
│   │   ├── auth.middleware.js
│   │   └── rbac.middleware.js
│   ├── services/
│   │   ├── auth.service.js
│   │   ├── token.service.js
│   │   └── user.service.js
│   ├── models/
│   │   ├── user.model.js
│   │   ├── role.model.js
│   │   └── permission.model.js
│   └── utils/
│       ├── password.utils.js
│       └── validation.utils.js
├── database/
│   ├── migrations/
│   └── seeders/
└── tests/
    ├── unit/
    └── integration/
```

## API Response Formats

### Successful Login Response
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "firstName": "John",
      "lastName": "Doe",
      "roles": ["user"]
    },
    "tokens": {
      "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
      "refreshToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
      "expiresIn": 900
    }
  }
}
```

### Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password",
    "details": {}
  }
}
```

## Testing Requirements

### Unit Tests (Minimum Coverage: 90%)
- Password hashing and validation
- JWT token generation and validation
- Role and permission checking
- Input validation functions
- Database model operations

### Integration Tests
- Complete authentication flow
- API endpoint functionality
- Middleware integration
- Database operations
- Error handling scenarios

### Security Tests
- Password strength validation
- Token manipulation attempts
- Rate limiting effectiveness
- SQL injection prevention
- XSS protection validation

## Environment Configuration

Required environment variables:
```env
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRATION=900
JWT_REFRESH_EXPIRATION=604800
BCRYPT_ROUNDS=12
DB_HOST=localhost
DB_PORT=5432
DB_NAME=auth_db
DB_USER=auth_user
DB_PASSWORD=secure_password
REDIS_URL=redis://localhost:6379
EMAIL_SERVICE_URL=http://email-service:3000
RATE_LIMIT_WINDOW=60000
RATE_LIMIT_MAX_REQUESTS=5
```

## Quality Assurance Checklist

Before marking this task complete, ensure:

- [ ] All API endpoints are implemented and tested
- [ ] Database schema is properly designed with relationships
- [ ] Password hashing uses bcrypt with minimum 12 rounds
- [ ] JWT tokens are properly generated and validated
- [ ] Role-based access control is functional
- [ ] Rate limiting is implemented and configured
- [ ] Account lockout mechanism is working
- [ ] Input validation prevents common attacks
- [ ] Error handling provides appropriate responses
- [ ] Logging captures security events
- [ ] Tests achieve minimum 90% coverage
- [ ] Documentation is complete and accurate
- [ ] Code follows established patterns and conventions
- [ ] Security best practices are implemented
- [ ] Performance requirements are met

## Success Metrics

- Authentication response time < 200ms
- Zero critical security vulnerabilities
- Test coverage > 90%
- All integration tests passing
- Proper error handling for all edge cases
- Complete audit trail for security events

## Important Notes

1. **Security First**: Never compromise on security for convenience
2. **Performance**: Optimize database queries and token validation
3. **Scalability**: Design for horizontal scaling
4. **Maintainability**: Write clean, well-documented code
5. **Testing**: Comprehensive test coverage is mandatory
6. **Documentation**: Keep all documentation up to date

Begin implementation by setting up the database schema and core authentication service. Focus on security best practices throughout the development process.