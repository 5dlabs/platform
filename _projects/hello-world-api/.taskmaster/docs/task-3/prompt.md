# Autonomous Agent Prompt: Implement Hello Endpoint

You are tasked with implementing the main Hello World endpoint for the Express.js API.

## Your Mission
Add the root endpoint to the Express.js server that returns a JSON response with "Hello, World!" message.

## Prerequisites
- Task 2 must be completed (Express server exists)
- Server file `src/index.js` must have basic structure
- Request logging middleware must be in place

## Required Actions

### 1. Locate Insertion Point
Open `src/index.js` and find the correct location to add the route:
- After middleware setup
- Before any error handlers (like 404 handler)
- After the logging middleware

### 2. Implement Route Handler
Add the following code:

```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### 3. Verify Route Order
Ensure your routes are in this order:
1. Request logging middleware
2. Hello endpoint (this task)
3. 404 error handler
4. Server listener

### 4. Complete Implementation
Your route handler must:
- Use GET method
- Match root path `/`
- Return 200 status code
- Send JSON response
- Include exact message format

## Validation Tests

### Test 1: Basic Functionality
```bash
curl http://localhost:3000/
```
**Expected Response:**
```json
{"message":"Hello, World!"}
```

### Test 2: Status Code Verification
```bash
curl -i http://localhost:3000/
```
**Expected:**
- Status: 200 OK
- Content-Type: application/json

### Test 3: Exact JSON Format
```bash
curl -s http://localhost:3000/ | python -m json.tool
```
**Expected Output:**
```json
{
    "message": "Hello, World!"
}
```

### Test 4: Method Restrictions
```bash
# POST should not work
curl -X POST http://localhost:3000/
```
**Expected:** 404 Not Found

```bash
# PUT should not work
curl -X PUT http://localhost:3000/
```
**Expected:** 404 Not Found

### Test 5: Request Logging
Make a request and check server console:
```bash
curl http://localhost:3000/
```
**Expected Console Log:** `[timestamp] - GET /`

## Common Mistakes to Avoid

1. **Wrong Response Format**
   ❌ `res.send('Hello, World!')`
   ✅ `res.json({ message: 'Hello, World!' })`

2. **Missing Status Code**
   ❌ `res.json({ message: 'Hello, World!' })`
   ✅ `res.status(200).json({ message: 'Hello, World!' })`

3. **Wrong Route Order**
   ❌ Placing after 404 handler
   ✅ Placing before 404 handler

4. **Typos in Response**
   ❌ `{ message: 'Hello World' }` (missing comma)
   ❌ `{ message: 'Hello, World' }` (missing exclamation)
   ✅ `{ message: 'Hello, World!' }`

## Success Criteria
- Endpoint responds at root path
- Returns exactly: `{"message":"Hello, World!"}`
- Status code is 200
- Content-Type is application/json
- Only responds to GET method
- Request is logged by middleware

## Troubleshooting

### Issue: 404 Not Found
- Check route is defined before 404 handler
- Verify path is exactly `/`
- Ensure server is restarted after changes

### Issue: Wrong Response Format
- Use `res.json()` not `res.send()`
- Verify object structure matches exactly

### Issue: No Logging
- Confirm logging middleware is before route
- Check console for any errors

Complete the implementation and run all validation tests before proceeding.