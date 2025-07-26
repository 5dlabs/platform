# Acceptance Criteria: Implement Health Check Endpoint

## Overview
This document defines the acceptance criteria for Task 4: Implement Health Check Endpoint. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Endpoint Configuration
- [ ] GET route is defined for path `/health`
- [ ] Route handler uses `app.get()` method
- [ ] Route is placed after hello endpoint but before 404 handler
- [ ] Route path is exactly `/health` (lowercase)

### 2. Response Structure
- [ ] Response is in JSON format
- [ ] Response uses `res.json()` method
- [ ] JSON contains exactly two fields: `status` and `timestamp`
- [ ] No additional fields in response
- [ ] Field order doesn't affect functionality

### 3. Status Field
- [ ] Field name is exactly `status` (lowercase)
- [ ] Field value is exactly `healthy` (lowercase)
- [ ] Value is a string, not boolean or number
- [ ] Value never changes (always "healthy")

### 4. Timestamp Field
- [ ] Field name is exactly `timestamp` (lowercase)
- [ ] Value is generated using `new Date().toISOString()`
- [ ] Format follows ISO 8601 standard
- [ ] Includes full date and time with milliseconds
- [ ] Ends with 'Z' indicating UTC timezone

### 5. Dynamic Behavior
- [ ] Timestamp is generated fresh for each request
- [ ] Timestamp is NOT cached or reused
- [ ] Multiple requests show different timestamps
- [ ] Timestamp reflects actual request time

### 6. HTTP Requirements
- [ ] Response status code is 200 OK
- [ ] Content-Type header is `application/json`
- [ ] Only GET method is supported
- [ ] Other HTTP methods return 404

## Test Cases

### Test Case 1: Basic Health Check
```bash
curl http://localhost:3000/health
```
**Expected Response Format:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.845Z"
}
```
**Note:** Timestamp will vary but format must match

### Test Case 2: Status Code Verification
```bash
curl -w "\nHTTP Status: %{http_code}\n" http://localhost:3000/health
```
**Expected:** HTTP Status: 200

### Test Case 3: Response Headers
```bash
curl -I http://localhost:3000/health
```
**Must Include:**
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
```

### Test Case 4: Timestamp Format Validation
```bash
curl -s http://localhost:3000/health | jq -r '.timestamp' | grep -E '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$'
```
**Expected:** Timestamp matching ISO 8601 pattern

### Test Case 5: Dynamic Timestamp Test
```bash
# Get two timestamps
TS1=$(curl -s http://localhost:3000/health | jq -r '.timestamp')
sleep 1
TS2=$(curl -s http://localhost:3000/health | jq -r '.timestamp')

# Compare
if [ "$TS1" != "$TS2" ]; then
  echo "✓ PASS: Timestamps are different"
else
  echo "✗ FAIL: Timestamps are identical"
fi
```

### Test Case 6: Field Validation
```bash
curl -s http://localhost:3000/health | jq 'keys | sort'
```
**Expected Output:**
```json
["status", "timestamp"]
```

### Test Case 7: Status Value Check
```bash
curl -s http://localhost:3000/health | jq -r '.status'
```
**Expected Output:**
```
healthy
```

### Test Case 8: POST Method Rejection
```bash
curl -X POST http://localhost:3000/health
```
**Expected Response:**
```json
{"error":"Not found"}
```
**Expected Status:** 404

### Test Case 9: Request Logging
```bash
curl http://localhost:3000/health
```
**Expected Server Log:**
```
2024-01-15T14:32:17.845Z - GET /health
```

### Test Case 10: Concurrent Requests
```bash
# Send 5 concurrent requests
for i in {1..5}; do
  curl -s http://localhost:3000/health &
done | wait
```
**Expected:** All responses valid with different timestamps

## Validation Script

Save as `validate-health-endpoint.js`:

```javascript
const http = require('http');
const assert = require('assert');

async function validateHealthEndpoint() {
  return new Promise((resolve, reject) => {
    http.get('http://localhost:3000/health', (res) => {
      let data = '';
      
      res.on('data', chunk => data += chunk);
      
      res.on('end', () => {
        try {
          // Parse response
          const response = JSON.parse(data);
          
          // Test 1: Status code
          assert.strictEqual(res.statusCode, 200, 'Status code should be 200');
          console.log('✓ Status code: 200');
          
          // Test 2: Content type
          assert(res.headers['content-type'].includes('application/json'), 
                 'Content-Type should be application/json');
          console.log('✓ Content-Type: application/json');
          
          // Test 3: Response structure
          const keys = Object.keys(response);
          assert.strictEqual(keys.length, 2, 'Response should have exactly 2 fields');
          assert(keys.includes('status'), 'Response should have "status" field');
          assert(keys.includes('timestamp'), 'Response should have "timestamp" field');
          console.log('✓ Response structure: correct');
          
          // Test 4: Status value
          assert.strictEqual(response.status, 'healthy', 
                            'Status should be "healthy"');
          console.log('✓ Status value: healthy');
          
          // Test 5: Timestamp format
          const timestamp = response.timestamp;
          const date = new Date(timestamp);
          assert(!isNaN(date.getTime()), 'Timestamp should be valid date');
          assert(timestamp.endsWith('Z'), 'Timestamp should end with Z');
          assert(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(timestamp),
                 'Timestamp should match ISO 8601 format');
          console.log('✓ Timestamp format: ISO 8601');
          
          console.log('\n✓ All validation tests passed!');
          console.log('Response:', response);
          resolve();
        } catch (error) {
          console.error('\n✗ Validation failed:', error.message);
          reject(error);
        }
      });
    }).on('error', reject);
  });
}

// Test dynamic timestamps
async function testDynamicTimestamps() {
  const getTimestamp = () => new Promise((resolve, reject) => {
    http.get('http://localhost:3000/health', (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const response = JSON.parse(data);
          resolve(response.timestamp);
        } catch (error) {
          reject(error);
        }
      });
    }).on('error', reject);
  });
  
  const ts1 = await getTimestamp();
  await new Promise(resolve => setTimeout(resolve, 100)); // Wait 100ms
  const ts2 = await getTimestamp();
  
  assert.notStrictEqual(ts1, ts2, 'Timestamps should be different');
  console.log('✓ Dynamic timestamps: working');
}

// Run all tests
(async () => {
  try {
    console.log('Validating Health Check Endpoint...\n');
    await validateHealthEndpoint();
    await testDynamicTimestamps();
    console.log('\n✅ Health endpoint is correctly implemented!');
  } catch (error) {
    console.error('\n❌ Health endpoint validation failed');
    process.exit(1);
  }
})();
```

## Performance Criteria

- [ ] Response time < 10ms for single request
- [ ] Can handle 1000 requests/second
- [ ] No memory leaks during sustained load
- [ ] CPU usage remains minimal

## Load Test

```bash
# Simple load test with Apache Bench (if available)
ab -n 1000 -c 10 http://localhost:3000/health

# Alternative with curl
time for i in {1..100}; do curl -s http://localhost:3000/health > /dev/null & done; wait
```

## Integration Test Scenarios

### 1. Monitoring System Integration
- Configure monitoring to poll `/health` every 30 seconds
- Verify 100% success rate over 1 hour
- Check timestamp progression

### 2. Container Health Check
```yaml
# docker-compose.yml example
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  timeout: 3s
  retries: 3
```

### 3. Load Balancer Configuration
- Add `/health` as health check endpoint
- Set interval: 10 seconds
- Healthy threshold: 2 consecutive successes
- Unhealthy threshold: 3 consecutive failures

## Definition of Done

The task is complete when:

1. All acceptance criteria checkboxes are marked
2. All test cases pass
3. Validation script runs successfully
4. Load test shows acceptable performance
5. No regression in existing endpoints
6. Code follows project standards
7. Health endpoint is documented in README

## Common Failure Scenarios

1. **Timestamp not updating**: Ensure `new Date()` is called inside route handler
2. **Wrong status value**: Must be exactly "healthy" (not "ok" or "alive")
3. **Extra fields in response**: Remove any fields beyond status and timestamp
4. **Missing 200 status**: Explicitly set with `res.status(200)`
5. **Malformed timestamp**: Use `.toISOString()` without modification