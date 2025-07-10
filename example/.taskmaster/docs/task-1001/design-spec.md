# Technical Design Specification: User Authentication System

## 1. System Architecture Overview

### 1.1 High-Level Architecture

The user authentication system follows a microservice architecture pattern with the following components:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client App    │    │   API Gateway   │    │  Auth Service   │
│   (Web/Mobile)  │◄──►│  (Rate Limit)   │◄──►│  (Node.js/JWT)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                       ┌─────────────────┐             │
                       │  Email Service  │◄────────────┤
                       │  (Verification) │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │   Redis Cache   │◄────────────┤
                       │ (Token/Session) │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  PostgreSQL DB  │◄────────────┘
                       │  (User/Role)    │
                       └─────────────────┘
```

### 1.2 Component Responsibilities

- **Auth Service**: Core authentication logic, token management, user operations
- **API Gateway**: Request routing, rate limiting, initial request validation
- **Email Service**: Email verification, password reset notifications
- **Redis Cache**: Token blacklisting, session management, rate limiting counters
- **PostgreSQL**: Persistent user data, roles, permissions, audit logs

## 2. Database Design

### 2.1 Entity Relationship Diagram

```
Users (1) ──── (M) User_Roles (M) ──── (1) Roles
                                         │
                                         │
                                         │
                                  (M) Role_Permissions (M) ──── (1) Permissions
                                         │
                                         │
                                         │
                                   (1) Audit_Logs
```

### 2.2 Database Schema

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    email_verification_expires TIMESTAMP,
    password_reset_token VARCHAR(255),
    password_reset_expires TIMESTAMP,
    failed_login_attempts INTEGER DEFAULT 0,
    account_locked_until TIMESTAMP,
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT users_email_check CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_verification_token ON users(email_verification_token);
CREATE INDEX idx_users_reset_token ON users(password_reset_token);
```

#### Roles Table
```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    is_system_role BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT roles_name_check CHECK (name ~* '^[a-zA-Z0-9_-]+$')
);

CREATE INDEX idx_roles_name ON roles(name);
```

#### Permissions Table
```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT permissions_name_check CHECK (name ~* '^[a-zA-Z0-9_:-]+$')
);

CREATE INDEX idx_permissions_resource_action ON permissions(resource, action);
```

#### User_Roles Table
```sql
CREATE TABLE user_roles (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    assigned_by UUID REFERENCES users(id),
    
    PRIMARY KEY (user_id, role_id)
);
```

#### Role_Permissions Table
```sql
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    granted_by UUID REFERENCES users(id),
    
    PRIMARY KEY (role_id, permission_id)
);
```

#### Audit_Logs Table
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    resource VARCHAR(50),
    resource_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    details JSONB,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT audit_logs_action_check CHECK (action IN (
        'login', 'logout', 'register', 'password_change', 'password_reset',
        'email_verify', 'role_assign', 'role_revoke', 'permission_grant',
        'permission_revoke', 'account_lock', 'account_unlock'
    ))
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
```

## 3. API Design

### 3.1 Authentication Endpoints

#### POST /auth/register
```javascript
// Request
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "firstName": "John",
  "lastName": "Doe"
}

// Response (201 Created)
{
  "success": true,
  "message": "Registration successful. Please check your email to verify your account.",
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "firstName": "John",
      "lastName": "Doe",
      "emailVerified": false
    }
  }
}
```

#### POST /auth/login
```javascript
// Request
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}

// Response (200 OK)
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "firstName": "John",
      "lastName": "Doe",
      "roles": ["user"],
      "permissions": ["read:profile", "update:profile"]
    },
    "tokens": {
      "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
      "refreshToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
      "expiresIn": 900
    }
  }
}
```

#### POST /auth/refresh
```javascript
// Request
{
  "refreshToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."
}

// Response (200 OK)
{
  "success": true,
  "data": {
    "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
    "expiresIn": 900
  }
}
```

### 3.2 JWT Token Structure

#### Access Token Payload
```javascript
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["user", "admin"],
  "permissions": ["read:profile", "write:profile", "delete:account"],
  "iat": 1640995200,
  "exp": 1640996100,
  "aud": "api.example.com",
  "iss": "auth.example.com",
  "jti": "token-uuid"
}
```

#### Refresh Token Payload
```javascript
{
  "sub": "user-uuid",
  "type": "refresh",
  "iat": 1640995200,
  "exp": 1641600000,
  "aud": "api.example.com",
  "iss": "auth.example.com",
  "jti": "refresh-token-uuid"
}
```

## 4. Security Implementation

### 4.1 Password Security

#### Password Hashing
```javascript
const bcrypt = require('bcrypt');
const SALT_ROUNDS = 12;

async function hashPassword(password) {
    const salt = await bcrypt.genSalt(SALT_ROUNDS);
    return bcrypt.hash(password, salt);
}

async function verifyPassword(password, hash) {
    return bcrypt.compare(password, hash);
}
```

#### Password Strength Requirements
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character
- Not in common password dictionary

### 4.2 JWT Token Security

#### Token Generation
```javascript
const jwt = require('jsonwebtoken');
const crypto = require('crypto');

function generateTokenPair(user) {
    const accessTokenPayload = {
        sub: user.id,
        email: user.email,
        roles: user.roles,
        permissions: user.permissions,
        jti: crypto.randomUUID()
    };

    const refreshTokenPayload = {
        sub: user.id,
        type: 'refresh',
        jti: crypto.randomUUID()
    };

    const accessToken = jwt.sign(accessTokenPayload, process.env.JWT_PRIVATE_KEY, {
        algorithm: 'RS256',
        expiresIn: process.env.JWT_EXPIRATION || '15m',
        audience: process.env.JWT_AUDIENCE,
        issuer: process.env.JWT_ISSUER
    });

    const refreshToken = jwt.sign(refreshTokenPayload, process.env.JWT_PRIVATE_KEY, {
        algorithm: 'RS256',
        expiresIn: process.env.JWT_REFRESH_EXPIRATION || '7d',
        audience: process.env.JWT_AUDIENCE,
        issuer: process.env.JWT_ISSUER
    });

    return { accessToken, refreshToken };
}
```

### 4.3 Rate Limiting Configuration

```javascript
const rateLimit = require('express-rate-limit');

const authLimiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 5, // limit each IP to 5 requests per windowMs
    message: 'Too many authentication attempts, please try again later.',
    standardHeaders: true,
    legacyHeaders: false
});

const registerLimiter = rateLimit({
    windowMs: 60 * 60 * 1000, // 1 hour
    max: 3, // limit each IP to 3 registration attempts per hour
    message: 'Too many registration attempts, please try again later.'
});
```

### 4.4 Account Lockout Implementation

```javascript
const MAX_LOGIN_ATTEMPTS = 5;
const LOCKOUT_TIME = 30 * 60 * 1000; // 30 minutes

async function handleFailedLogin(userId) {
    await db.query(`
        UPDATE users 
        SET failed_login_attempts = failed_login_attempts + 1,
            account_locked_until = CASE 
                WHEN failed_login_attempts + 1 >= $1 
                THEN NOW() + INTERVAL '30 minutes'
                ELSE account_locked_until
            END
        WHERE id = $2
    `, [MAX_LOGIN_ATTEMPTS, userId]);
}

async function isAccountLocked(userId) {
    const result = await db.query(`
        SELECT account_locked_until 
        FROM users 
        WHERE id = $1
    `, [userId]);
    
    const lockoutTime = result.rows[0]?.account_locked_until;
    return lockoutTime && new Date() < lockoutTime;
}
```

## 5. Middleware Implementation

### 5.1 Authentication Middleware

```javascript
const jwt = require('jsonwebtoken');
const redis = require('redis');

async function authenticateToken(req, res, next) {
    const authHeader = req.headers['authorization'];
    const token = authHeader && authHeader.split(' ')[1];

    if (!token) {
        return res.status(401).json({
            success: false,
            error: { code: 'MISSING_TOKEN', message: 'Access token required' }
        });
    }

    try {
        // Check if token is blacklisted
        const isBlacklisted = await redis.get(`blacklist:${token}`);
        if (isBlacklisted) {
            return res.status(401).json({
                success: false,
                error: { code: 'TOKEN_BLACKLISTED', message: 'Token has been revoked' }
            });
        }

        const decoded = jwt.verify(token, process.env.JWT_PUBLIC_KEY, {
            algorithms: ['RS256'],
            audience: process.env.JWT_AUDIENCE,
            issuer: process.env.JWT_ISSUER
        });

        req.user = decoded;
        next();
    } catch (error) {
        return res.status(401).json({
            success: false,
            error: { code: 'INVALID_TOKEN', message: 'Invalid or expired token' }
        });
    }
}
```

### 5.2 Authorization Middleware

```javascript
function requirePermission(permission) {
    return (req, res, next) => {
        if (!req.user) {
            return res.status(401).json({
                success: false,
                error: { code: 'UNAUTHORIZED', message: 'Authentication required' }
            });
        }

        if (!req.user.permissions.includes(permission)) {
            return res.status(403).json({
                success: false,
                error: { code: 'FORBIDDEN', message: 'Insufficient permissions' }
            });
        }

        next();
    };
}

function requireRole(role) {
    return (req, res, next) => {
        if (!req.user) {
            return res.status(401).json({
                success: false,
                error: { code: 'UNAUTHORIZED', message: 'Authentication required' }
            });
        }

        if (!req.user.roles.includes(role)) {
            return res.status(403).json({
                success: false,
                error: { code: 'FORBIDDEN', message: 'Insufficient role privileges' }
            });
        }

        next();
    };
}
```

## 6. Error Handling

### 6.1 Error Code Standards

```javascript
const AUTH_ERRORS = {
    INVALID_CREDENTIALS: {
        code: 'INVALID_CREDENTIALS',
        message: 'Invalid email or password',
        httpStatus: 401
    },
    ACCOUNT_LOCKED: {
        code: 'ACCOUNT_LOCKED',
        message: 'Account is temporarily locked due to too many failed attempts',
        httpStatus: 423
    },
    EMAIL_NOT_VERIFIED: {
        code: 'EMAIL_NOT_VERIFIED',
        message: 'Please verify your email address before logging in',
        httpStatus: 401
    },
    TOKEN_EXPIRED: {
        code: 'TOKEN_EXPIRED',
        message: 'Token has expired',
        httpStatus: 401
    },
    INVALID_TOKEN: {
        code: 'INVALID_TOKEN',
        message: 'Invalid or malformed token',
        httpStatus: 401
    },
    INSUFFICIENT_PERMISSIONS: {
        code: 'INSUFFICIENT_PERMISSIONS',
        message: 'You do not have permission to perform this action',
        httpStatus: 403
    }
};
```

### 6.2 Error Response Format

```javascript
function formatErrorResponse(error, req) {
    const response = {
        success: false,
        error: {
            code: error.code,
            message: error.message,
            timestamp: new Date().toISOString(),
            requestId: req.id
        }
    };

    // Add additional context in development
    if (process.env.NODE_ENV === 'development') {
        response.error.stack = error.stack;
        response.error.details = error.details;
    }

    return response;
}
```

## 7. Performance Optimization

### 7.1 Database Indexing Strategy

```sql
-- User lookup optimization
CREATE INDEX idx_users_email_verified ON users(email, email_verified);
CREATE INDEX idx_users_last_login ON users(last_login);

-- Role and permission queries
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);

-- Audit log performance
CREATE INDEX idx_audit_logs_user_timestamp ON audit_logs(user_id, timestamp);
```

### 7.2 Caching Strategy

```javascript
const redis = require('redis');
const client = redis.createClient(process.env.REDIS_URL);

// Cache user permissions for 15 minutes
async function cacheUserPermissions(userId, permissions) {
    await client.setex(`user:${userId}:permissions`, 900, JSON.stringify(permissions));
}

async function getCachedUserPermissions(userId) {
    const cached = await client.get(`user:${userId}:permissions`);
    return cached ? JSON.parse(cached) : null;
}

// Cache role definitions for 1 hour
async function cacheRoleDefinitions() {
    const roles = await db.query(`
        SELECT r.name, array_agg(p.name) as permissions
        FROM roles r
        JOIN role_permissions rp ON r.id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        GROUP BY r.id, r.name
    `);
    
    await client.setex('roles:definitions', 3600, JSON.stringify(roles.rows));
}
```

## 8. Monitoring and Observability

### 8.1 Metrics Collection

```javascript
const prometheus = require('prom-client');

const authMetrics = {
    loginAttempts: new prometheus.Counter({
        name: 'auth_login_attempts_total',
        help: 'Total number of login attempts',
        labelNames: ['status', 'method']
    }),
    
    tokenValidations: new prometheus.Counter({
        name: 'auth_token_validations_total',
        help: 'Total number of token validations',
        labelNames: ['status', 'type']
    }),
    
    responseTime: new prometheus.Histogram({
        name: 'auth_response_time_seconds',
        help: 'Response time for authentication operations',
        labelNames: ['operation']
    })
};

function recordLoginAttempt(success, method = 'password') {
    authMetrics.loginAttempts.inc({ 
        status: success ? 'success' : 'failure',
        method 
    });
}
```

### 8.2 Audit Logging

```javascript
async function logAuthEvent(userId, action, details = {}) {
    const auditLog = {
        user_id: userId,
        action,
        resource: 'auth',
        ip_address: details.ipAddress,
        user_agent: details.userAgent,
        details: JSON.stringify(details),
        timestamp: new Date()
    };

    await db.query(`
        INSERT INTO audit_logs (user_id, action, resource, ip_address, user_agent, details)
        VALUES ($1, $2, $3, $4, $5, $6)
    `, [auditLog.user_id, auditLog.action, auditLog.resource, 
        auditLog.ip_address, auditLog.user_agent, auditLog.details]);
}
```

## 9. Testing Strategy

### 9.1 Unit Test Structure

```javascript
// tests/unit/auth.service.test.js
describe('AuthService', () => {
    describe('hashPassword', () => {
        it('should hash password with bcrypt', async () => {
            const password = 'TestPassword123!';
            const hash = await authService.hashPassword(password);
            expect(hash).toBeDefined();
            expect(hash).not.toBe(password);
        });
    });

    describe('verifyPassword', () => {
        it('should verify correct password', async () => {
            const password = 'TestPassword123!';
            const hash = await authService.hashPassword(password);
            const isValid = await authService.verifyPassword(password, hash);
            expect(isValid).toBe(true);
        });
    });

    describe('generateTokenPair', () => {
        it('should generate valid JWT tokens', async () => {
            const user = { id: 'uuid', email: 'test@example.com' };
            const tokens = await authService.generateTokenPair(user);
            expect(tokens.accessToken).toBeDefined();
            expect(tokens.refreshToken).toBeDefined();
        });
    });
});
```

### 9.2 Integration Test Structure

```javascript
// tests/integration/auth.api.test.js
describe('Authentication API', () => {
    beforeEach(async () => {
        await db.query('DELETE FROM users');
        await db.query('DELETE FROM audit_logs');
    });

    describe('POST /auth/register', () => {
        it('should register new user successfully', async () => {
            const userData = {
                email: 'test@example.com',
                password: 'TestPassword123!',
                firstName: 'Test',
                lastName: 'User'
            };

            const response = await request(app)
                .post('/auth/register')
                .send(userData)
                .expect(201);

            expect(response.body.success).toBe(true);
            expect(response.body.data.user.email).toBe(userData.email);
        });
    });

    describe('POST /auth/login', () => {
        it('should login with valid credentials', async () => {
            // Create test user
            await createTestUser({
                email: 'test@example.com',
                password: 'TestPassword123!'
            });

            const response = await request(app)
                .post('/auth/login')
                .send({
                    email: 'test@example.com',
                    password: 'TestPassword123!'
                })
                .expect(200);

            expect(response.body.success).toBe(true);
            expect(response.body.data.tokens.accessToken).toBeDefined();
        });
    });
});
```

## 10. Deployment Configuration

### 10.1 Environment Variables

```env
# JWT Configuration
JWT_PRIVATE_KEY=-----BEGIN RSA PRIVATE KEY-----\n...
JWT_PUBLIC_KEY=-----BEGIN PUBLIC KEY-----\n...
JWT_EXPIRATION=900
JWT_REFRESH_EXPIRATION=604800
JWT_AUDIENCE=api.example.com
JWT_ISSUER=auth.example.com

# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=auth_db
DB_USER=auth_user
DB_PASSWORD=secure_password
DB_SSL=true

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=redis_password

# Email Service
EMAIL_SERVICE_URL=http://email-service:3000
EMAIL_SERVICE_API_KEY=email_service_key

# Security Configuration
BCRYPT_ROUNDS=12
MAX_LOGIN_ATTEMPTS=5
LOCKOUT_TIME=1800
RATE_LIMIT_WINDOW=900000
RATE_LIMIT_MAX=5

# Application Configuration
NODE_ENV=production
PORT=3000
LOG_LEVEL=info
```

### 10.2 Docker Configuration

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .

RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001

USER nodejs

EXPOSE 3000

CMD ["node", "src/index.js"]
```

This technical design specification provides a comprehensive foundation for implementing a secure, scalable, and maintainable user authentication system.