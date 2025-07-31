# Task 3: User Authentication System - Toolman Usage Guide

## Overview

This guide explains how to use Toolman to implement the User Authentication System efficiently and securely. Toolman provides automated assistance for file creation, dependency management, testing, and security validation throughout the implementation process.

## Prerequisites

Before starting, ensure:
- Node.js 16+ is installed
- npm or yarn is available
- Redis server is accessible
- PostgreSQL or MySQL database is configured
- SMTP credentials for email service

## Implementation Workflow

### Step 1: Initialize Project Structure

```bash
# Use Toolman to create the authentication module structure
toolman create-structure auth

# This will create:
# - src/controllers/auth.controller.ts
# - src/services/jwt.service.ts
# - src/services/user.service.ts
# - src/services/email.service.ts
# - src/middleware/auth.middleware.ts
# - src/routes/auth.routes.ts
# - src/validators/auth.validator.ts
# - tests/auth.test.ts
```

### Step 2: Install Dependencies

```bash
# Let Toolman install all required packages
toolman install-deps auth

# This installs:
# - jsonwebtoken, bcrypt, redis, ioredis, nodemailer
# - TypeScript types for all packages
# - Testing frameworks (jest, supertest)
# - Security tools (helmet, express-rate-limit)
```

### Step 3: Configure Environment

```bash
# Generate environment template
toolman generate-env auth

# This creates .env.example with:
JWT_SECRET=generate-secure-secret-here
JWT_REFRESH_SECRET=generate-different-secret-here
REDIS_URL=redis://localhost:6379
DATABASE_URL=postgresql://user:pass@localhost:5432/dbname
EMAIL_HOST=smtp.gmail.com
EMAIL_PORT=587
EMAIL_USER=your-email@gmail.com
EMAIL_PASS=your-app-password
FRONTEND_URL=http://localhost:3000
```

### Step 4: Implement Core Services

#### JWT Service Implementation

```bash
# Generate JWT service with best practices
toolman implement jwt-service

# Toolman will:
# 1. Create JWT service with token generation
# 2. Implement secure token storage in Redis
# 3. Add token validation and refresh logic
# 4. Include proper error handling
```

#### User Service Implementation

```bash
# Generate user service
toolman implement user-service

# This creates:
# - User CRUD operations
# - Password hashing utilities
# - Email validation
# - Database queries with prepared statements
```

#### Email Service Setup

```bash
# Configure email service
toolman implement email-service

# Includes:
# - SMTP configuration
# - HTML email templates
# - Password reset email logic
# - Error handling for failed sends
```

### Step 5: Create Authentication Controller

```bash
# Generate complete auth controller
toolman implement auth-controller

# This implements all endpoints:
# - /register with validation
# - /login with rate limiting
# - /refresh with token rotation
# - /profile (protected)
# - /reset-password with email
# - /logout with token revocation
```

### Step 6: Set Up Middleware

```bash
# Create authentication middleware
toolman implement auth-middleware

# Generates:
# - Token extraction from headers
# - JWT verification logic
# - User context attachment
# - Error handling for expired/invalid tokens
```

### Step 7: Configure Routes

```bash
# Set up authentication routes
toolman configure-routes auth

# This will:
# 1. Create route definitions
# 2. Apply middleware appropriately
# 3. Set up rate limiting per endpoint
# 4. Configure CORS if needed
```

## Security Testing Procedures

### 1. Vulnerability Scanning

```bash
# Run security audit
toolman security-scan auth

# Checks for:
# - Dependency vulnerabilities
# - SQL injection risks
# - XSS vulnerabilities
# - Insecure configurations
```

### 2. Password Security Testing

```bash
# Test password hashing
toolman test-security passwords

# Validates:
# - Bcrypt rounds (12+)
# - Password complexity rules
# - No plaintext storage
# - Timing attack resistance
```

### 3. Token Security Validation

```bash
# Verify JWT implementation
toolman test-security tokens

# Tests:
# - Token expiration
# - Signature validation
# - Refresh token rotation
# - Secret strength
```

### 4. Rate Limiting Tests

```bash
# Test rate limits
toolman test-limits auth

# Simulates:
# - Multiple registration attempts
# - Brute force login attempts
# - Password reset abuse
```

## Token Management Best Practices

### Access Token Handling

```typescript
// Toolman generates this pattern:
const authenticateRequest = async (req: Request) => {
  const token = req.headers.authorization?.replace('Bearer ', '');
  
  if (!token) {
    throw new UnauthorizedError('No token provided');
  }
  
  try {
    const payload = jwt.verify(token, process.env.JWT_SECRET);
    return payload;
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      throw new UnauthorizedError('Token expired');
    }
    throw new UnauthorizedError('Invalid token');
  }
};
```

### Refresh Token Rotation

```typescript
// Toolman implements automatic rotation:
const rotateRefreshToken = async (userId: string, oldToken: string) => {
  // Verify old token exists in Redis
  const storedToken = await redis.get(`refresh_token:${userId}`);
  
  if (storedToken !== oldToken) {
    // Possible token reuse attack
    await redis.del(`refresh_token:${userId}`);
    throw new SecurityError('Invalid refresh token');
  }
  
  // Generate new tokens
  const { accessToken, refreshToken } = generateTokens(userId);
  
  // Store new refresh token
  await redis.set(
    `refresh_token:${userId}`,
    refreshToken,
    'EX',
    7 * 24 * 60 * 60
  );
  
  return { accessToken, refreshToken };
};
```

## Email Service Configuration

### SMTP Setup

```bash
# Configure email service
toolman configure-email

# For Gmail:
# 1. Enable 2-factor authentication
# 2. Generate app-specific password
# 3. Use app password in EMAIL_PASS

# For SendGrid/Mailgun:
# 1. Obtain API credentials
# 2. Configure appropriate transport
```

### Email Templates

```typescript
// Toolman provides professional templates:
const passwordResetTemplate = (resetUrl: string) => `
  <!DOCTYPE html>
  <html>
    <head>
      <style>
        .button {
          background-color: #4CAF50;
          color: white;
          padding: 14px 20px;
          text-decoration: none;
          display: inline-block;
          border-radius: 4px;
        }
      </style>
    </head>
    <body>
      <h2>Password Reset Request</h2>
      <p>Click the button below to reset your password:</p>
      <a href="${resetUrl}" class="button">Reset Password</a>
      <p>This link expires in 1 hour.</p>
      <p>If you didn't request this, please ignore this email.</p>
    </body>
  </html>
`;
```

## Common Authentication Patterns

### 1. Remember Me Functionality

```bash
# Implement remember me option
toolman add-feature remember-me

# Extends refresh token expiry
# Adds persistent session option
```

### 2. Two-Factor Authentication

```bash
# Add 2FA support
toolman add-feature 2fa

# Implements:
# - TOTP generation
# - QR code generation
# - Backup codes
```

### 3. OAuth Integration

```bash
# Add social login
toolman add-feature oauth --provider google

# Sets up:
# - OAuth2 flow
# - Profile mapping
# - Account linking
```

## Testing Workflow

### 1. Unit Tests

```bash
# Run unit tests
toolman test unit auth

# Tests individual components:
# - JWT service methods
# - Password hashing
# - Email sending
# - Validation logic
```

### 2. Integration Tests

```bash
# Run integration tests
toolman test integration auth

# Tests complete flows:
# - Registration → Login → Access
# - Token refresh cycle
# - Password reset flow
```

### 3. Load Testing

```bash
# Performance testing
toolman test load auth

# Simulates:
# - 1000 concurrent logins
# - Token refresh under load
# - Database connection pooling
```

## Troubleshooting

### Common Issues

1. **JWT Secret Not Set**
```bash
toolman diagnose jwt-secret
# Checks environment variables
# Suggests secure secret generation
```

2. **Redis Connection Failed**
```bash
toolman diagnose redis
# Tests Redis connectivity
# Checks authentication
# Verifies persistence settings
```

3. **Email Not Sending**
```bash
toolman diagnose email
# Tests SMTP connection
# Validates credentials
# Checks firewall settings
```

### Debug Mode

```bash
# Enable detailed logging
toolman debug auth --verbose

# Shows:
# - Token generation process
# - Database queries
# - Redis operations
# - Email sending logs
```

## Deployment Checklist

```bash
# Run pre-deployment checks
toolman deploy-check auth

✓ Environment variables set
✓ HTTPS configured
✓ Rate limiting enabled
✓ Database indexes created
✓ Redis persistence configured
✓ Email service verified
✓ Security headers set
✓ CORS configured
✓ Monitoring enabled
✓ Backup strategy defined
```

## Monitoring and Maintenance

### Health Checks

```bash
# Set up monitoring endpoints
toolman monitor auth

# Creates:
# GET /health - Basic health check
# GET /health/detailed - Component status
# GET /metrics - Authentication metrics
```

### Log Analysis

```bash
# Analyze authentication logs
toolman analyze-logs auth --period 24h

# Reports:
# - Failed login attempts
# - Token refresh patterns
# - Password reset requests
# - Suspicious activities
```

## Best Practices Summary

1. **Always use Toolman's security templates** - Pre-configured with best practices
2. **Run security scans regularly** - Catch vulnerabilities early
3. **Test rate limiting** - Prevent abuse
4. **Monitor authentication metrics** - Detect anomalies
5. **Keep dependencies updated** - Use `toolman update-deps`
6. **Document custom modifications** - Maintain consistency
7. **Use Toolman's validation** - Catch configuration errors

## Getting Help

```bash
# Show available commands
toolman help auth

# Get detailed command info
toolman help implement jwt-service

# Check documentation
toolman docs auth-patterns

# Report issues
toolman report-issue
```

Remember: Toolman is designed to enforce security best practices automatically. Trust its defaults and only override when absolutely necessary.