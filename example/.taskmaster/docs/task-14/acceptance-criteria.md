# Acceptance Criteria: Implement User Routes

## Test Cases and Validation

### 1. GET /api/users Endpoint

#### Test Case 1.1: Retrieve All Users
**Given**: Server is running with user routes
**When**: Making GET request to `/api/users`
**Then**: Returns all users in the system

**Verification Commands**:
```bash
curl -X GET http://localhost:3000/api/users
```

**Expected Response**:
- Status: 200 OK
- Content-Type: application/json
- Body: Array of UserResponse objects

#### Test Case 1.2: Empty Users List
**Given**: No users in the system
**When**: Making GET request to `/api/users`
**Then**: Returns empty array

**Test Code**:
```bash
# Reset users array (restart server)
curl -X GET http://localhost:3000/api/users | jq '. | length'
```

**Expected**: Response is valid JSON array (empty or populated)

#### Test Case 1.3: Response Format Validation
**Given**: Users exist in the system
**When**: Making GET request to `/api/users`
**Then**: Each user has correct response format

**Test Code**:
```bash
curl -s http://localhost:3000/api/users | jq '.[0] | has("id") and has("name") and has("email") and has("createdAt")'
```

**Expected**: `true` for each required field

### 2. GET /api/users/:id Endpoint

#### Test Case 2.1: Retrieve Specific User
**Given**: User with specific ID exists
**When**: Making GET request to `/api/users/:id`
**Then**: Returns the specific user

**Test Code**:
```bash
# Get first user ID
USER_ID=$(curl -s http://localhost:3000/api/users | jq -r '.[0].id')

# Get specific user
curl -X GET http://localhost:3000/api/users/$USER_ID
```

**Expected**: Single UserResponse object with matching ID

#### Test Case 2.2: User Not Found
**Given**: User with ID does not exist
**When**: Making GET request to `/api/users/:id`
**Then**: Returns 404 error

**Test Code**:
```bash
curl -w "%{http_code}" -o /dev/null -s http://localhost:3000/api/users/nonexistent-id
```

**Expected**: HTTP 404 status code

### 3. POST /api/users Endpoint

#### Test Case 3.1: Create New User
**Given**: Valid user data
**When**: Making POST request to `/api/users`
**Then**: Creates and returns new user

**Test Code**:
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Johnson","email":"alice@example.com"}'
```

**Expected**:
- Status: 201 Created
- Response includes generated ID and createdAt timestamp
- User is added to users list

#### Test Case 3.2: Validation - Missing Name
**Given**: Request body missing name field
**When**: Making POST request to `/api/users`
**Then**: Returns 400 validation error

**Test Code**:
```bash
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com"}'
```

**Expected**: HTTP 400 status code with validation error

#### Test Case 3.3: Validation - Missing Email
**Given**: Request body missing email field
**When**: Making POST request to `/api/users`
**Then**: Returns 400 validation error

**Test Code**:
```bash
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test User"}'
```

**Expected**: HTTP 400 status code with validation error

#### Test Case 3.4: Validation - Invalid Email Format
**Given**: Request body with invalid email format
**When**: Making POST request to `/api/users`
**Then**: Returns 400 validation error

**Test Code**:
```bash
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test User","email":"invalid-email"}'
```

**Expected**: HTTP 400 status code with validation error

#### Test Case 3.5: Duplicate Email Prevention
**Given**: User with email already exists
**When**: Making POST request with same email
**Then**: Returns 409 conflict error

**Test Code**:
```bash
# Create first user
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"User One","email":"duplicate@example.com"}'

# Try to create second user with same email
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"User Two","email":"duplicate@example.com"}'
```

**Expected**: Second request returns HTTP 409 status code

### 4. PUT /api/users/:id Endpoint

#### Test Case 4.1: Update User Name
**Given**: User exists in system
**When**: Making PUT request to update name
**Then**: Updates and returns modified user

**Test Code**:
```bash
# Get user ID
USER_ID=$(curl -s http://localhost:3000/api/users | jq -r '.[0].id')

# Update user name
curl -X PUT http://localhost:3000/api/users/$USER_ID \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name"}'
```

**Expected**: User name is updated, other fields unchanged

#### Test Case 4.2: Update User Email
**Given**: User exists in system
**When**: Making PUT request to update email
**Then**: Updates and returns modified user

**Test Code**:
```bash
# Get user ID
USER_ID=$(curl -s http://localhost:3000/api/users | jq -r '.[0].id')

# Update user email
curl -X PUT http://localhost:3000/api/users/$USER_ID \
  -H "Content-Type: application/json" \
  -d '{"email":"updated@example.com"}'
```

**Expected**: User email is updated, other fields unchanged

#### Test Case 4.3: Update Non-existent User
**Given**: User ID does not exist
**When**: Making PUT request to update user
**Then**: Returns 404 error

**Test Code**:
```bash
curl -w "%{http_code}" -X PUT http://localhost:3000/api/users/nonexistent-id \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name"}'
```

**Expected**: HTTP 404 status code

### 5. DELETE /api/users/:id Endpoint

#### Test Case 5.1: Delete Existing User
**Given**: User exists in system
**When**: Making DELETE request to `/api/users/:id`
**Then**: Removes user and returns 204

**Test Code**:
```bash
# Get user ID
USER_ID=$(curl -s http://localhost:3000/api/users | jq -r '.[0].id')

# Delete user
curl -w "%{http_code}" -X DELETE http://localhost:3000/api/users/$USER_ID
```

**Expected**: HTTP 204 No Content status code

#### Test Case 5.2: Delete Non-existent User
**Given**: User ID does not exist
**When**: Making DELETE request to `/api/users/:id`
**Then**: Returns 404 error

**Test Code**:
```bash
curl -w "%{http_code}" -X DELETE http://localhost:3000/api/users/nonexistent-id
```

**Expected**: HTTP 404 status code

### 6. Data Persistence Testing

#### Test Case 6.1: Data Persists Between Requests
**Given**: User is created
**When**: Making subsequent GET request
**Then**: User data is still available

**Test Code**:
```bash
# Create user
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Persistent User","email":"persistent@example.com"}'

# Verify user exists
curl -s http://localhost:3000/api/users | jq '.[] | select(.email=="persistent@example.com")'
```

**Expected**: User data is returned correctly

#### Test Case 6.2: Data Lost on Server Restart
**Given**: Users exist in system
**When**: Server is restarted
**Then**: In-memory data is lost

**Manual Test**: Restart server and verify users are reset

### 7. Error Handling Testing

#### Test Case 7.1: Invalid JSON Request
**Given**: Malformed JSON in request body
**When**: Making POST request
**Then**: Returns 400 bad request

**Test Code**:
```bash
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","email"}'
```

**Expected**: HTTP 400 status code

#### Test Case 7.2: Missing Content-Type Header
**Given**: Request without Content-Type header
**When**: Making POST request
**Then**: Request fails appropriately

**Test Code**:
```bash
curl -w "%{http_code}" -X POST http://localhost:3000/api/users \
  -d '{"name":"Test","email":"test@example.com"}'
```

**Expected**: Request handled appropriately (400 or successful parsing)

### 8. TypeScript Integration Testing

#### Test Case 8.1: Type Safety Compilation
**Given**: User routes implementation
**When**: Running TypeScript compiler
**Then**: No type errors reported

**Verification Commands**:
```bash
npx tsc --noEmit
```

**Expected**: No compilation errors

#### Test Case 8.2: Type Guard Validation
**Given**: Type guards are implemented
**When**: Processing request data
**Then**: Type validation works correctly

**Test Code**:
```typescript
// This should be verified in the implementation
import { isValidCreateUserRequest } from '../types/user';

const validRequest = { name: 'Test', email: 'test@example.com' };
const invalidRequest = { name: 'Test' };

console.assert(isValidCreateUserRequest(validRequest) === true);
console.assert(isValidCreateUserRequest(invalidRequest) === false);
```

### 9. Performance Testing

#### Test Case 9.1: Response Time Under Load
**Given**: Multiple concurrent requests
**When**: Making simultaneous API calls
**Then**: Response times remain acceptable

**Test Code**:
```bash
# Create multiple users concurrently
for i in {1..10}; do
  curl -X POST http://localhost:3000/api/users \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"User $i\",\"email\":\"user$i@example.com\"}" &
done
wait
```

**Expected**: All requests complete successfully within 2 seconds

#### Test Case 9.2: Memory Usage Stability
**Given**: Continuous API usage
**When**: Creating and deleting users repeatedly
**Then**: Memory usage remains stable

**Manual Test**: Monitor memory usage during extended testing

### 10. Integration Testing

#### Test Case 10.1: Route Integration with Main App
**Given**: User routes are mounted in main app
**When**: Making requests to user endpoints
**Then**: Routes are accessible and functional

**Test Code**:
```bash
# Test all endpoints are accessible
curl -X GET http://localhost:3000/api/users
curl -X POST http://localhost:3000/api/users -H "Content-Type: application/json" -d '{"name":"Test","email":"test@example.com"}'
```

**Expected**: All endpoints respond correctly

#### Test Case 10.2: Middleware Integration
**Given**: Express middleware is configured
**When**: Making requests to user endpoints
**Then**: Middleware works correctly with user routes

**Test**: Verify JSON parsing, error handling, and other middleware functions

### 11. UUID Generation Testing

#### Test Case 11.1: Unique ID Generation
**Given**: Multiple users are created
**When**: Checking generated IDs
**Then**: All IDs are unique

**Test Code**:
```bash
# Create multiple users and check ID uniqueness
for i in {1..5}; do
  curl -X POST http://localhost:3000/api/users \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"User $i\",\"email\":\"unique$i@example.com\"}"
done

# Check for duplicate IDs
curl -s http://localhost:3000/api/users | jq '[.[].id] | group_by(.) | map(length) | max'
```

**Expected**: Maximum group size is 1 (no duplicates)

#### Test Case 11.2: UUID Format Validation
**Given**: Users are created
**When**: Checking generated IDs
**Then**: IDs are valid UUID v4 format

**Test Code**:
```bash
curl -s http://localhost:3000/api/users | jq '.[0].id' | grep -E '"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}"'
```

**Expected**: ID matches UUID v4 pattern

## Acceptance Checklist

### Core Functionality
- [ ] GET /api/users returns all users
- [ ] GET /api/users/:id returns specific user
- [ ] POST /api/users creates new user
- [ ] PUT /api/users/:id updates existing user
- [ ] DELETE /api/users/:id removes user
- [ ] Proper HTTP status codes for all scenarios

### Data Validation
- [ ] Name field is required and validated
- [ ] Email field is required and validated
- [ ] Email format validation works correctly
- [ ] Duplicate email prevention works
- [ ] Input sanitization (trimming, case normalization)

### Error Handling
- [ ] 400 Bad Request for invalid data
- [ ] 404 Not Found for non-existent resources
- [ ] 409 Conflict for duplicate emails
- [ ] 500 Internal Server Error for server issues
- [ ] Proper error message format

### TypeScript Integration
- [ ] All types are properly defined
- [ ] TypeScript compilation passes
- [ ] Type safety throughout implementation
- [ ] Type guards work correctly

### Performance
- [ ] Response times under 100ms for normal requests
- [ ] Handles concurrent requests correctly
- [ ] Memory usage remains stable
- [ ] No memory leaks detected

### Integration
- [ ] Routes integrate with main Express app
- [ ] Middleware compatibility verified
- [ ] UUID generation works correctly
- [ ] In-memory storage functions properly

### Security
- [ ] Input validation prevents injection
- [ ] No sensitive data in error messages
- [ ] Proper data sanitization
- [ ] Rate limiting considerations addressed

## Performance Benchmarks

- **GET /api/users**: < 50ms response time
- **POST /api/users**: < 100ms response time
- **PUT /api/users/:id**: < 75ms response time
- **DELETE /api/users/:id**: < 50ms response time
- **Concurrent requests**: 100 requests in < 2 seconds
- **Memory usage**: < 10MB additional memory for user storage

## Rollback Plan

If any acceptance criteria fail:
1. Review user routes implementation
2. Check TypeScript type definitions
3. Verify Express route integration
4. Test individual validation functions
5. Check UUID package installation
6. Verify error handling implementation
7. Test with different request formats
8. Check server startup and routing configuration