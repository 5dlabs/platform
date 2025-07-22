# Acceptance Criteria: Add Input Validation Middleware

## Core Requirements

### 1. File Structure
- [ ] `/src/middleware/validateUser.js` exists
- [ ] `/src/utils/validators.js` exists
- [ ] Files use ES6 module syntax
- [ ] Proper imports and exports

### 2. Validation Middleware Implementation
- [ ] `validateCreateUser` function exists
- [ ] Middleware signature: (req, res, next)
- [ ] Validates both name and email
- [ ] Collects all errors before responding
- [ ] Throws ValidationError with all errors
- [ ] Sanitizes valid input data

### 3. Validation Rules - Name
- [ ] Required field check
- [ ] Type check (must be string)
- [ ] Empty string check (after trim)
- [ ] Minimum length: 2 characters
- [ ] Maximum length: 100 characters
- [ ] Whitespace trimmed from value

### 4. Validation Rules - Email
- [ ] Required field check
- [ ] Type check (must be string)
- [ ] Valid email format (using validator)
- [ ] Converted to lowercase
- [ ] Whitespace trimmed from value

### 5. Validation Utilities
- [ ] Generic `validateString` helper function
- [ ] Configurable min/max length
- [ ] Required/optional field support
- [ ] `validateEmail` helper function
- [ ] Reusable across different endpoints

### 6. Integration
- [ ] Middleware added to POST /api/users route
- [ ] Runs before controller function
- [ ] Controller simplified (no validation code)
- [ ] Error handling maintained
- [ ] Middleware order correct

## Test Cases

### Test 1: Valid Input
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "  John Doe  ", "email": "  JOHN@EXAMPLE.COM  "}'

# Expected: 201 Created
# Name should be "John Doe" (trimmed)
# Email should be "john@example.com" (lowercase, trimmed)
```

### Test 2: Missing Fields
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{}'

# Expected: 400 Bad Request
# {
#   "error": "ValidationError",
#   "message": "Name is required, Email is required"
# }
```

### Test 3: Invalid Types
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": 123, "email": true}'

# Expected: 400 Bad Request
# Message includes type errors
```

### Test 4: Name Too Short
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "A", "email": "test@example.com"}'

# Expected: 400 Bad Request
# Message: "Name must be at least 2 characters long"
```

### Test 5: Invalid Email
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "not-an-email"}'

# Expected: 400 Bad Request
# Message: "Email must be a valid email address"
```

### Test 6: Multiple Errors
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "", "email": "invalid"}'

# Expected: 400 Bad Request
# Message includes both name and email errors
```

### Test 7: Whitespace Only
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "   ", "email": "test@example.com"}'

# Expected: 400 Bad Request
# Message: "Name cannot be empty"
```

## Edge Cases
- [ ] Handles null values
- [ ] Handles undefined values
- [ ] Handles arrays instead of strings
- [ ] Handles objects instead of strings
- [ ] Very long strings (> 1000 chars)
- [ ] Special characters in names
- [ ] International email formats

## Performance Requirements
- [ ] Validation adds < 5ms latency
- [ ] No regex compilation on each request
- [ ] Efficient string operations
- [ ] No memory leaks

## Code Quality
- [ ] DRY principle followed
- [ ] Validation logic centralized
- [ ] Easy to add new validations
- [ ] Clear error messages
- [ ] Well-documented functions

## Definition of Done
- All validation rules enforced
- Multiple errors reported together
- Data properly sanitized
- Controllers simplified
- Middleware integrated correctly
- Ready for additional validations