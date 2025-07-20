# Task 5: Add Request Validation and Error Handling - Acceptance Criteria

## Overview
This document defines acceptance criteria for implementing comprehensive input validation, rate limiting, and standardized error handling across the Express application.

## Dependencies Criteria

### ✓ Required Packages Installed
- **Requirement**: Validation and rate limiting packages installed
- **Verification**:
  ```bash
  npm list express-validator express-rate-limit
  ```
- **Expected Versions**:
  - express-validator: ^7.0.1
  - express-rate-limit: ^7.2.0

## Validation Implementation Criteria

### ✓ Validation Middleware Created
- **Requirement**: Comprehensive validation rules defined
- **Verification**:
  ```bash
  test -f src/middleware/validation.js && echo "Validation middleware exists"
  ```
- **Expected**: File contains validation rules for all endpoints

### ✓ Email Validation
- **Requirement**: Email format validated and normalized
- **Test Cases**:
  ```bash
  # Invalid format
  curl -X POST http://localhost:3000/auth/register \
    -H "Content-Type: application/json" \
    -d '{"email":"invalid","password":"password123"}'
  # Expected: 400 "Invalid email format"
  
  # Too long
  curl -X POST http://localhost:3000/auth/register \
    -H "Content-Type: application/json" \
    -d '{"email":"very-long-email-address-that-exceeds-255-characters@example.com","password":"password123"}'
  # Expected: 400 "Email must be less than 255 characters"
  ```

### ✓ Password Validation
- **Requirement**: Password length and optional complexity rules
- **Test Cases**:
  ```bash
  # Too short
  -d '{"email":"test@example.com","password":"short"}'
  # Expected: 400 "Password must be at least 8 characters"
  
  # Too long (128+ chars)
  -d '{"email":"test@example.com","password":"<129 chars>"}'
  # Expected: 400 "Password must be less than 128 characters"
  ```

### ✓ Task Title Validation
- **Requirement**: Required, 1-255 characters, trimmed
- **Test Cases**:
  ```bash
  # Empty title
  -d '{"title":"","description":"Test"}'
  # Expected: 400 "Title is required"
  
  # Only whitespace
  -d '{"title":"   ","description":"Test"}'
  # Expected: 400 "Title is required"
  
  # Too long (256+ chars)
  -d '{"title":"<256 chars>","description":"Test"}'
  # Expected: 400 "Title must be between 1 and 255 characters"
  ```

### ✓ Task Description Validation
- **Requirement**: Optional, max 1000 characters
- **Test Cases**:
  ```bash
  # Null description (should work)
  -d '{"title":"Test"}'
  # Expected: 201 Success
  
  # Too long (1001+ chars)
  -d '{"title":"Test","description":"<1001 chars>"}'
  # Expected: 400 "Description must be less than 1000 characters"
  ```

### ✓ ID Parameter Validation
- **Requirement**: Must be positive integer
- **Test Cases**:
  ```bash
  # Non-numeric
  curl http://localhost:3000/api/tasks/abc \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Invalid ID format"
  
  # Negative number
  curl http://localhost:3000/api/tasks/-1 \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Invalid ID format"
  
  # Zero
  curl http://localhost:3000/api/tasks/0 \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Invalid ID format"
  ```

### ✓ Query Parameter Validation
- **Requirement**: Pagination and filter parameters validated
- **Test Cases**:
  ```bash
  # Invalid completed value
  curl "http://localhost:3000/api/tasks?completed=maybe" \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Completed must be true or false"
  
  # Limit too high
  curl "http://localhost:3000/api/tasks?limit=200" \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Limit must be between 1 and 100"
  
  # Negative offset
  curl "http://localhost:3000/api/tasks?offset=-10" \
    -H "Authorization: Bearer $TOKEN"
  # Expected: 400 "Offset must be 0 or greater"
  ```

### ✓ Input Sanitization
- **Requirement**: Inputs are trimmed and normalized
- **Test**: Send "  test@EXAMPLE.com  " as email
- **Expected**: Stored as "test@example.com"

### ✓ Validation Error Format
- **Requirement**: Consistent error response structure
- **Expected Format**:
  ```json
  {
    "error": {
      "message": "Invalid email format",
      "field": "email",
      "code": "VALIDATION_ERROR",
      "value": "invalid-email"
    }
  }
  ```

## Rate Limiting Criteria

### ✓ Rate Limiters Configured
- **Requirement**: Different limits for different endpoint types
- **Verification**:
  ```bash
  test -f src/middleware/rateLimiter.js && echo "Rate limiter exists"
  ```

### ✓ Auth Endpoint Rate Limiting
- **Requirement**: 5 requests per hour
- **Test**:
  ```bash
  # Make 6 login attempts quickly
  for i in {1..6}; do
    curl -X POST http://localhost:3000/auth/login \
      -H "Content-Type: application/json" \
      -d '{"email":"test@example.com","password":"wrong"}' &
  done
  wait
  ```
- **Expected**: 6th request returns 429 "Too many authentication attempts"

### ✓ General API Rate Limiting
- **Requirement**: 100 requests per 15 minutes
- **Test**: Make 101 requests to any protected endpoint
- **Expected**: 101st request returns 429

### ✓ Creation Endpoint Rate Limiting
- **Requirement**: 30 creations per 15 minutes
- **Test**: Create 31 tasks quickly
- **Expected**: 31st request returns 429

### ✓ Rate Limit Headers
- **Requirement**: Standard headers included
- **Verification**:
  ```bash
  curl -I http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected Headers**:
  - RateLimit-Limit
  - RateLimit-Remaining
  - RateLimit-Reset

### ✓ Rate Limit by User
- **Requirement**: Authenticated users limited by user ID
- **Test**: Make requests with different user tokens
- **Expected**: Each user has separate rate limit

### ✓ Skip in Test Environment
- **Requirement**: Rate limiting disabled when NODE_ENV=test
- **Test**: Set NODE_ENV=test and make many requests
- **Expected**: No rate limiting applied

## Error Handling Criteria

### ✓ Custom Error Classes
- **Requirement**: Error classes for different scenarios
- **Verification**:
  ```bash
  test -f src/utils/errors.js && echo "Error classes exist"
  ```
- **Expected Classes**:
  - AppError (base class)
  - ValidationError (400)
  - AuthenticationError (401)
  - AuthorizationError (403)
  - NotFoundError (404)
  - ConflictError (409)
  - RateLimitError (429)

### ✓ Async Error Handling
- **Requirement**: All async routes wrapped with asyncHandler
- **Test**: Cause database error in async route
- **Expected**: Error caught and handled properly

### ✓ Consistent Error Response Format
- **Requirement**: All errors follow same structure
- **Expected Format**:
  ```json
  {
    "error": {
      "message": "Human-readable message",
      "code": "ERROR_CODE",
      "field": "field_name" // optional
    }
  }
  ```

### ✓ Error Logging
- **Requirement**: All errors logged with context
- **Verification**: Check console output on error
- **Expected Log Fields**:
  - message
  - stack
  - statusCode
  - code
  - path
  - method
  - ip
  - userId (if authenticated)

### ✓ Production Error Handling
- **Requirement**: No sensitive info in production
- **Test**: Set NODE_ENV=production and trigger error
- **Expected**: Generic message, no stack trace

### ✓ Development Error Details
- **Requirement**: Full details in development
- **Test**: Set NODE_ENV=development and trigger error
- **Expected**: Stack trace included in response

## Integration Criteria

### ✓ Auth Routes Updated
- **Requirement**: All auth endpoints use validation
- **Test Each**:
  - POST /auth/register - with invalid data
  - POST /auth/login - with invalid data
  - POST /auth/refresh - without token
  - GET /auth/me - works with error handling

### ✓ Task Routes Updated
- **Requirement**: All task endpoints use validation
- **Test Each**:
  - GET /api/tasks - with invalid query params
  - POST /api/tasks - with invalid body
  - GET /api/tasks/:id - with invalid ID
  - PUT /api/tasks/:id - with invalid data
  - DELETE /api/tasks/:id - with invalid ID

### ✓ Error Handler Registered
- **Requirement**: Global error handler is last middleware
- **Verification**: Check app.js middleware order
- **Expected**: errorHandler after all routes

### ✓ 404 Handler
- **Requirement**: Catches undefined routes
- **Test**:
  ```bash
  curl http://localhost:3000/undefined-route
  ```
- **Expected**: 404 "Not Found" with standard error format

## Security Criteria

### ✓ SQL Injection Prevention
- **Requirement**: Validation prevents SQL injection
- **Test**: Try SQL in parameters
  ```bash
  curl "http://localhost:3000/api/tasks?limit=1;DROP TABLE tasks;" \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 400 validation error

### ✓ XSS Prevention
- **Requirement**: Input sanitized
- **Test**: Send HTML/script in title
  ```bash
  -d '{"title":"<script>alert(1)</script>","description":"XSS test"}'
  ```
- **Expected**: HTML escaped or stripped

### ✓ No Information Leakage
- **Requirement**: Errors don't reveal system details
- **Test**: Various invalid requests
- **Expected**: Generic messages, no paths or table names

## Performance Criteria

### ✓ Validation Performance
- **Requirement**: Validation adds minimal overhead
- **Test**: Time requests with/without validation
- **Expected**: < 10ms difference

### ✓ Error Handling Performance
- **Requirement**: Errors handled efficiently
- **Test**: Trigger various errors
- **Expected**: Quick response, no hanging

## Test Summary Checklist

- [ ] express-validator and express-rate-limit installed
- [ ] Validation middleware file created
- [ ] Email validation works (format, length)
- [ ] Password validation works (length)
- [ ] Task validation works (title, description)
- [ ] ID parameter validation works
- [ ] Query parameter validation works
- [ ] Input sanitization works
- [ ] Validation error format consistent
- [ ] Rate limiters configured
- [ ] Auth endpoints rate limited (5/hour)
- [ ] General endpoints rate limited (100/15min)
- [ ] Rate limit headers present
- [ ] Rate limiting by user ID
- [ ] Custom error classes created
- [ ] Async errors handled properly
- [ ] Error response format consistent
- [ ] Errors logged with context
- [ ] Production hides sensitive info
- [ ] Development shows full errors
- [ ] All auth routes use validation
- [ ] All task routes use validation
- [ ] Global error handler registered
- [ ] 404 handler works
- [ ] SQL injection prevented
- [ ] XSS prevented
- [ ] No information leakage

## Definition of Done

Task 5 is complete when:
1. All endpoints validate input before processing
2. Rate limiting prevents abuse
3. Errors handled consistently across application
4. Security vulnerabilities addressed
5. All acceptance criteria met
6. No sensitive information exposed
7. Performance impact minimal
8. Integration seamless with existing code

## Notes

- Test both valid and invalid inputs
- Verify rate limits reset properly
- Check error logs for proper formatting
- Test in both development and production modes
- Ensure validation doesn't break existing functionality