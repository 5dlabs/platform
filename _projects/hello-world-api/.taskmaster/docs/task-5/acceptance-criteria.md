# Acceptance Criteria: Add Error Handling and Documentation

## Overview
This document defines the acceptance criteria for Task 5: Add Error Handling and Documentation. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Error Handling Middleware
- [ ] Error handling middleware is implemented in src/index.js
- [ ] Middleware function has exactly 4 parameters: (err, req, res, next)
- [ ] First parameter is named `err` or similar error indicator
- [ ] Middleware logs error stack to console using `console.error()`
- [ ] Response status is set to 500
- [ ] Response is JSON format: `{ "error": "Something went wrong!" }`
- [ ] No sensitive error details exposed to client

### 2. Middleware Ordering
- [ ] Error handler is placed AFTER all route definitions
- [ ] Error handler is placed BEFORE the 404 handler
- [ ] 404 handler remains the last middleware
- [ ] Middleware chain is: routes → error handler → 404 handler

### 3. README.md File
- [ ] README.md exists in project root directory
- [ ] File uses proper Markdown formatting
- [ ] All required sections are present
- [ ] No placeholder text remains

### 4. README Content - Required Sections
- [ ] Title: "# Hello World API"
- [ ] Description paragraph explaining the API
- [ ] Installation section with npm install command
- [ ] Usage section with npm start command
- [ ] Endpoints section documenting both routes
- [ ] Example Requests section with curl examples
- [ ] Error Handling section explaining error responses
- [ ] Development section with project structure
- [ ] Requirements listing Node.js version
- [ ] License section

### 5. Endpoint Documentation
- [ ] GET / endpoint is documented
- [ ] GET /health endpoint is documented
- [ ] Each endpoint shows example response
- [ ] Response format is shown as JSON code blocks
- [ ] HTTP methods are clearly specified

### 6. Example Commands
- [ ] curl examples are provided for both endpoints
- [ ] HTTPie examples are included as alternatives
- [ ] Examples use correct URLs (http://localhost:3000)
- [ ] Examples are properly formatted in code blocks

## Test Cases

### Test Case 1: Error Handler Function
```javascript
// Verify error handler has 4 parameters
// In src/index.js, the error handler should match:
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});
```

### Test Case 2: Middleware Order Verification
```bash
# Check src/index.js for correct order
grep -n "app.get\|app.use" src/index.js
```
**Expected Order:**
1. Logging middleware
2. GET / route
3. GET /health route
4. Error handling middleware (4 params)
5. 404 handler (2-3 params)

### Test Case 3: Error Handler Testing
Create a temporary test by adding this route before the error handler:
```javascript
app.get('/test-error', (req, res) => {
  throw new Error('Test error');
});
```

Test it:
```bash
curl http://localhost:3000/test-error
```
**Expected Response:**
```json
{"error":"Something went wrong!"}
```
**Expected Status:** 500

### Test Case 4: 404 Handler Still Works
```bash
curl http://localhost:3000/nonexistent
```
**Expected Response:**
```json
{"error":"Not found"}
```
**Expected Status:** 404

### Test Case 5: README Exists
```bash
test -f README.md && echo "✓ README exists" || echo "✗ README missing"
```

### Test Case 6: README Content Validation
```bash
# Check for required sections
grep -q "# Hello World API" README.md && echo "✓ Title found"
grep -q "## Installation" README.md && echo "✓ Installation section found"
grep -q "## Usage" README.md && echo "✓ Usage section found"
grep -q "## Endpoints" README.md && echo "✓ Endpoints section found"
grep -q "## Example Requests" README.md && echo "✓ Examples section found"
grep -q "## Error Handling" README.md && echo "✓ Error section found"
```

### Test Case 7: Example Commands Work
```bash
# Extract and test curl commands from README
grep "curl http://localhost:3000" README.md | while read -r line; do
  cmd=$(echo "$line" | grep -o "curl [^\"]*")
  echo "Testing: $cmd"
  eval "$cmd"
done
```

### Test Case 8: Project Structure Accuracy
```bash
# Verify documented structure matches reality
find . -type f -name "*.js" -o -name "*.json" -o -name "*.md" | grep -E "(src/|package|README)" | sort
```

### Test Case 9: Error Logging
```bash
# Cause an error and check server console
# Should see error stack trace in server logs, not in response
```

## Validation Script

Save as `validate-final-task.js`:

```javascript
const http = require('http');
const fs = require('fs');
const assert = require('assert');

// Test 1: Check README exists
console.log('\n1. Checking README.md...');
try {
  const readme = fs.readFileSync('README.md', 'utf8');
  console.log('✓ README.md exists');
  
  // Check required sections
  const sections = [
    '# Hello World API',
    '## Installation',
    '## Usage',
    '## Endpoints',
    '## Example Requests',
    '## Error Handling',
    '## Development'
  ];
  
  sections.forEach(section => {
    if (readme.includes(section)) {
      console.log(`✓ Found section: ${section}`);
    } else {
      console.log(`✗ Missing section: ${section}`);
    }
  });
} catch (error) {
  console.log('✗ README.md not found');
}

// Test 2: Check error handler in source
console.log('\n2. Checking error handler implementation...');
try {
  const source = fs.readFileSync('src/index.js', 'utf8');
  
  // Look for error handler pattern
  const errorHandlerRegex = /app\.use\s*\(\s*\(\s*err\s*,\s*req\s*,\s*res\s*,\s*next\s*\)/;
  if (errorHandlerRegex.test(source)) {
    console.log('✓ Error handler found with 4 parameters');
  } else {
    console.log('✗ Error handler not found or incorrect parameters');
  }
  
  // Check for 500 status
  if (source.includes('status(500)')) {
    console.log('✓ Sets 500 status code');
  } else {
    console.log('✗ Missing 500 status code');
  }
} catch (error) {
  console.log('✗ Could not read src/index.js');
}

// Test 3: Test endpoints
console.log('\n3. Testing API endpoints...');

function testEndpoint(path, expectedField) {
  return new Promise((resolve) => {
    http.get(`http://localhost:3000${path}`, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const response = JSON.parse(data);
          if (response[expectedField]) {
            console.log(`✓ ${path} works correctly`);
          } else {
            console.log(`✗ ${path} missing ${expectedField}`);
          }
        } catch (e) {
          console.log(`✗ ${path} returned invalid JSON`);
        }
        resolve();
      });
    }).on('error', () => {
      console.log(`✗ Could not connect to ${path}`);
      resolve();
    });
  });
}

// Run endpoint tests
(async () => {
  await testEndpoint('/', 'message');
  await testEndpoint('/health', 'status');
  await testEndpoint('/nonexistent', 'error');
  
  console.log('\n✅ Validation complete!');
})();
```

## Code Quality Criteria

### Error Handler Quality
- [ ] Uses descriptive parameter names
- [ ] Logs full error stack for debugging
- [ ] Returns generic message to client
- [ ] Properly formatted with consistent indentation

### README Quality
- [ ] Clear, concise writing
- [ ] Proper Markdown syntax
- [ ] Code blocks use appropriate language tags
- [ ] Examples are accurate and tested
- [ ] No spelling or grammar errors

### Overall Code Structure
- [ ] Middleware order is logical and correct
- [ ] Comments explain purpose where needed
- [ ] Consistent coding style throughout
- [ ] No commented-out code or debug statements

## Definition of Done

The task is complete when:

1. All acceptance criteria checkboxes are marked
2. All test cases pass
3. Validation script runs successfully
4. Manual testing confirms error handling works
5. README can be followed by a new developer
6. No regression in existing functionality
7. Code review approved

## Security Validation

### Error Handler Security
- [ ] No stack traces sent to client
- [ ] No internal paths exposed
- [ ] No sensitive configuration leaked
- [ ] Generic error message used

### Documentation Security
- [ ] No secrets or tokens in examples
- [ ] No internal URLs or IPs exposed
- [ ] Examples use localhost only

## Performance Impact

- [ ] Error handler adds minimal overhead
- [ ] 500 errors return quickly (< 50ms)
- [ ] No memory leaks in error scenarios
- [ ] Server remains stable after errors

## Final Manual Test Checklist

1. [ ] Start server with `npm start`
2. [ ] Test hello endpoint works
3. [ ] Test health endpoint works
4. [ ] Test 404 returns proper error
5. [ ] README displays correctly in GitHub/GitLab
6. [ ] All curl examples from README work
7. [ ] Server handles errors gracefully
8. [ ] Console shows error logs (not client)