# Acceptance Criteria: Develop User Management Endpoints with Mock Data

## Core Requirements

### 1. Dependencies
- [ ] `validator@13.x.x` installed in package.json
- [ ] Package-lock.json updated

### 2. File Structure
- [ ] `/src/data/` directory exists
- [ ] `/src/data/users.js` file created
- [ ] `/src/controllers/users.js` file created
- [ ] `/src/routes/users.js` file created

### 3. Mock Data Implementation
- [ ] Initial users array with at least 2 sample users
- [ ] Each user has: id, name, email, createdAt
- [ ] Auto-incrementing ID counter
- [ ] Data persists during server runtime
- [ ] Functions exported: getUsers(), createUser()

### 4. GET /api/users Endpoint
- [ ] Returns 200 OK status
- [ ] Returns array of all users
- [ ] Response is valid JSON
- [ ] Empty array returned if no users
- [ ] Content-Type: application/json

### 5. POST /api/users Endpoint
- [ ] Returns 201 Created for valid data
- [ ] Returns 400 Bad Request for invalid data
- [ ] Creates new user with:
  - [ ] Auto-generated unique ID
  - [ ] Provided name and email
  - [ ] Current timestamp as createdAt
- [ ] New user added to mock data store
- [ ] Returns created user object

### 6. Input Validation
- [ ] Name field:
  - [ ] Required (not null/undefined)
  - [ ] Not empty string
  - [ ] Trimmed of whitespace
- [ ] Email field:
  - [ ] Required (not null/undefined)
  - [ ] Valid email format (using validator)
  - [ ] Lowercase normalized
- [ ] Error messages are descriptive

## Test Cases

### Test 1: List All Users
```bash
curl http://localhost:3000/api/users

# Expected: Array with initial users
# [
#   { "id": 1, "name": "John Doe", "email": "john@example.com", "createdAt": "..." },
#   { "id": 2, "name": "Jane Smith", "email": "jane@example.com", "createdAt": "..." }
# ]
```

### Test 2: Create Valid User
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Bob Wilson", "email": "bob@example.com"}'

# Expected: 201 Created
# { "id": 3, "name": "Bob Wilson", "email": "bob@example.com", "createdAt": "2025-..." }
```

### Test 3: Missing Required Fields
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Only Name"}'

# Expected: 400 Bad Request
# { "error": "Bad Request", "message": "Name and email are required" }
```

### Test 4: Invalid Email Format
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "not-an-email"}'

# Expected: 400 Bad Request
# { "error": "Bad Request", "message": "Invalid email format" }
```

### Test 5: Empty Values
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "", "email": ""}'

# Expected: 400 Bad Request
```

### Test 6: ID Auto-increment
```bash
# Create multiple users and verify IDs increment: 1, 2, 3, 4, etc.
# IDs should never duplicate or skip numbers
```

### Test 7: Data Persistence
```bash
# 1. Create a new user
# 2. List all users - new user should appear
# 3. Create another user
# 4. List again - both new users present
```

## Edge Cases
- [ ] Handles null/undefined body
- [ ] Handles non-JSON content type
- [ ] Handles extra fields in request (ignored)
- [ ] Handles very long strings appropriately
- [ ] Unicode characters in names handled correctly

## Performance Requirements
- [ ] GET /api/users responds in < 50ms
- [ ] POST /api/users responds in < 100ms
- [ ] No memory leaks with repeated operations
- [ ] Can handle 1000+ users in memory

## Definition of Done
- All test cases pass
- Validation works correctly
- Error messages are helpful
- Code follows established patterns
- Ready for error handling middleware integration