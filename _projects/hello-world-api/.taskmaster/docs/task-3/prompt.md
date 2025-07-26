# Autonomous Agent Prompt: Implement Hello Endpoint

You are an autonomous agent tasked with implementing the main Hello World endpoint for the API. Your goal is to create a properly formatted JSON endpoint that returns a greeting message.

## Prerequisites

Verify before starting:
- Express.js server is running (from Task 2)
- Server has logging middleware in place
- Placeholder route exists at root path (/)

## Task Requirements

### 1. Locate the Existing Route

Find the placeholder route in `src/index.js`:
```javascript
// Basic route placeholder
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### 2. Replace with Hello Endpoint

Replace the entire placeholder route with:
```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### 3. Implementation Checklist

Ensure your implementation:
- [ ] Uses `app.get()` for GET method only
- [ ] Responds to root path `/`
- [ ] Sets status code to 200
- [ ] Returns JSON format using `res.json()`
- [ ] JSON structure: `{ "message": "Hello, World!" }`
- [ ] Includes the exact punctuation in "Hello, World!"

### 4. Verify Route Order

Confirm the route order in your file:
1. Express setup and middleware
2. Request logging middleware
3. **Hello endpoint (your new route)**
4. 404 handler (must remain last)

## Validation Steps

### Step 1: Test Basic Functionality
```bash
curl http://localhost:3000/
```
**Expected Output:**
```json
{"message":"Hello, World!"}
```

### Step 2: Verify JSON Format
```bash
curl -H "Accept: application/json" http://localhost:3000/
```
Confirm response is valid JSON with proper formatting.

### Step 3: Check Status Code
```bash
curl -I http://localhost:3000/
```
Verify: `HTTP/1.1 200 OK`

### Step 4: Test Content-Type
```bash
curl -v http://localhost:3000/ 2>&1 | grep -i content-type
```
Verify: `Content-Type: application/json`

### Step 5: Ensure Logging Works
Make a request and check server console for:
```
2024-XX-XX...Z - GET /
```

### Step 6: Test Other Methods Return 404
```bash
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/
```
Should return 404 errors.

## Common Mistakes to Avoid

1. **Don't use `res.send()`** - Use `res.json()` for JSON responses
2. **Don't forget the status code** - Explicitly set `.status(200)`
3. **Don't misspell the message** - Must be exactly "Hello, World!" with comma and exclamation
4. **Don't create multiple routes** - Replace the placeholder, don't add another
5. **Don't change the path** - Keep it at root `/`

## Expected File Structure

After implementation, the route section should look like:
```javascript
// ... middleware above ...

// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// ... server.listen below ...
```

## Success Indicators

You know the task is complete when:
- GET request to `/` returns JSON with "Hello, World!" message
- Status code is 200 OK
- Content-Type header shows application/json
- No errors in server console
- Logging still shows all requests
- Other endpoints still return 404

## Final Verification

Run this quick test:
```bash
# Should output: {"message":"Hello, World!"}
curl -s http://localhost:3000/ | grep -q "Hello, World!" && echo "✓ Success" || echo "✗ Failed"
```

If you see "✓ Success", the endpoint is correctly implemented!