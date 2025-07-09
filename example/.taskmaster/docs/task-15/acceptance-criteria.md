# Acceptance Criteria: Add Error Handling Middleware

## Test Cases and Validation

### 1. Error Middleware Integration

#### Test Case 1.1: Global Error Handler Installation
**Given**: Error middleware is implemented
**When**: Server is started
**Then**: Error handler is the last middleware in the stack

**Verification Commands**:
```bash
# Check server starts without errors
npm run dev
```

**Expected**: Server starts successfully with error middleware loaded

#### Test Case 1.2: Error Handler Catches Errors
**Given**: Error occurs in route handler
**When**: Error is thrown or passed to next()
**Then**: Error handler processes and responds appropriately

**Test Code**:
```bash
# Test route that throws error
curl -X GET http://localhost:3000/api/nonexistent-route
```

**Expected**: Consistent error response format with proper status code

### 2. Error Response Format

#### Test Case 2.1: Standard Error Response Structure
**Given**: Any error occurs
**When**: Error handler processes the error
**Then**: Response follows standardized format

**Test Code**:
```bash
curl -s http://localhost:3000/api/nonexistent-route | jq '.'
```

**Expected Response Structure**:
```json
{
  "error": "string",
  "code": "string",
  "timestamp": "ISO-8601 string",
  "path": "string",
  "method": "string",
  "requestId": "string"
}
```

#### Test Case 2.2: Required Fields Present
**Given**: Error response is generated
**When**: Examining response fields
**Then**: All required fields are present

**Test Code**:
```bash
curl -s http://localhost:3000/api/nonexistent-route | jq 'has("error") and has("code") and has("timestamp") and has("path") and has("method") and has("requestId")'
```

**Expected**: `true` for all required fields

#### Test Case 2.3: Content-Type Header
**Given**: Error response is sent
**When**: Checking response headers
**Then**: Content-Type is application/json

**Test Code**:
```bash
curl -I http://localhost:3000/api/nonexistent-route | grep -i "content-type: application/json"
```

**Expected**: Content-Type header is set correctly

### 3. HTTP Status Code Validation

#### Test Case 3.1: 400 Bad Request
**Given**: Validation error occurs
**When**: Making request with invalid data
**Then**: Returns 400 status code

**Test Code**:
```bash
curl -w "%{http_code}" -o /dev/null -s -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"","email":"invalid"}'
```

**Expected**: HTTP 400 status code

#### Test Case 3.2: 404 Not Found
**Given**: Resource does not exist
**When**: Making request to non-existent resource
**Then**: Returns 404 status code

**Test Code**:
```bash
curl -w "%{http_code}" -o /dev/null -s -X GET http://localhost:3000/api/users/nonexistent-id
```

**Expected**: HTTP 404 status code

#### Test Case 3.3: 409 Conflict
**Given**: Duplicate resource creation is attempted
**When**: Making request with conflicting data
**Then**: Returns 409 status code

**Test Code**:
```bash
# Create first user
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","email":"conflict@example.com"}'

# Try to create duplicate
curl -w "%{http_code}" -o /dev/null -s -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test2","email":"conflict@example.com"}'
```

**Expected**: HTTP 409 status code

#### Test Case 3.4: 500 Internal Server Error
**Given**: Unexpected server error occurs
**When**: Server encounters internal error
**Then**: Returns 500 status code

**Manual Test**: Simulate server error and verify 500 response

### 4. Error Code Consistency

#### Test Case 4.1: Validation Error Code
**Given**: Validation error occurs
**When**: Checking error response
**Then**: Error code is VALIDATION_ERROR

**Test Code**:
```bash
curl -s -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"","email":"invalid"}' | jq '.code'
```

**Expected**: `"VALIDATION_ERROR"`

#### Test Case 4.2: Not Found Error Code
**Given**: Resource not found
**When**: Checking error response
**Then**: Error code is NOT_FOUND

**Test Code**:
```bash
curl -s -X GET http://localhost:3000/api/users/nonexistent-id | jq '.code'
```

**Expected**: `"NOT_FOUND"`

#### Test Case 4.3: Conflict Error Code
**Given**: Conflict error occurs
**When**: Checking error response
**Then**: Error code is CONFLICT

**Test Code**:
```bash
# Create user first, then try duplicate
curl -s -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test2","email":"conflict@example.com"}' | jq '.code'
```

**Expected**: `"CONFLICT"`

### 5. Request ID Tracking

#### Test Case 5.1: Request ID Generation
**Given**: Request is made to any endpoint
**When**: Checking response or headers
**Then**: Request ID is present

**Test Code**:
```bash
curl -s http://localhost:3000/api/health | jq '.requestId'
# or for error responses:
curl -s http://localhost:3000/api/nonexistent-route | jq '.requestId'
```

**Expected**: Non-null request ID string

#### Test Case 5.2: Request ID Uniqueness
**Given**: Multiple requests are made
**When**: Checking request IDs
**Then**: Each request has unique ID

**Test Code**:
```bash
# Make multiple requests and collect request IDs
for i in {1..5}; do
  curl -s http://localhost:3000/api/health | jq -r '.requestId'
done | sort | uniq -c
```

**Expected**: All request IDs are unique (count = 1 for each)

### 6. Error Details Handling

#### Test Case 6.1: Validation Error Details
**Given**: Validation error with details
**When**: Checking error response
**Then**: Details are included in response

**Test Code**:
```bash
curl -s -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"","email":"invalid"}' | jq '.details'
```

**Expected**: Details object with validation information

#### Test Case 6.2: Production vs Development Details
**Given**: Different environments
**When**: Error occurs
**Then**: Details are filtered based on environment

**Test Code**:
```bash
# In development - should have more details
NODE_ENV=development npm run dev

# In production - should have limited details
NODE_ENV=production npm run start
```

**Expected**: Different detail levels based on environment

### 7. Error Logging

#### Test Case 7.1: Error Logging Occurs
**Given**: Error occurs in application
**When**: Error handler processes error
**Then**: Error is logged to console/logs

**Manual Test**: Monitor console output when errors occur

#### Test Case 7.2: Log Entry Format
**Given**: Error is logged
**When**: Examining log output
**Then**: Log contains required information

**Expected Log Fields**:
- Timestamp
- Request ID
- Method and path
- Status code
- Error code
- Error message

#### Test Case 7.3: Log Levels
**Given**: Different types of errors
**When**: Errors are logged
**Then**: Appropriate log levels are used

**Expected**:
- 5xx errors: ERROR level
- 4xx errors: WARN level
- Normal operations: INFO level

### 8. Security Validation

#### Test Case 8.1: No Stack Traces in Production
**Given**: Production environment
**When**: Error occurs
**Then**: Stack traces are not included in response

**Test Code**:
```bash
NODE_ENV=production npm run start
curl -s http://localhost:3000/api/nonexistent-route | jq 'has("stack")'
```

**Expected**: `false` (no stack trace in production)

#### Test Case 8.2: Sensitive Information Filtering
**Given**: Error contains sensitive data
**When**: Error response is generated
**Then**: Sensitive information is filtered out

**Manual Test**: Verify no database connection strings, API keys, or passwords in error responses

### 9. Custom Error Classes

#### Test Case 9.1: ValidationError Class
**Given**: ValidationError is thrown
**When**: Error handler processes it
**Then**: Correct status code and error code are returned

**Test Code**:
```typescript
// In route handler
throw new ValidationError('Test validation error', { field: 'name' });
```

**Expected**: 400 status code with VALIDATION_ERROR code

#### Test Case 9.2: NotFoundError Class
**Given**: NotFoundError is thrown
**When**: Error handler processes it
**Then**: Correct status code and error code are returned

**Test Code**:
```typescript
// In route handler
throw new NotFoundError('Resource not found');
```

**Expected**: 404 status code with NOT_FOUND code

#### Test Case 9.3: ConflictError Class
**Given**: ConflictError is thrown
**When**: Error handler processes it
**Then**: Correct status code and error code are returned

**Test Code**:
```typescript
// In route handler
throw new ConflictError('Resource conflict');
```

**Expected**: 409 status code with CONFLICT code

### 10. Async Error Handling

#### Test Case 10.1: Async Handler Wrapper
**Given**: Async route handler throws error
**When**: Error occurs in async function
**Then**: Error is properly caught and handled

**Test Code**:
```typescript
// Async route handler
router.get('/async-error', asyncHandler(async (req, res) => {
  throw new Error('Async error test');
}));
```

**Expected**: Error is caught and processed by error handler

#### Test Case 10.2: Promise Rejection Handling
**Given**: Promise rejection occurs
**When**: Unhandled promise rejection happens
**Then**: Global handler processes it

**Manual Test**: Verify unhandled promise rejections are caught

### 11. Rate Limiting Integration

#### Test Case 11.1: Rate Limit Error Format
**Given**: Rate limit is exceeded
**When**: Too many requests are made
**Then**: Error response follows standard format

**Test Code**:
```bash
# Make many requests quickly
for i in {1..150}; do
  curl -s http://localhost:3000/api/health > /dev/null
done

# Check error response format
curl -s http://localhost:3000/api/health | jq '.'
```

**Expected**: Standard error format with rate limit information

### 12. 404 Handler for Unmatched Routes

#### Test Case 12.1: Unmatched Route Handling
**Given**: Request is made to non-existent route
**When**: No route matches the request
**Then**: 404 handler provides consistent response

**Test Code**:
```bash
curl -s http://localhost:3000/completely/non/existent/route | jq '.'
```

**Expected**: 404 response with standard error format

### 13. Error Middleware Order

#### Test Case 13.1: Error Handler is Last
**Given**: Multiple middleware are configured
**When**: Error occurs
**Then**: Error handler processes it (must be last in stack)

**Manual Test**: Verify error handler is registered after all routes

### 14. Performance Testing

#### Test Case 14.1: Error Processing Performance
**Given**: High volume of errors
**When**: Multiple errors occur simultaneously
**Then**: Error processing doesn't significantly impact performance

**Test Code**:
```bash
# Generate multiple errors quickly
for i in {1..100}; do
  curl -s http://localhost:3000/api/nonexistent-route > /dev/null &
done
wait
```

**Expected**: All errors processed within reasonable time

### 15. TypeScript Integration

#### Test Case 15.1: Type Safety
**Given**: Error handling code is implemented
**When**: Running TypeScript compiler
**Then**: No type errors are reported

**Test Code**:
```bash
npx tsc --noEmit
```

**Expected**: No TypeScript compilation errors

#### Test Case 15.2: Custom Error Types
**Given**: Custom error classes are defined
**When**: Using error classes
**Then**: Type checking works correctly

**Test Code**:
```typescript
// This should compile without errors
const error = new ValidationError('Test', { field: 'name' });
expect(error.statusCode).toBe(400);
```

## Acceptance Checklist

### Core Requirements
- [ ] Global error handler is implemented and registered
- [ ] Error responses follow standardized format
- [ ] Correct HTTP status codes for different error types
- [ ] Request ID tracking is implemented
- [ ] Error logging is functional
- [ ] Custom error classes work correctly

### Security Requirements
- [ ] No sensitive information in error responses
- [ ] Stack traces filtered in production
- [ ] Appropriate error message sanitization
- [ ] Rate limiting integration

### Performance Requirements
- [ ] Error processing doesn't block application
- [ ] Memory usage remains stable during errors
- [ ] Error handling performance is acceptable
- [ ] No memory leaks in error handlers

### Integration Requirements
- [ ] Works with all existing routes
- [ ] Middleware order is correct
- [ ] TypeScript compilation passes
- [ ] Async error handling works
- [ ] 404 handler for unmatched routes

### Monitoring Requirements
- [ ] Error logging includes required fields
- [ ] Log levels are appropriate
- [ ] Request correlation works
- [ ] Error metrics can be collected

## Performance Benchmarks

- **Error Processing Time**: < 5ms per error
- **Memory Usage**: < 1MB additional memory for error system
- **Throughput**: Handle 1000 errors/second without degradation
- **Response Time**: Error responses < 10ms

## Rollback Plan

If any acceptance criteria fail:
1. Review error middleware implementation
2. Check middleware registration order
3. Verify TypeScript error class definitions
4. Test error response format
5. Check logging configuration
6. Verify security filtering
7. Test async error handling
8. Review integration with existing routes