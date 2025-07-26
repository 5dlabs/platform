# Task 3: Implement Hello Endpoint

## Overview
This task implements the core functionality of the Hello World API by creating the root endpoint that returns a JSON response with a greeting message.

## Objectives
- Create GET endpoint at root path (`/`)
- Return JSON response with "Hello, World!" message
- Ensure 200 OK status code
- Follow RESTful API conventions

## Technical Approach

### 1. RESTful Design
The endpoint follows REST principles:
- Uses GET method for data retrieval
- Returns JSON content type
- Includes appropriate HTTP status code
- Stateless operation

### 2. Express Route Handler
Express.js route handlers accept request and response objects:
- `req`: Contains request data (headers, params, body)
- `res`: Used to send response back to client

### 3. JSON Response Format
The API returns structured JSON data:
```json
{
  "message": "Hello, World!"
}
```

## Implementation Details

### Route Handler Implementation
```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Key Components:
1. **HTTP Method**: `app.get()` handles GET requests
2. **Route Path**: `/` matches root URL
3. **Status Code**: `200` indicates success
4. **Response Type**: `json()` sets Content-Type header
5. **Response Body**: Object with message property

### Integration Points
- Must be placed after middleware setup
- Must be placed before 404 handler
- Integrates with request logging middleware

## Dependencies
- Task 2 must be completed (Express server created)
- Express application instance must exist
- Server must be running on port 3000

## Testing Strategy

### Unit Tests

#### Test 1: Successful Response
```bash
curl -i http://localhost:3000/
```
**Expected:**
```
HTTP/1.1 200 OK
Content-Type: application/json

{"message":"Hello, World!"}
```

#### Test 2: Request Method Validation
```bash
curl -X POST http://localhost:3000/
```
**Expected:** 404 Not Found (POST not supported)

#### Test 3: Response Headers
```bash
curl -I http://localhost:3000/
```
**Expected Headers:**
- Content-Type: application/json
- Status: 200 OK

### Integration Tests
1. **With Logging Middleware**
   - Verify request is logged
   - Confirm log shows: `GET /`

2. **Multiple Requests**
   - Send 10 rapid requests
   - All should return same response
   - No errors or timeouts

3. **Browser Compatibility**
   - Test in Chrome, Firefox, Safari
   - Verify JSON is properly displayed

## Success Criteria
- ✅ Endpoint responds to GET requests at `/`
- ✅ Returns 200 status code
- ✅ Response is valid JSON
- ✅ JSON contains `message` field
- ✅ Message value is "Hello, World!"
- ✅ Content-Type header is application/json

## Error Scenarios
1. **Server Not Running**: Connection refused
2. **Wrong HTTP Method**: Returns 404
3. **Malformed URL**: Handled by 404 middleware

## Performance Benchmarks
- Response time: < 10ms
- Throughput: > 1000 requests/second
- Memory usage: Stable under load

## Security Considerations
- No user input processed (safe from injection)
- No sensitive data exposed
- Follows least privilege principle
- No authentication required (public endpoint)

## Code Quality Standards
```javascript
// Good: Clear and concise
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Avoid: Unnecessary complexity
app.get('/', (req, res) => {
  const data = {};
  data.message = 'Hello, World!';
  res.status(200);
  res.json(data);
});
```

## API Documentation
```yaml
endpoint: GET /
description: Returns a greeting message
responses:
  200:
    description: Successful response
    content:
      application/json:
        schema:
          type: object
          properties:
            message:
              type: string
              example: "Hello, World!"
```

## Next Steps
After implementing this endpoint:
- Add health check endpoint (Task 4)
- Implement error handling
- Add API documentation
- Consider rate limiting for production