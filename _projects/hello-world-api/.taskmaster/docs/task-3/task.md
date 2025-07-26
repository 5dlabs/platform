# Task 3: Implement Hello Endpoint

## Overview

This task implements the core functionality of the Hello World API by creating the root endpoint that returns a JSON-formatted greeting message. This endpoint represents the primary feature of the API and demonstrates proper RESTful response formatting.

## Purpose and Objectives

The primary objective is to implement a GET endpoint at the root path (/) that:

- Returns a JSON response with a "Hello, World!" message
- Uses proper HTTP status code (200 OK)
- Follows RESTful API conventions
- Replaces the placeholder route from Task 2

## Technical Approach

### 1. RESTful Design
- Implement GET method for resource retrieval
- Return JSON as the response format
- Use appropriate HTTP status codes

### 2. Express.js Route Handler
- Define route using Express's `app.get()` method
- Utilize Express's `res.json()` for proper JSON formatting
- Set explicit status code for clarity

### 3. Response Structure
- Follow consistent JSON structure
- Use clear, descriptive field names
- Ensure proper content-type headers are set

## Implementation Details

### Route Implementation

Replace the placeholder route in `src/index.js` with:

```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Complete Context

The route should be placed in the proper position within the server file:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Hello endpoint - REPLACES the placeholder route
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Server setup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

### Key Implementation Details

1. **Route Method**: Uses `app.get()` to handle GET requests only
2. **Path**: Root path `/` serves as the main endpoint
3. **Status Code**: Explicitly sets 200 OK status
4. **Response Format**: JSON object with "message" field
5. **Content-Type**: Automatically set to `application/json` by Express

## Dependencies and Requirements

### Prerequisites
- Completed Task 1 (Project initialization)
- Completed Task 2 (Express server setup)
- Express.js server running successfully

### Technical Requirements
- Express.js ^4.18.2
- Node.js 20+

### API Requirements
- Endpoint: GET /
- Response Status: 200 OK
- Response Format: JSON
- Response Structure: `{ "message": "Hello, World!" }`

## Testing Strategy

### Manual Testing

1. **Basic Endpoint Test**
   ```bash
   curl http://localhost:3000/
   ```
   Expected response:
   ```json
   {"message":"Hello, World!"}
   ```

2. **Verbose Test with Headers**
   ```bash
   curl -v http://localhost:3000/
   ```
   Expected headers:
   ```
   < HTTP/1.1 200 OK
   < Content-Type: application/json; charset=utf-8
   ```

3. **Browser Test**
   - Navigate to http://localhost:3000/
   - Should display JSON response in browser

4. **HTTP Method Test**
   ```bash
   # Test that only GET is supported
   curl -X POST http://localhost:3000/
   curl -X PUT http://localhost:3000/
   curl -X DELETE http://localhost:3000/
   ```
   Expected: 404 errors for non-GET methods

### Automated Testing

Create `test-hello-endpoint.js`:

```javascript
const http = require('http');

const testEndpoint = () => {
  http.get('http://localhost:3000/', (res) => {
    let data = '';
    
    res.on('data', chunk => {
      data += chunk;
    });
    
    res.on('end', () => {
      const response = JSON.parse(data);
      
      console.log('Status Code Test:', res.statusCode === 200 ? '✓ PASS' : '✗ FAIL');
      console.log('Content-Type Test:', res.headers['content-type'].includes('application/json') ? '✓ PASS' : '✗ FAIL');
      console.log('Message Test:', response.message === 'Hello, World!' ? '✓ PASS' : '✗ FAIL');
      console.log('Response:', response);
    });
  });
};

testEndpoint();
```

### Using Testing Tools

**With Postman:**
1. Create new GET request
2. URL: http://localhost:3000/
3. Send request
4. Verify: Status 200, JSON response with message

**With HTTPie:**
```bash
http GET localhost:3000
```

**With Newman (Postman CLI):**
```json
{
  "info": { "name": "Hello World API Test" },
  "item": [{
    "name": "Test Hello Endpoint",
    "request": {
      "method": "GET",
      "url": "http://localhost:3000/"
    },
    "response": [{
      "status": "OK",
      "code": 200,
      "body": "{\"message\":\"Hello, World!\"}"
    }]
  }]
}
```

## Success Criteria

The endpoint is successfully implemented when:

1. GET request to `/` returns status code 200
2. Response body is valid JSON
3. JSON contains field "message" with value "Hello, World!"
4. Content-Type header is set to "application/json"
5. Other HTTP methods (POST, PUT, DELETE) return 404
6. Request logging continues to work for this endpoint
7. Response time is under 50ms

## Common Issues and Solutions

### Issue 1: Wrong Status Code
**Problem**: Endpoint returns 304 or other status
**Solution**: Explicitly set `res.status(200)` before sending response

### Issue 2: Plain Text Response
**Problem**: Response is plain text instead of JSON
**Solution**: Use `res.json()` instead of `res.send()`

### Issue 3: Malformed JSON
**Problem**: JSON parsing fails on client
**Solution**: Ensure using `res.json({ message: 'Hello, World!' })` syntax

### Issue 4: Route Not Found
**Problem**: Still getting "Server is running" message
**Solution**: Ensure old placeholder route is completely replaced

## Best Practices Implemented

1. **Explicit Status Codes**: Always set status explicitly for clarity
2. **Consistent JSON Structure**: Use object with descriptive field names
3. **RESTful Conventions**: GET for retrieval, proper status codes
4. **Error Prevention**: Let Express handle JSON serialization
5. **Maintainability**: Clear comments and route organization

## Performance Considerations

- Minimal processing required (static response)
- No database queries or external API calls
- Response should be nearly instantaneous
- Suitable for high-frequency health checks

## Security Considerations

- No user input processing (no injection risks)
- No sensitive data in response
- Standard Express security headers applied
- No authentication required (public endpoint)

## Next Steps

After completing this task:
- Task 4: Implement Health Check Endpoint - Add monitoring capabilities
- Task 5: Add Error Handling and Documentation - Complete the API
- Consider adding request/response validation
- Consider implementing API versioning