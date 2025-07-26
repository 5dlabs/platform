# Acceptance Criteria: Implement Health Check Endpoint

## Definition of Done
The health check endpoint is considered successfully implemented when all the following criteria are met:

## Required Outcomes

### 1. Endpoint Configuration ✓
- [ ] GET method handler exists at /health
- [ ] Route defined after root endpoint
- [ ] Route defined before 404 handler
- [ ] No route conflicts

### 2. Response Format ✓
Required JSON structure:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.123Z"
}
```
- [ ] Contains "status" property
- [ ] Status value is exactly "healthy"
- [ ] Contains "timestamp" property
- [ ] No additional properties

### 3. Timestamp Requirements ✓
- [ ] ISO 8601 format (YYYY-MM-DDTHH:mm:ss.sssZ)
- [ ] Includes milliseconds
- [ ] UTC timezone (Z suffix)
- [ ] Updates on each request

### 4. HTTP Response ✓
- [ ] Status code is 200
- [ ] Content-Type is application/json
- [ ] Response time < 100ms
- [ ] No caching headers set

## Test Cases

### Test Case 1: Basic Health Check
```bash
curl http://localhost:3000/health
```
**Expected Output:**
```json
{"status":"healthy","timestamp":"2024-01-15T14:32:17.123Z"}
```

### Test Case 2: Response Headers
```bash
curl -I http://localhost:3000/health
```
**Expected Headers:**
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
```

### Test Case 3: Timestamp Format Validation
```bash
curl -s http://localhost:3000/health | jq -r .timestamp | grep -E '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$'
```
**Expected:** Match found (exit code 0)

### Test Case 4: Dynamic Timestamp
```bash
timestamp1=$(curl -s http://localhost:3000/health | jq -r .timestamp)
sleep 1
timestamp2=$(curl -s http://localhost:3000/health | jq -r .timestamp)
[ "$timestamp1" != "$timestamp2" ] && echo "PASS" || echo "FAIL"
```
**Expected:** PASS

### Test Case 5: Concurrent Requests
```bash
for i in {1..10}; do
  curl -s http://localhost:3000/health &
done | wait
```
**Expected:** All requests succeed with valid responses

## Performance Testing

### Response Time Test
```bash
curl -w "\nTotal time: %{time_total}s\n" http://localhost:3000/health
```
**Expected:** Total time < 0.1s

### Load Test
```bash
# Using Apache Bench (if available)
ab -n 1000 -c 10 http://localhost:3000/health
```
**Expected:**
- All requests succeed
- Average response time < 50ms
- No errors

## Integration Testing

### Test with Monitoring Tools
```yaml
# Example Kubernetes probe
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```
**Expected:** Probe succeeds

### Test with Load Balancer
```bash
# AWS ALB health check simulation
curl -H "User-Agent: ELB-HealthChecker/2.0" http://localhost:3000/health
```
**Expected:** 200 OK response

## Edge Cases

### Test Case: Multiple Slashes
```bash
curl http://localhost:3000//health
```
**Expected:** 404 (Express normalizes paths)

### Test Case: Case Sensitivity
```bash
curl http://localhost:3000/Health
curl http://localhost:3000/HEALTH
```
**Expected:** 404 (routes are case-sensitive)

### Test Case: With Query Parameters
```bash
curl "http://localhost:3000/health?debug=true"
```
**Expected:** Parameters ignored, normal response

## Common Issues & Solutions

### Issue 1: Timestamp Not Updating
**Symptom**: Same timestamp on multiple requests
**Cause**: Timestamp calculated once at startup
**Fix**: Use `new Date().toISOString()` inside handler

### Issue 2: Invalid JSON
**Symptom**: Parse errors in monitoring tools
**Cause**: Manual JSON construction
**Fix**: Use `res.json()` method

### Issue 3: Wrong Content-Type
**Symptom**: text/html instead of application/json
**Cause**: Using res.send() instead of res.json()
**Fix**: Always use res.json() for JSON responses

### Issue 4: Timezone Issues
**Symptom**: Local time instead of UTC
**Cause**: Using toString() or toLocaleString()
**Fix**: Always use toISOString() for UTC

## Monitoring Integration

### Prometheus Format (Future Enhancement)
```
# HELP api_health_status API health status (1 = healthy, 0 = unhealthy)
# TYPE api_health_status gauge
api_health_status 1
```

### DataDog Check
```javascript
// Expected to work with DataDog HTTP check
{
  "check": "api.health",
  "status": 0,  // 0 = OK
  "timestamp": 1705329137
}
```

## Security Validation
- [ ] No sensitive data exposed
- [ ] No system information leaked
- [ ] Response size is minimal
- [ ] No user input processed

## Documentation Requirements
- [ ] Endpoint documented in README
- [ ] Response format explained
- [ ] Usage examples provided
- [ ] Monitoring setup documented

## Sign-off Checklist
- [ ] All test cases pass
- [ ] Performance requirements met
- [ ] Integration tests successful
- [ ] No regression in other endpoints
- [ ] Code follows project standards