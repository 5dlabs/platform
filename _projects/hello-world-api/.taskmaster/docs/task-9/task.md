# Task 9: Implement Root Endpoint

## Overview
**Title**: Implement Root Endpoint  
**Status**: pending  
**Priority**: medium  
**Dependencies**: Task 8 (Create Main Server File)  

## Description
Create the root endpoint that returns a 'Hello, World!' message. This task implements the primary API endpoint that responds to GET requests at the root path (/) with a JSON message, fulfilling the core requirement of the Hello World API.

## Technical Approach

### 1. Route Handler Implementation
- Define GET route for root path (/)
- Use Express route handler syntax
- Position correctly in middleware chain

### 2. Response Format
- Return JSON response with message field
- Set appropriate HTTP status code (200)
- Use Express res.json() for proper headers

### 3. Integration
- Add route after middleware but before server.listen()
- Ensure no route conflicts
- Maintain clean code structure

## Implementation Details

### Complete Route Implementation
```javascript
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Integration in index.js
```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Error handling for server startup
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// ... error handling code ...
```

### Response Specification

#### HTTP Response
- **Status Code**: 200 (OK)
- **Content-Type**: application/json
- **Body**: `{"message": "Hello, World!"}`

#### Express Methods Used
- `app.get()`: Define GET route handler
- `res.status()`: Set HTTP status code
- `res.json()`: Send JSON response with proper headers

## Subtasks Breakdown

### 1. Create basic route handler for root endpoint
- **Status**: pending
- **Dependencies**: None
- **Implementation**: Add GET / route handler
- **Location**: After middleware, before server.listen()

### 2. Add proper HTTP status code
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Use res.status(200)
- **Purpose**: Explicit success status

### 3. Format JSON response structure
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Use res.json() with message object
- **Format**: `{ message: 'Hello, World!' }`

### 4. Add comments and documentation
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Add descriptive comment above handler
- **Purpose**: Code clarity and maintenance

### 5. Verify endpoint integration with Express app
- **Status**: pending
- **Dependencies**: All subtasks
- **Validation**: Test complete integration
- **Checks**: Route accessibility, response format

## Dependencies
- Express.js server running (from Task 8)
- Express routing functionality
- JSON response capability

## Testing Strategy

### Manual Testing

#### 1. Browser Test
- Navigate to http://localhost:3000/
- Should display JSON: `{"message":"Hello, World!"}`

#### 2. cURL Test
```bash
curl http://localhost:3000/
# Expected output:
# {"message":"Hello, World!"}
```

#### 3. Verbose cURL Test
```bash
curl -v http://localhost:3000/
# Should show:
# < HTTP/1.1 200 OK
# < Content-Type: application/json; charset=utf-8
# {"message":"Hello, World!"}
```

#### 4. HTTP Method Tests
```bash
# Test other methods (should fail)
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/
curl -X DELETE http://localhost:3000/
# Expected: 404 or method not allowed
```

### Expected Server Logs
```
2024-01-15T10:30:45.123Z - GET /
```

## Common Issues and Solutions

### Issue: Route returns 404
**Solution**: Ensure route is defined before app.listen() and after app initialization

### Issue: Returns HTML instead of JSON
**Solution**: Use res.json() instead of res.send()

### Issue: No Content-Type header
**Solution**: res.json() automatically sets application/json header

### Issue: Route conflicts with static files
**Solution**: Define API routes before static middleware

## API Documentation

### GET /
**Description**: Returns a welcome message  
**Parameters**: None  
**Headers**: None required  
**Success Response**:
- **Code**: 200
- **Content**: `{ "message": "Hello, World!" }`

**Example Request**:
```bash
GET / HTTP/1.1
Host: localhost:3000
```

**Example Response**:
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Content-Length: 27

{"message":"Hello, World!"}
```

## Performance Considerations

- Minimal processing overhead
- No database queries or external calls
- Response time should be < 10ms
- Suitable for health checks and monitoring

## Security Considerations

- No sensitive data exposed
- No user input processed
- Safe from injection attacks
- Appropriate for public access

## Next Steps
After completing this task:
- Implement health check endpoint (Task 10)
- Add error handling middleware (Task 11)
- Create comprehensive documentation (Task 12)
- Test all endpoints together

The API now has its primary functionality implemented and can respond to basic requests.