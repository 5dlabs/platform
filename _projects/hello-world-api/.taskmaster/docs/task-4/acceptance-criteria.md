# Acceptance Criteria: Implement Health Check Endpoint

## Required Deliverables

### 1. Endpoint Implementation ✓
- [ ] GET route for `/health` is defined
- [ ] Route handler uses `app.get()`
- [ ] Handler placed after root endpoint
- [ ] Handler placed before error handlers

### 2. Response Structure ✓
- [ ] Response is JSON format
- [ ] Contains exactly two properties
- [ ] `status` property with value "healthy"
- [ ] `timestamp` property with ISO date string
- [ ] No additional properties

### 3. HTTP Standards ✓
- [ ] Status code is 200
- [ ] Content-Type is `application/json`
- [ ] Appropriate response headers

### 4. Timestamp Requirements ✓
- [ ] Uses `new Date().toISOString()`
- [ ] Format matches ISO 8601 standard
- [ ] Includes milliseconds
- [ ] UTC timezone (Z suffix)

## Test Cases

### Test Case 1: Basic Request
**Steps:**
1. Start server
2. Execute: `curl http://localhost:3000/health`

**Expected Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-26T10:30:45.123Z"
}
```

### Test Case 2: Full Response Headers
**Steps:**
1. Start server
2. Execute: `curl -i http://localhost:3000/health`

**Expected:**
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8

{
  "status": "healthy",
  "timestamp": "2024-01-26T10:30:45.123Z"
}
```

### Test Case 3: JSON Key Validation
**Steps:**
1. Execute: `curl -s http://localhost:3000/health | jq 'keys | sort'`

**Expected Output:**
```json
["status", "timestamp"]
```

### Test Case 4: Status Value Check
**Steps:**
1. Execute: `curl -s http://localhost:3000/health | jq -r .status`

**Expected Output:**
```
healthy
```

### Test Case 5: Timestamp Format Validation
**Steps:**
1. Execute: `curl -s http://localhost:3000/health | jq -r .timestamp`
2. Verify format matches: `YYYY-MM-DDTHH:mm:ss.sssZ`

**Expected:**
- Valid ISO 8601 date
- Includes milliseconds
- Ends with 'Z'

### Test Case 6: Timestamp Uniqueness
**Steps:**
1. Make 3 requests with 1-second delays:
```bash
for i in {1..3}; do
  curl -s http://localhost:3000/health | jq -r .timestamp
  sleep 1
done
```

**Expected:**
- Three different timestamps
- Each ~1 second apart

### Test Case 7: Method Restrictions
**Steps:**
1. Execute: `curl -X POST http://localhost:3000/health`
2. Execute: `curl -X PUT http://localhost:3000/health`
3. Execute: `curl -X DELETE http://localhost:3000/health`

**Expected for all:**
- Status: 404 Not Found
- Response: `{"error":"Not found"}`

### Test Case 8: Request Logging
**Steps:**
1. Execute: `curl http://localhost:3000/health`
2. Check server console

**Expected Log:**
- Format: `[timestamp] - GET /health`

## Performance Requirements

### Response Time
- [ ] Average < 5ms
- [ ] 95th percentile < 10ms
- [ ] 99th percentile < 20ms

### Load Testing
- [ ] Handle 1000 requests/second
- [ ] No memory leaks after 10,000 requests
- [ ] Consistent response times under load

### Benchmarking Test
```bash
# Apache Bench test
ab -n 1000 -c 10 http://localhost:3000/health
```

**Expected:**
- All requests successful
- Mean response time < 5ms
- No failed requests

## Monitoring Tool Compatibility

### Test Case 9: Prometheus Format
**Verification:**
- Response parseable by Prometheus
- Status field = "healthy" (string)
- Timestamp in ISO format

### Test Case 10: Kubernetes Probe
**Simulation:**
```bash
# Simulate K8s health probe
curl -f http://localhost:3000/health
echo $?  # Should be 0
```

### Test Case 11: Docker Health Check
**Command:**
```bash
docker exec [container] curl -f http://localhost:3000/health
```
**Expected:** Exit code 0

## Edge Cases

### Test Case 12: Rapid Sequential Requests
**Steps:**
```bash
for i in {1..100}; do
  curl -s http://localhost:3000/health &
done
wait
```
**Expected:**
- All requests succeed
- No errors or timeouts

### Test Case 13: Query Parameters
**Steps:**
1. Execute: `curl "http://localhost:3000/health?debug=true"`

**Expected:**
- Query parameters ignored
- Same response as without parameters

### Test Case 14: Request Headers
**Steps:**
1. Execute: `curl -H "X-Custom: test" http://localhost:3000/health`

**Expected:**
- Headers ignored
- Standard response returned

## Security Validation

### Information Disclosure
- [ ] No server version exposed
- [ ] No internal paths revealed
- [ ] No stack traces on errors

### Input Validation
- [ ] No user input processed
- [ ] No SQL injection possible
- [ ] No XSS vulnerabilities

## Automated Testing Script
```bash
#!/bin/bash
# Health endpoint test suite

echo "Running health endpoint tests..."

# Test 1: Endpoint exists
if curl -s -f http://localhost:3000/health > /dev/null; then
    echo "✓ Endpoint accessible"
else
    echo "✗ Endpoint not found"
    exit 1
fi

# Test 2: Status field
status=$(curl -s http://localhost:3000/health | jq -r .status)
if [ "$status" = "healthy" ]; then
    echo "✓ Status field correct"
else
    echo "✗ Status field incorrect: $status"
fi

# Test 3: Timestamp format
timestamp=$(curl -s http://localhost:3000/health | jq -r .timestamp)
if [[ $timestamp =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z$ ]]; then
    echo "✓ Timestamp format valid"
else
    echo "✗ Timestamp format invalid: $timestamp"
fi

# Test 4: Response time
start=$(date +%s%N)
curl -s http://localhost:3000/health > /dev/null
end=$(date +%s%N)
duration=$((($end - $start) / 1000000))
if [ $duration -lt 50 ]; then
    echo "✓ Response time OK: ${duration}ms"
else
    echo "✗ Response time slow: ${duration}ms"
fi

echo "Tests complete!"
```

## Definition of Done
- [ ] All test cases pass
- [ ] Performance requirements met
- [ ] Monitoring compatibility verified
- [ ] Security checks complete
- [ ] No console errors
- [ ] Documentation complete
- [ ] Code follows conventions