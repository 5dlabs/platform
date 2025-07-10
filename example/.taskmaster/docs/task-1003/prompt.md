# AI Agent Prompt: Implement API Rate Limiting

## Task Context

You are tasked with implementing a comprehensive API rate limiting system for a microservice architecture. This system must prevent abuse, ensure fair usage, and support different rate limits for different endpoints and user tiers using Redis-backed distributed rate limiting.

## Primary Objective

Build a flexible and scalable rate limiting system that includes:
- Redis-based distributed rate limiting
- Sliding window and fixed window algorithms
- Configurable rate limit policies per endpoint and user tier
- Middleware integration for automatic enforcement
- Administrative interface for policy management
- Comprehensive monitoring and analytics

## Technical Requirements

### Core Components to Implement

1. **Rate Limiting Engine**
   - Sliding window algorithm implementation
   - Fixed window algorithm implementation
   - Token bucket algorithm (optional)
   - Redis-based counter management
   - Efficient key naming and cleanup

2. **Policy Management System**
   - Flexible policy configuration format
   - User tier-based rate limiting
   - Endpoint-specific rate limiting
   - Priority-based policy resolution
   - Administrative override capabilities

3. **Rate Limiting Middleware**
   - Automatic rate limit enforcement
   - HTTP header management
   - Proper error responses (429 Too Many Requests)
   - Bypass mechanisms for internal services
   - Integration with authentication system

4. **Configuration Management**
   - Dynamic policy loading
   - Configuration validation
   - Hot reloading of policies
   - Environment-specific configurations
   - Backup and restore capabilities

### Performance Requirements

- Rate limit check latency < 10ms
- Support for 10,000+ requests per second
- Memory-efficient Redis operations
- Minimal impact on API response times
- Efficient cleanup of expired counters

### Scalability Requirements

- Distributed rate limiting across multiple instances
- Horizontal scaling with Redis cluster
- Support for millions of unique rate limit keys
- Cross-instance coordination via Redis
- Load balancing compatibility

## Implementation Approach

### Phase 1: Core Rate Limiting Engine
1. Set up Redis connection and configuration
2. Implement sliding window rate limiting algorithm
3. Implement fixed window rate limiting algorithm
4. Create efficient key management system
5. Add counter cleanup and expiration logic

### Phase 2: Policy Management System
1. Design policy configuration schema
2. Create policy matching and resolution logic
3. Implement user tier and endpoint-based policies
4. Add priority-based policy selection
5. Create policy validation and error handling

### Phase 3: Middleware Integration
1. Create rate limiting middleware for Express/Koa
2. Implement HTTP header management
3. Add proper error responses and status codes
4. Create bypass mechanisms for internal services
5. Integrate with authentication middleware

### Phase 4: Administration and Monitoring
1. Create admin API for policy management
2. Implement rate limit monitoring and metrics
3. Add violation logging and analytics
4. Create dashboard for operational visibility
5. Set up alerting for abuse patterns

## Code Structure Expectations

```
src/
├── rate-limiting/
│   ├── algorithms/
│   │   ├── sliding-window.js
│   │   ├── fixed-window.js
│   │   └── token-bucket.js
│   ├── middleware/
│   │   ├── rate-limit.middleware.js
│   │   └── admin.middleware.js
│   ├── policies/
│   │   ├── policy-manager.js
│   │   ├── policy-matcher.js
│   │   └── policy-validator.js
│   ├── storage/
│   │   ├── redis-client.js
│   │   └── key-manager.js
│   ├── monitoring/
│   │   ├── metrics.js
│   │   └── dashboard.js
│   └── config/
│       ├── policies.json
│       └── settings.js
├── controllers/
│   ├── rate-limit.controller.js
│   └── admin.controller.js
└── tests/
    ├── unit/
    ├── integration/
    └── performance/
```

## Rate Limiting Algorithm Implementation

### Sliding Window Algorithm
```javascript
class SlidingWindowRateLimiter {
  constructor(redis, window = 60, limit = 100) {
    this.redis = redis;
    this.window = window; // seconds
    this.limit = limit;
  }

  async checkLimit(key) {
    const now = Date.now();
    const pipeline = this.redis.pipeline();
    
    // Remove expired entries
    pipeline.zremrangebyscore(key, 0, now - (this.window * 1000));
    
    // Count current requests
    pipeline.zcard(key);
    
    // Add current request
    pipeline.zadd(key, now, `${now}-${Math.random()}`);
    
    // Set expiration
    pipeline.expire(key, this.window);
    
    const results = await pipeline.exec();
    const count = results[1][1];
    
    return {
      allowed: count < this.limit,
      count: count,
      limit: this.limit,
      remaining: Math.max(0, this.limit - count - 1),
      resetTime: now + (this.window * 1000)
    };
  }
}
```

### Fixed Window Algorithm
```javascript
class FixedWindowRateLimiter {
  constructor(redis, window = 60, limit = 100) {
    this.redis = redis;
    this.window = window;
    this.limit = limit;
  }

  async checkLimit(key) {
    const now = Date.now();
    const windowStart = Math.floor(now / (this.window * 1000)) * (this.window * 1000);
    const windowKey = `${key}:${windowStart}`;
    
    const pipeline = this.redis.pipeline();
    pipeline.incr(windowKey);
    pipeline.expire(windowKey, this.window);
    
    const results = await pipeline.exec();
    const count = results[0][1];
    
    return {
      allowed: count <= this.limit,
      count: count,
      limit: this.limit,
      remaining: Math.max(0, this.limit - count),
      resetTime: windowStart + (this.window * 1000)
    };
  }
}
```

## Policy Configuration Schema

### Rate Limit Policy Format
```json
{
  "id": "premium-user",
  "name": "Premium User Policy",
  "description": "Higher limits for premium users",
  "enabled": true,
  "priority": 1,
  "conditions": {
    "userTiers": ["premium", "enterprise"],
    "endpoints": ["*"],
    "methods": ["*"],
    "ipRanges": []
  },
  "limits": {
    "requests_per_second": 50,
    "requests_per_minute": 1000,
    "requests_per_hour": 10000,
    "requests_per_day": 100000
  },
  "algorithm": "sliding_window",
  "actions": {
    "onExceeded": "block",
    "responseCode": 429,
    "responseMessage": "Rate limit exceeded"
  }
}
```

## Middleware Implementation

### Rate Limiting Middleware
```javascript
const rateLimitMiddleware = (options = {}) => {
  const rateLimiter = new RateLimiter(options);
  
  return async (req, res, next) => {
    try {
      // Extract identification keys
      const identifiers = await extractIdentifiers(req);
      
      // Check for bypass conditions
      if (await shouldBypassRateLimit(req, identifiers)) {
        return next();
      }
      
      // Get applicable policies
      const policies = await rateLimiter.getApplicablePolicies(req, identifiers);
      
      // Check rate limits
      const results = await rateLimiter.checkLimits(policies, identifiers);
      
      // Set response headers
      setRateLimitHeaders(res, results);
      
      // Check if any limit is exceeded
      const violation = results.find(r => !r.allowed);
      if (violation) {
        await logViolation(req, identifiers, violation);
        return res.status(429).json({
          error: {
            code: 'RATE_LIMIT_EXCEEDED',
            message: 'Rate limit exceeded',
            details: {
              limit: violation.limit,
              window: violation.window,
              retryAfter: violation.retryAfter
            }
          }
        });
      }
      
      next();
    } catch (error) {
      console.error('Rate limiting error:', error);
      // Fail open - allow request if rate limiter fails
      next();
    }
  };
};
```

### Response Header Management
```javascript
function setRateLimitHeaders(res, results) {
  if (results.length === 0) return;
  
  // Use the most restrictive limit for headers
  const primaryResult = results.reduce((prev, curr) => 
    curr.remaining < prev.remaining ? curr : prev
  );
  
  res.set({
    'X-RateLimit-Limit': primaryResult.limit,
    'X-RateLimit-Remaining': primaryResult.remaining,
    'X-RateLimit-Reset': Math.ceil(primaryResult.resetTime / 1000),
    'X-RateLimit-Policy': primaryResult.policy
  });
  
  if (primaryResult.retryAfter) {
    res.set('Retry-After', primaryResult.retryAfter);
  }
}
```

## Redis Integration

### Redis Client Setup
```javascript
const Redis = require('ioredis');

class RedisRateLimitStorage {
  constructor(options = {}) {
    this.redis = new Redis({
      host: options.host || 'localhost',
      port: options.port || 6379,
      db: options.db || 0,
      keyPrefix: options.keyPrefix || 'rate_limit:',
      retryDelayOnFailover: 100,
      enableReadyCheck: true,
      maxRetriesPerRequest: 3
    });
    
    this.redis.on('error', (error) => {
      console.error('Redis connection error:', error);
    });
  }

  async checkAndIncrement(key, limit, window, algorithm = 'sliding_window') {
    const limiter = this.getLimiter(algorithm);
    return limiter.checkLimit(key, limit, window);
  }

  async cleanupExpiredKeys() {
    const stream = this.redis.scanStream({
      match: `${this.redis.options.keyPrefix}*`,
      count: 100
    });
    
    let deletedCount = 0;
    stream.on('data', async (keys) => {
      for (const key of keys) {
        const ttl = await this.redis.ttl(key);
        if (ttl === -1) { // Key without expiration
          await this.redis.del(key);
          deletedCount++;
        }
      }
    });
    
    stream.on('end', () => {
      console.log(`Cleaned up ${deletedCount} expired rate limit keys`);
    });
  }
}
```

## Monitoring and Analytics

### Metrics Collection
```javascript
const prometheus = require('prom-client');

const rateLimitMetrics = {
  requestsChecked: new prometheus.Counter({
    name: 'rate_limit_requests_checked_total',
    help: 'Total number of requests checked for rate limiting',
    labelNames: ['policy', 'endpoint', 'method', 'user_tier']
  }),
  
  requestsBlocked: new prometheus.Counter({
    name: 'rate_limit_requests_blocked_total',
    help: 'Total number of requests blocked by rate limiting',
    labelNames: ['policy', 'endpoint', 'method', 'user_tier', 'window']
  }),
  
  checkLatency: new prometheus.Histogram({
    name: 'rate_limit_check_duration_seconds',
    help: 'Time taken to check rate limits',
    buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
  }),
  
  policyUtilization: new prometheus.Gauge({
    name: 'rate_limit_policy_utilization_percent',
    help: 'Current utilization of rate limit policies',
    labelNames: ['policy', 'window']
  })
};

class RateLimitMonitor {
  constructor(redis) {
    this.redis = redis;
    this.metrics = rateLimitMetrics;
  }

  recordRequest(policy, endpoint, method, userTier) {
    this.metrics.requestsChecked.inc({ policy, endpoint, method, user_tier: userTier });
  }

  recordBlocked(policy, endpoint, method, userTier, window) {
    this.metrics.requestsBlocked.inc({ policy, endpoint, method, user_tier: userTier, window });
  }

  recordLatency(duration) {
    this.metrics.checkLatency.observe(duration);
  }

  async updateUtilization(policy, window, current, limit) {
    const utilization = (current / limit) * 100;
    this.metrics.policyUtilization.set({ policy, window }, utilization);
  }
}
```

## Administrative Interface

### Policy Management API
```javascript
class RateLimitController {
  constructor(policyManager) {
    this.policyManager = policyManager;
  }

  async getPolicies(req, res) {
    try {
      const policies = await this.policyManager.getAllPolicies();
      res.json({ success: true, data: { policies } });
    } catch (error) {
      res.status(500).json({ success: false, error: error.message });
    }
  }

  async createPolicy(req, res) {
    try {
      const policy = await this.policyManager.createPolicy(req.body);
      res.status(201).json({ success: true, data: { policy } });
    } catch (error) {
      res.status(400).json({ success: false, error: error.message });
    }
  }

  async updatePolicy(req, res) {
    try {
      const policy = await this.policyManager.updatePolicy(req.params.id, req.body);
      res.json({ success: true, data: { policy } });
    } catch (error) {
      res.status(400).json({ success: false, error: error.message });
    }
  }

  async deletePolicy(req, res) {
    try {
      await this.policyManager.deletePolicy(req.params.id);
      res.json({ success: true });
    } catch (error) {
      res.status(400).json({ success: false, error: error.message });
    }
  }

  async getRateLimitStatus(req, res) {
    try {
      const identifiers = await extractIdentifiers(req);
      const status = await this.policyManager.getRateLimitStatus(identifiers);
      res.json({ success: true, data: status });
    } catch (error) {
      res.status(500).json({ success: false, error: error.message });
    }
  }
}
```

## Testing Requirements

### Unit Tests (Minimum Coverage: 90%)
- Rate limiting algorithm accuracy
- Policy matching and resolution logic
- Redis operations and error handling
- Configuration validation
- Identifier extraction and key building

### Integration Tests
- Middleware integration with Express/Koa
- End-to-end rate limiting scenarios
- Multi-policy enforcement
- Redis cluster functionality
- Admin API operations

### Performance Tests
- Rate limiter latency under load (target: <10ms)
- Memory usage with millions of keys
- Redis performance with high concurrency
- Cleanup efficiency for expired keys
- Throughput testing (target: 10,000+ RPS)

### Security Tests
- Rate limit bypass attempts
- Policy manipulation testing
- Redis security configuration
- Input validation and sanitization
- Authentication and authorization

## Environment Configuration

Required environment variables:
```env
# Redis Configuration
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_DB=0
REDIS_PASSWORD=secure_password
REDIS_KEY_PREFIX=rate_limit:

# Rate Limiting Configuration
RATE_LIMIT_DEFAULT_ALGORITHM=sliding_window
RATE_LIMIT_CLEANUP_INTERVAL=300000
RATE_LIMIT_FAIL_OPEN=true

# Monitoring Configuration
RATE_LIMIT_METRICS_ENABLED=true
RATE_LIMIT_MONITORING_INTERVAL=60000
RATE_LIMIT_ALERT_THRESHOLD=0.8

# Admin Configuration
RATE_LIMIT_ADMIN_ENABLED=true
RATE_LIMIT_ADMIN_AUTH_REQUIRED=true
```

## Quality Assurance Checklist

Before marking this task complete, ensure:

- [ ] Sliding window algorithm is accurately implemented
- [ ] Fixed window algorithm is working correctly
- [ ] Policy matching logic handles all conditions properly
- [ ] Redis operations are efficient and error-resistant
- [ ] Middleware integrates seamlessly with existing code
- [ ] Rate limit headers are set correctly
- [ ] Error responses follow HTTP standards
- [ ] Admin API allows policy management
- [ ] Monitoring and metrics are comprehensive
- [ ] Performance requirements are met (<10ms latency)
- [ ] Security measures prevent common attacks
- [ ] Cleanup processes handle expired keys efficiently
- [ ] Configuration validation prevents invalid policies
- [ ] Tests achieve minimum 90% coverage
- [ ] Documentation is complete and accurate

## Success Metrics

- Rate limit check latency consistently < 10ms
- System handles 10,000+ requests per second
- Zero data loss during Redis failover
- Policy changes take effect within 60 seconds
- Test coverage > 90%
- All integration tests passing
- Performance requirements met under load
- Security tests show no critical vulnerabilities

## Important Notes

1. **Performance Priority**: Rate limiting must not become a bottleneck
2. **Fail-Safe Design**: System should fail open if Redis is unavailable
3. **Accuracy**: Sliding window algorithm must be precise for fair limiting
4. **Scalability**: Design for distributed deployment from the start
5. **Monitoring**: Comprehensive metrics are essential for operations
6. **Security**: Prevent bypass attempts and abuse patterns
7. **Usability**: Admin interface should be intuitive and powerful

Begin implementation with the core rate limiting algorithms and Redis integration. Focus on performance and accuracy throughout the development process.