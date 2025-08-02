# Task 3: User Authentication System - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement a secure authentication system. The tools focus on file creation for auth components and research for security best practices.

## Core Tools

### 1. brave_web_search
**Purpose**: Research authentication best practices and implementation patterns
**When to use**: 
- Before implementing security features
- When choosing email service providers
- To find JWT implementation patterns
- For rate limiting strategies

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "JWT refresh token best practices Node.js 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "JWT access refresh token pattern Node.js"
- "bcrypt vs argon2 password hashing 2024"
- "Express rate limiting Redis production"
- "Nodemailer SendGrid setup TypeScript"
- "JWT security vulnerabilities prevention"

### 2. create_directory
**Purpose**: Organize authentication-related code
**When to use**:
- Setting up auth folder structure
- Creating middleware directory
- Organizing validators and services

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/controllers/auth"
}
```

**Directory structure**:
```
/backend/src/
├── controllers/
│   └── authController.ts
├── middleware/
│   ├── auth.ts
│   └── rateLimiter.ts
├── services/
│   ├── tokenService.ts
│   └── emailService.ts
├── validators/
│   └── authValidators.ts
└── utils/
    └── passwordUtils.ts
```

### 3. write_file
**Purpose**: Create all authentication system files
**When to use**:
- Writing controller classes
- Creating middleware functions
- Implementing services
- Setting up validators

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/controllers/authController.ts",
  "content": "import { Request, Response } from 'express';\n..."
}
```

### 4. edit_file
**Purpose**: Update existing files with auth integration
**When to use**:
- Adding auth routes to main router
- Updating app.ts with middleware
- Adding auth types to type definitions
- Modifying package.json for new dependencies

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/backend/src/app.ts",
  "old_string": "// Routes",
  "new_string": "// Routes\napp.use('/api/auth', authRoutes);"
}
```

### 5. read_file
**Purpose**: Review existing code before modifications
**When to use**:
- Before updating route configurations
- To check existing type definitions
- To understand current middleware setup
- Before modifying environment configs

## Implementation Flow

### Phase 1: Research Security Patterns (10 minutes)
1. **JWT implementation**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "JWT access refresh token security best practices 2024"
   }
   ```

2. **Password hashing**:
   ```json
   {
     "tool": "brave_web_search", 
     "query": "bcrypt rounds recommendation production 2024"
   }
   ```

3. **Rate limiting**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Express rate limit Redis distributed applications"
   }
   ```

### Phase 2: Create Auth Structure (15 minutes)
1. **Create directories**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/validators"
   }
   ```

2. **Write auth controller**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/controllers/authController.ts",
     "content": "// Complete AuthController implementation"
   }
   ```

3. **Create token service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/services/tokenService.ts",
     "content": "// JWT token management service"
   }
   ```

### Phase 3: Implement Middleware (20 minutes)
1. **Authentication middleware**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/middleware/auth.ts",
     "content": "// JWT verification middleware"
   }
   ```

2. **Rate limiting setup**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/middleware/rateLimiter.ts",
     "content": "// Redis-based rate limiting"
   }
   ```

3. **Validation middleware**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/validators/authValidators.ts",
     "content": "// Input validation rules"
   }
   ```

### Phase 4: Integration (15 minutes)
1. **Update main router**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/backend/src/routes/index.ts"
   }
   ```
   Then:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/src/routes/index.ts",
     "old_string": "export const routes = Router();",
     "new_string": "export const routes = Router();\n\nroutes.use('/auth', authRoutes);"
   }
   ```

2. **Add dependencies**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/package.json",
     "old_string": "\"dependencies\": {",
     "new_string": "\"dependencies\": {\n    \"jsonwebtoken\": \"^9.0.0\",\n    \"bcrypt\": \"^5.1.0\","
   }
   ```

## Best Practices

### Security Implementation Order
1. Research current best practices first
2. Implement password hashing before storage
3. Set up rate limiting early
4. Add validation to all endpoints
5. Test security measures thoroughly

### File Organization
```typescript
// Group related auth files together
/auth/
  authController.ts    // All auth endpoints
  authService.ts      // Business logic
  authValidators.ts   // Input validation
  authTypes.ts        // TypeScript interfaces
```

### Code Patterns
```typescript
// Always hash passwords before storage
const hashedPassword = await bcrypt.hash(password, 10);

// Never store tokens in code
const secret = process.env.JWT_SECRET;

// Always validate input
const errors = validationResult(req);
if (!errors.isEmpty()) {
  return res.status(400).json({ errors: errors.array() });
}
```

## Common Patterns

### Research → Implement → Test
```javascript
// 1. Research best practice
const research = await brave_web_search("JWT storage best practices");

// 2. Implement based on findings
await write_file("tokenService.ts", implementationCode);

// 3. Create tests
await write_file("authController.test.ts", testCode);
```

### Incremental Integration
```javascript
// 1. Create component
await write_file("auth/authController.ts", controllerCode);

// 2. Read existing router
const router = await read_file("routes/index.ts");

// 3. Update router
await edit_file("routes/index.ts", oldRouter, newRouter);
```

## Troubleshooting

### Issue: JWT token not verifying
**Solution**: Check JWT_SECRET in environment, ensure same secret for sign/verify

### Issue: Bcrypt hanging on hash
**Solution**: Reduce rounds in development (10), ensure async/await usage

### Issue: Rate limiting not working
**Solution**: Verify Redis connection, check key prefix configuration

### Issue: Email not sending
**Solution**: Check SMTP credentials, verify firewall rules

## Security Checklist
- [ ] All passwords hashed with bcrypt
- [ ] JWT secrets in environment variables
- [ ] Rate limiting on all auth endpoints
- [ ] Input validation on every field
- [ ] No sensitive data in responses
- [ ] Proper error messages (no user enumeration)
- [ ] Token expiry times configured
- [ ] Refresh tokens in Redis only

## Task Completion Verification
1. Test all auth endpoints manually
2. Verify token generation and validation
3. Check rate limiting effectiveness
4. Confirm emails are sent
5. Test with invalid inputs
6. Verify security headers
7. Check Redis for stored tokens
8. Review all error messages

This implementation creates a production-ready authentication system following current security best practices.