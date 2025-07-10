# Technical Design Specification: API Rate Limiting

## 1. System Architecture Overview

### 1.1 High-Level Architecture

The API rate limiting system follows a distributed architecture with Redis as the central coordination layer:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Gateway   │    │  Load Balancer  │    │  App Server 1   │
│ (Rate Limiting) │◄──►│  (Consistent    │◄──►│ (Rate Limiting  │
│                 │    │   Hashing)      │    │  Middleware)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                       ┌─────────────────┐             │
                       │  App Server 2   │◄────────────┤
                       │ (Rate Limiting  │             │
                       │  Middleware)    │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  App Server N   │◄────────────┤
                       │ (Rate Limiting  │             │
                       │  Middleware)    │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  Redis Cluster  │◄────────────┘
                       │ (Rate Counters  │
                       │  & Policies)    │
                       └─────────────────┘
                                │
                       ┌─────────────────┐
                       │  Admin Panel    │
                       │ (Policy Mgmt)   │
                       └─────────────────┘
```

### 1.2 Component Responsibilities

- **Rate Limiting Middleware**: Intercepts requests, checks limits, enforces policies
- **Redis Cluster**: Stores rate counters, policies, and coordination data
- **Policy Manager**: Manages rate limit policies and their application
- **Admin Panel**: Provides interface for policy management and monitoring
- **Metrics Collector**: Gathers performance and usage statistics

## 2. Rate Limiting Algorithms

### 2.1 Sliding Window Algorithm

The sliding window algorithm provides the most accurate rate limiting by maintaining a sorted set of request timestamps.

```javascript
class SlidingWindowRateLimiter {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.keyPrefix = options.keyPrefix || 'rate_limit:sw:';
    this.defaultTTL = options.defaultTTL || 3600; // 1 hour
  }

  async checkLimit(identifier, limit, windowSizeSeconds) {
    const key = `${this.keyPrefix}${identifier}`;
    const now = Date.now();
    const windowStart = now - (windowSizeSeconds * 1000);
    
    const pipeline = this.redis.pipeline();
    
    // Remove expired entries
    pipeline.zremrangebyscore(key, 0, windowStart);
    
    // Count current requests in window
    pipeline.zcard(key);
    
    // Add current request
    const requestId = `${now}-${Math.random().toString(36).substr(2, 9)}`;
    pipeline.zadd(key, now, requestId);
    
    // Set expiration
    pipeline.expire(key, this.defaultTTL);
    
    const results = await pipeline.exec();
    const currentCount = results[1][1];
    
    // Check if limit is exceeded
    const allowed = currentCount < limit;
    const remaining = Math.max(0, limit - currentCount - 1);
    
    // Calculate reset time (end of current window)
    const resetTime = now + (windowSizeSeconds * 1000);
    
    if (!allowed) {
      // Remove the request we just added since it's not allowed
      await this.redis.zrem(key, requestId);
    }
    
    return {
      allowed,
      count: currentCount + (allowed ? 1 : 0),
      limit,
      remaining,
      resetTime,
      retryAfter: this.calculateRetryAfter(key, windowSizeSeconds)
    };
  }

  async calculateRetryAfter(key, windowSizeSeconds) {
    // Get the oldest request in the window
    const oldest = await this.redis.zrange(key, 0, 0, 'WITHSCORES');
    if (oldest.length === 0) return 0;
    
    const oldestTime = parseInt(oldest[1]);
    const windowStart = Date.now() - (windowSizeSeconds * 1000);
    
    return Math.max(0, Math.ceil((oldestTime - windowStart) / 1000));
  }
}
```

### 2.2 Fixed Window Algorithm

The fixed window algorithm divides time into fixed intervals and counts requests within each interval.

```javascript
class FixedWindowRateLimiter {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.keyPrefix = options.keyPrefix || 'rate_limit:fw:';
  }

  async checkLimit(identifier, limit, windowSizeSeconds) {
    const now = Date.now();
    const windowStart = Math.floor(now / (windowSizeSeconds * 1000)) * (windowSizeSeconds * 1000);
    const key = `${this.keyPrefix}${identifier}:${windowStart}`;
    
    const pipeline = this.redis.pipeline();
    pipeline.incr(key);
    pipeline.expire(key, windowSizeSeconds);
    
    const results = await pipeline.exec();
    const currentCount = results[0][1];
    
    const allowed = currentCount <= limit;
    const remaining = Math.max(0, limit - currentCount);
    const resetTime = windowStart + (windowSizeSeconds * 1000);
    
    return {
      allowed,
      count: currentCount,
      limit,
      remaining,
      resetTime,
      retryAfter: allowed ? 0 : Math.ceil((resetTime - now) / 1000)
    };
  }
}
```

### 2.3 Token Bucket Algorithm

The token bucket algorithm allows burst traffic while maintaining an average rate.

```javascript
class TokenBucketRateLimiter {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.keyPrefix = options.keyPrefix || 'rate_limit:tb:';
  }

  async checkLimit(identifier, limit, refillRate, bucketSize = limit) {
    const key = `${this.keyPrefix}${identifier}`;
    const now = Date.now();
    
    // Lua script for atomic token bucket operations
    const script = `
      local key = KEYS[1]
      local now = tonumber(ARGV[1])
      local refill_rate = tonumber(ARGV[2])
      local bucket_size = tonumber(ARGV[3])
      local tokens_requested = tonumber(ARGV[4])
      
      local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
      local tokens = tonumber(bucket[1]) or bucket_size
      local last_refill = tonumber(bucket[2]) or now
      
      -- Calculate tokens to add based on time elapsed
      local time_elapsed = (now - last_refill) / 1000
      local tokens_to_add = math.floor(time_elapsed * refill_rate)
      tokens = math.min(bucket_size, tokens + tokens_to_add)
      
      local allowed = tokens >= tokens_requested
      if allowed then
        tokens = tokens - tokens_requested
      end
      
      -- Update bucket state
      redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
      redis.call('EXPIRE', key, 3600)
      
      return {allowed and 1 or 0, tokens, bucket_size}
    `;
    
    const result = await this.redis.eval(script, 1, key, now, refillRate, bucketSize, 1);
    const [allowed, tokens, capacity] = result;
    
    return {
      allowed: allowed === 1,
      tokens: tokens,
      capacity: capacity,
      resetTime: now + ((capacity - tokens) / refillRate) * 1000
    };
  }
}
```

## 3. Policy Management System

### 3.1 Policy Configuration Schema

```javascript
const policySchema = {
  type: 'object',
  properties: {
    id: { type: 'string', pattern: '^[a-zA-Z0-9_-]+$' },
    name: { type: 'string', maxLength: 255 },
    description: { type: 'string', maxLength: 1000 },
    enabled: { type: 'boolean', default: true },
    priority: { type: 'integer', minimum: 0, maximum: 100 },
    conditions: {
      type: 'object',
      properties: {
        userTiers: { type: 'array', items: { type: 'string' } },
        endpoints: { type: 'array', items: { type: 'string' } },
        methods: { type: 'array', items: { type: 'string' } },
        ipRanges: { type: 'array', items: { type: 'string' } },
        headers: { type: 'object' },
        timeRanges: { type: 'array', items: { type: 'object' } }
      }
    },
    limits: {
      type: 'object',
      properties: {
        requests_per_second: { type: 'integer', minimum: 1 },
        requests_per_minute: { type: 'integer', minimum: 1 },
        requests_per_hour: { type: 'integer', minimum: 1 },
        requests_per_day: { type: 'integer', minimum: 1 }
      },
      minProperties: 1
    },
    algorithm: {
      type: 'string',
      enum: ['sliding_window', 'fixed_window', 'token_bucket']
    },
    actions: {
      type: 'object',
      properties: {
        onExceeded: { type: 'string', enum: ['block', 'delay', 'log'] },
        responseCode: { type: 'integer', minimum: 400, maximum: 599 },
        responseMessage: { type: 'string' },
        delaySeconds: { type: 'integer', minimum: 1 }
      }
    }
  },
  required: ['id', 'name', 'limits', 'algorithm'],
  additionalProperties: false
};
```

### 3.2 Policy Manager Implementation

```javascript
class PolicyManager {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.policiesKey = options.policiesKey || 'rate_limit:policies';
    this.cache = new Map();
    this.cacheTimeout = options.cacheTimeout || 60000; // 1 minute
  }

  async createPolicy(policyData) {
    // Validate policy data
    const { error, value } = this.validatePolicy(policyData);
    if (error) {
      throw new Error(`Invalid policy: ${error.message}`);
    }

    // Check for duplicate ID
    const existingPolicy = await this.getPolicy(value.id);
    if (existingPolicy) {
      throw new Error(`Policy with ID '${value.id}' already exists`);
    }

    // Store policy
    const policy = {
      ...value,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };

    await this.redis.hset(this.policiesKey, policy.id, JSON.stringify(policy));
    this.cache.delete('all_policies');
    
    return policy;
  }

  async updatePolicy(policyId, updates) {
    const existingPolicy = await this.getPolicy(policyId);
    if (!existingPolicy) {
      throw new Error(`Policy with ID '${policyId}' not found`);
    }

    const updatedPolicy = {
      ...existingPolicy,
      ...updates,
      updatedAt: new Date().toISOString()
    };

    // Validate updated policy
    const { error, value } = this.validatePolicy(updatedPolicy);
    if (error) {
      throw new Error(`Invalid policy update: ${error.message}`);
    }

    await this.redis.hset(this.policiesKey, policyId, JSON.stringify(value));
    this.cache.delete('all_policies');
    this.cache.delete(policyId);
    
    return value;
  }

  async getPolicy(policyId) {
    const cached = this.cache.get(policyId);
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      return cached.policy;
    }

    const policyData = await this.redis.hget(this.policiesKey, policyId);
    if (!policyData) return null;

    const policy = JSON.parse(policyData);
    this.cache.set(policyId, { policy, timestamp: Date.now() });
    
    return policy;
  }

  async getAllPolicies() {
    const cached = this.cache.get('all_policies');
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      return cached.policies;
    }

    const policiesData = await this.redis.hgetall(this.policiesKey);
    const policies = Object.values(policiesData).map(data => JSON.parse(data));
    
    this.cache.set('all_policies', { policies, timestamp: Date.now() });
    
    return policies;
  }

  async deletePolicy(policyId) {
    const existingPolicy = await this.getPolicy(policyId);
    if (!existingPolicy) {
      throw new Error(`Policy with ID '${policyId}' not found`);
    }

    await this.redis.hdel(this.policiesKey, policyId);
    this.cache.delete('all_policies');
    this.cache.delete(policyId);
    
    return true;
  }

  async getApplicablePolicies(request, identifiers) {
    const allPolicies = await this.getAllPolicies();
    
    const applicablePolicies = allPolicies
      .filter(policy => policy.enabled && this.policyMatches(policy, request, identifiers))
      .sort((a, b) => b.priority - a.priority);
    
    return applicablePolicies;
  }

  policyMatches(policy, request, identifiers) {
    const { conditions } = policy;
    
    // Check user tiers
    if (conditions.userTiers && conditions.userTiers.length > 0) {
      if (!conditions.userTiers.includes('*') && 
          !conditions.userTiers.includes(identifiers.userTier)) {
        return false;
      }
    }
    
    // Check endpoints
    if (conditions.endpoints && conditions.endpoints.length > 0) {
      if (!conditions.endpoints.includes('*') && 
          !conditions.endpoints.some(pattern => this.matchesEndpoint(request.path, pattern))) {
        return false;
      }
    }
    
    // Check HTTP methods
    if (conditions.methods && conditions.methods.length > 0) {
      if (!conditions.methods.includes('*') && 
          !conditions.methods.includes(request.method)) {
        return false;
      }
    }
    
    // Check IP ranges
    if (conditions.ipRanges && conditions.ipRanges.length > 0) {
      if (!this.matchesIPRange(identifiers.ipAddress, conditions.ipRanges)) {
        return false;
      }
    }
    
    // Check headers
    if (conditions.headers) {
      if (!this.matchesHeaders(request.headers, conditions.headers)) {
        return false;
      }
    }
    
    // Check time ranges
    if (conditions.timeRanges && conditions.timeRanges.length > 0) {
      if (!this.matchesTimeRange(new Date(), conditions.timeRanges)) {
        return false;
      }
    }
    
    return true;
  }

  matchesEndpoint(path, pattern) {
    // Convert glob pattern to regex
    const regexPattern = pattern
      .replace(/\*/g, '.*')
      .replace(/\?/g, '.')
      .replace(/\[([^\]]+)\]/g, '[$1]');
    
    const regex = new RegExp(`^${regexPattern}$`);
    return regex.test(path);
  }

  matchesIPRange(ip, ranges) {
    const ipaddr = require('ipaddr.js');
    
    try {
      const parsedIP = ipaddr.process(ip);
      return ranges.some(range => {
        try {
          const [subnet, prefix] = range.split('/');
          const parsedSubnet = ipaddr.process(subnet);
          return parsedIP.match(parsedSubnet, parseInt(prefix));
        } catch (e) {
          return false;
        }
      });
    } catch (e) {
      return false;
    }
  }

  matchesHeaders(requestHeaders, conditionHeaders) {
    for (const [key, value] of Object.entries(conditionHeaders)) {
      const headerValue = requestHeaders[key.toLowerCase()];
      if (!headerValue || !this.matchesHeaderValue(headerValue, value)) {
        return false;
      }
    }
    return true;
  }

  matchesHeaderValue(actual, expected) {
    if (typeof expected === 'string') {
      return actual === expected;
    } else if (expected.regex) {
      return new RegExp(expected.regex).test(actual);
    } else if (expected.contains) {
      return actual.includes(expected.contains);
    }
    return false;
  }

  matchesTimeRange(now, timeRanges) {
    return timeRanges.some(range => {
      const startTime = new Date(range.start);
      const endTime = new Date(range.end);
      return now >= startTime && now <= endTime;
    });
  }

  validatePolicy(policy) {
    const Joi = require('joi');
    const schema = Joi.object(policySchema);
    return schema.validate(policy);
  }
}
```

## 4. Rate Limiting Engine

### 4.1 Core Rate Limiter

```javascript
class RateLimiter {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.algorithms = {
      sliding_window: new SlidingWindowRateLimiter(redis, options),
      fixed_window: new FixedWindowRateLimiter(redis, options),
      token_bucket: new TokenBucketRateLimiter(redis, options)
    };
    this.policyManager = new PolicyManager(redis, options);
    this.keyBuilder = new KeyBuilder(options);
  }

  async checkRateLimit(request, identifiers) {
    const startTime = Date.now();
    
    try {
      // Get applicable policies
      const policies = await this.policyManager.getApplicablePolicies(request, identifiers);
      
      if (policies.length === 0) {
        return { allowed: true, policies: [] };
      }

      // Check each policy
      const results = [];
      for (const policy of policies) {
        const policyResult = await this.checkPolicy(policy, identifiers);
        results.push({
          policy: policy.id,
          ...policyResult
        });
        
        // If any policy blocks the request, return immediately
        if (!policyResult.allowed) {
          return {
            allowed: false,
            policies: results,
            blockedBy: policy.id,
            retryAfter: policyResult.retryAfter
          };
        }
      }

      return {
        allowed: true,
        policies: results
      };
    } finally {
      const duration = Date.now() - startTime;
      this.recordMetrics(request, identifiers, duration);
    }
  }

  async checkPolicy(policy, identifiers) {
    const algorithm = this.algorithms[policy.algorithm];
    if (!algorithm) {
      throw new Error(`Unknown algorithm: ${policy.algorithm}`);
    }

    const results = {};
    
    // Check each time window in the policy
    for (const [window, limit] of Object.entries(policy.limits)) {
      const windowSeconds = this.parseTimeWindow(window);
      const key = this.keyBuilder.buildKey(policy.id, identifiers, window);
      
      const result = await algorithm.checkLimit(key, limit, windowSeconds);
      results[window] = result;
      
      // If any window is exceeded, the policy is violated
      if (!result.allowed) {
        return {
          allowed: false,
          window,
          limit,
          current: result.count,
          retryAfter: result.retryAfter,
          resetTime: result.resetTime
        };
      }
    }

    // Find the most restrictive limit for response headers
    const mostRestrictive = Object.values(results).reduce((prev, curr) => 
      curr.remaining < prev.remaining ? curr : prev
    );

    return {
      allowed: true,
      results,
      limit: mostRestrictive.limit,
      remaining: mostRestrictive.remaining,
      resetTime: mostRestrictive.resetTime
    };
  }

  parseTimeWindow(window) {
    const windowMap = {
      'requests_per_second': 1,
      'requests_per_minute': 60,
      'requests_per_hour': 3600,
      'requests_per_day': 86400
    };
    
    return windowMap[window] || 60;
  }

  recordMetrics(request, identifiers, duration) {
    // Record metrics for monitoring
    if (this.metricsCollector) {
      this.metricsCollector.recordCheck(request, identifiers, duration);
    }
  }
}
```

### 4.2 Key Builder

```javascript
class KeyBuilder {
  constructor(options = {}) {
    this.prefix = options.keyPrefix || 'rate_limit:';
    this.separator = options.separator || ':';
  }

  buildKey(policyId, identifiers, window) {
    const parts = [this.prefix, policyId, window];
    
    // Add user identifier if available
    if (identifiers.userId) {
      parts.push(`user`, identifiers.userId);
    } else if (identifiers.sessionId) {
      parts.push(`session`, identifiers.sessionId);
    }
    
    // Add IP address for additional tracking
    if (identifiers.ipAddress) {
      parts.push(`ip`, this.hashIP(identifiers.ipAddress));
    }
    
    return parts.join(this.separator);
  }

  hashIP(ip) {
    const crypto = require('crypto');
    return crypto.createHash('sha256').update(ip).digest('hex').substring(0, 16);
  }
}
```

## 5. Middleware Implementation

### 5.1 Express Middleware

```javascript
function createRateLimitMiddleware(options = {}) {
  const rateLimiter = new RateLimiter(options.redis, options);
  
  return async (req, res, next) => {
    try {
      // Extract identifiers from request
      const identifiers = await extractIdentifiers(req);
      
      // Check for bypass conditions
      if (await shouldBypassRateLimit(req, identifiers, options)) {
        return next();
      }
      
      // Check rate limits
      const result = await rateLimiter.checkRateLimit(req, identifiers);
      
      // Set response headers
      setRateLimitHeaders(res, result);
      
      if (!result.allowed) {
        // Log violation
        await logRateLimitViolation(req, identifiers, result);
        
        // Return rate limit exceeded response
        return res.status(429).json({
          error: {
            code: 'RATE_LIMIT_EXCEEDED',
            message: 'Rate limit exceeded',
            details: {
              policy: result.blockedBy,
              limit: result.limit,
              retryAfter: result.retryAfter
            }
          }
        });
      }
      
      // Continue to next middleware
      next();
    } catch (error) {
      console.error('Rate limiting error:', error);
      
      // Fail open - allow request if rate limiter fails
      if (options.failOpen !== false) {
        return next();
      }
      
      // Fail closed - block request on error
      return res.status(503).json({
        error: {
          code: 'SERVICE_UNAVAILABLE',
          message: 'Rate limiting service unavailable'
        }
      });
    }
  };
}

async function extractIdentifiers(req) {
  const identifiers = {};
  
  // Extract user ID from authenticated request
  if (req.user && req.user.id) {
    identifiers.userId = req.user.id;
    identifiers.userTier = req.user.tier || 'free';
  }
  
  // Extract session ID from session
  if (req.session && req.session.id) {
    identifiers.sessionId = req.session.id;
  }
  
  // Extract IP address
  identifiers.ipAddress = req.ip || req.connection.remoteAddress;
  
  // Extract API key if present
  if (req.headers['x-api-key']) {
    identifiers.apiKey = req.headers['x-api-key'];
  }
  
  return identifiers;
}

async function shouldBypassRateLimit(req, identifiers, options) {
  // Check for admin bypass
  if (req.user && req.user.roles && req.user.roles.includes('admin')) {
    return true;
  }
  
  // Check for internal service bypass
  if (req.headers['x-internal-service']) {
    return true;
  }
  
  // Check for IP whitelist
  if (options.ipWhitelist && options.ipWhitelist.includes(identifiers.ipAddress)) {
    return true;
  }
  
  return false;
}

function setRateLimitHeaders(res, result) {
  if (!result.policies || result.policies.length === 0) {
    return;
  }
  
  // Use the most restrictive policy for headers
  const mostRestrictive = result.policies.reduce((prev, curr) => 
    curr.remaining < prev.remaining ? curr : prev
  );
  
  res.set({
    'X-RateLimit-Limit': mostRestrictive.limit,
    'X-RateLimit-Remaining': mostRestrictive.remaining,
    'X-RateLimit-Reset': Math.ceil(mostRestrictive.resetTime / 1000),
    'X-RateLimit-Policy': mostRestrictive.policy
  });
  
  if (result.retryAfter) {
    res.set('Retry-After', result.retryAfter);
  }
}
```

## 6. Monitoring and Analytics

### 6.1 Metrics Collection

```javascript
const prometheus = require('prom-client');

class RateLimitMetrics {
  constructor() {
    this.metrics = {
      requestsChecked: new prometheus.Counter({
        name: 'rate_limit_requests_checked_total',
        help: 'Total number of requests checked for rate limiting',
        labelNames: ['policy', 'endpoint', 'method', 'user_tier', 'status']
      }),
      
      requestsBlocked: new prometheus.Counter({
        name: 'rate_limit_requests_blocked_total',
        help: 'Total number of requests blocked by rate limiting',
        labelNames: ['policy', 'endpoint', 'method', 'user_tier', 'window']
      }),
      
      checkDuration: new prometheus.Histogram({
        name: 'rate_limit_check_duration_seconds',
        help: 'Time taken to check rate limits',
        buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0],
        labelNames: ['policy_count']
      }),
      
      policyUtilization: new prometheus.Gauge({
        name: 'rate_limit_policy_utilization_percent',
        help: 'Current utilization of rate limit policies',
        labelNames: ['policy', 'window', 'user_tier']
      }),
      
      redisOperations: new prometheus.Counter({
        name: 'rate_limit_redis_operations_total',
        help: 'Total number of Redis operations',
        labelNames: ['operation', 'status']
      })
    };
  }

  recordCheck(request, identifiers, duration, policies, result) {
    const labels = {
      policy: result.blockedBy || 'allowed',
      endpoint: this.sanitizeEndpoint(request.path),
      method: request.method,
      user_tier: identifiers.userTier || 'anonymous',
      status: result.allowed ? 'allowed' : 'blocked'
    };
    
    this.metrics.requestsChecked.inc(labels);
    this.metrics.checkDuration.observe(
      { policy_count: policies.length },
      duration / 1000
    );
    
    if (!result.allowed) {
      this.metrics.requestsBlocked.inc({
        ...labels,
        window: result.window
      });
    }
  }

  recordPolicyUtilization(policy, window, userTier, current, limit) {
    const utilization = (current / limit) * 100;
    this.metrics.policyUtilization.set(
      { policy, window, user_tier: userTier },
      utilization
    );
  }

  recordRedisOperation(operation, status) {
    this.metrics.redisOperations.inc({ operation, status });
  }

  sanitizeEndpoint(path) {
    // Remove specific IDs to group similar endpoints
    return path.replace(/\/\d+/g, '/:id')
               .replace(/\/[a-f0-9-]{36}/g, '/:uuid');
  }
}
```

### 6.2 Dashboard and Analytics

```javascript
class RateLimitDashboard {
  constructor(redis, metricsCollector) {
    this.redis = redis;
    this.metrics = metricsCollector;
  }

  async getDashboardData(timeRange = '24h') {
    const [
      overview,
      topEndpoints,
      topPolicies,
      violations,
      utilization
    ] = await Promise.all([
      this.getOverviewStats(timeRange),
      this.getTopEndpoints(timeRange),
      this.getTopPolicies(timeRange),
      this.getViolations(timeRange),
      this.getPolicyUtilization()
    ]);

    return {
      overview,
      topEndpoints,
      topPolicies,
      violations,
      utilization,
      timestamp: new Date().toISOString()
    };
  }

  async getOverviewStats(timeRange) {
    const window = this.getTimeWindow(timeRange);
    const key = `rate_limit:stats:${window}`;
    
    const stats = await this.redis.hgetall(key);
    
    return {
      totalRequests: parseInt(stats.total_requests) || 0,
      blockedRequests: parseInt(stats.blocked_requests) || 0,
      uniqueUsers: parseInt(stats.unique_users) || 0,
      uniqueIPs: parseInt(stats.unique_ips) || 0,
      averageLatency: parseFloat(stats.average_latency) || 0,
      blockRate: stats.total_requests > 0 ? 
        (stats.blocked_requests / stats.total_requests) * 100 : 0
    };
  }

  async getTopEndpoints(timeRange, limit = 10) {
    const window = this.getTimeWindow(timeRange);
    const key = `rate_limit:top_endpoints:${window}`;
    
    const endpoints = await this.redis.zrevrange(key, 0, limit - 1, 'WITHSCORES');
    
    return this.formatLeaderboard(endpoints);
  }

  async getTopPolicies(timeRange, limit = 10) {
    const window = this.getTimeWindow(timeRange);
    const key = `rate_limit:top_policies:${window}`;
    
    const policies = await this.redis.zrevrange(key, 0, limit - 1, 'WITHSCORES');
    
    return this.formatLeaderboard(policies);
  }

  async getViolations(timeRange, limit = 100) {
    const window = this.getTimeWindow(timeRange);
    const key = `rate_limit:violations:${window}`;
    
    const violations = await this.redis.lrange(key, 0, limit - 1);
    
    return violations.map(v => JSON.parse(v));
  }

  async getPolicyUtilization() {
    const policies = await this.redis.hgetall('rate_limit:policies');
    const utilization = [];
    
    for (const [policyId, policyData] of Object.entries(policies)) {
      const policy = JSON.parse(policyData);
      const stats = await this.getPolicyStats(policyId);
      
      utilization.push({
        policy: policyId,
        name: policy.name,
        enabled: policy.enabled,
        ...stats
      });
    }
    
    return utilization;
  }

  async getPolicyStats(policyId) {
    const key = `rate_limit:policy_stats:${policyId}`;
    const stats = await this.redis.hgetall(key);
    
    return {
      currentRequests: parseInt(stats.current_requests) || 0,
      totalRequests: parseInt(stats.total_requests) || 0,
      blockedRequests: parseInt(stats.blocked_requests) || 0,
      lastActivity: stats.last_activity || null
    };
  }

  formatLeaderboard(redisResult) {
    const leaderboard = [];
    for (let i = 0; i < redisResult.length; i += 2) {
      leaderboard.push({
        name: redisResult[i],
        count: parseInt(redisResult[i + 1])
      });
    }
    return leaderboard;
  }

  getTimeWindow(timeRange) {
    const windows = {
      '1h': 'hour',
      '24h': 'day',
      '7d': 'week',
      '30d': 'month'
    };
    
    return windows[timeRange] || 'day';
  }
}
```

## 7. Redis Data Structures

### 7.1 Data Organization

```redis
# Rate limiting counters (sliding window)
rate_limit:sw:policy_id:window:user:user_id:ip:hashed_ip
# ZSET with timestamp scores

# Rate limiting counters (fixed window)
rate_limit:fw:policy_id:window:user:user_id:ip:hashed_ip:timestamp
# STRING with counter value

# Policy storage
rate_limit:policies
# HASH with policy_id -> policy_json

# Statistics
rate_limit:stats:day
rate_limit:stats:hour
# HASH with various counters

# Top endpoints/policies
rate_limit:top_endpoints:day
rate_limit:top_policies:day
# ZSET with scores

# Violations log
rate_limit:violations:day
# LIST with violation records
```

### 7.2 Redis Operations Optimization

```javascript
class OptimizedRedisOperations {
  constructor(redis) {
    this.redis = redis;
    this.pipeline = null;
    this.batchSize = 100;
  }

  async executePipeline(operations) {
    const pipeline = this.redis.pipeline();
    
    operations.forEach(op => {
      pipeline[op.command](...op.args);
    });
    
    return pipeline.exec();
  }

  async cleanupExpiredKeys() {
    const stream = this.redis.scanStream({
      match: 'rate_limit:*',
      count: 100
    });
    
    let deletedCount = 0;
    const expiredKeys = [];
    
    stream.on('data', async (keys) => {
      for (const key of keys) {
        const ttl = await this.redis.ttl(key);
        if (ttl === -1) {
          expiredKeys.push(key);
        }
        
        if (expiredKeys.length >= this.batchSize) {
          await this.redis.del(...expiredKeys);
          deletedCount += expiredKeys.length;
          expiredKeys.length = 0;
        }
      }
    });
    
    stream.on('end', async () => {
      if (expiredKeys.length > 0) {
        await this.redis.del(...expiredKeys);
        deletedCount += expiredKeys.length;
      }
      
      console.log(`Cleaned up ${deletedCount} expired rate limit keys`);
    });
  }

  async updateStatistics(operations) {
    const pipeline = this.redis.pipeline();
    const now = new Date();
    const hourKey = `rate_limit:stats:${now.getHours()}`;
    const dayKey = `rate_limit:stats:${now.getDate()}`;
    
    operations.forEach(op => {
      pipeline.hincrby(hourKey, op.field, op.increment);
      pipeline.hincrby(dayKey, op.field, op.increment);
    });
    
    pipeline.expire(hourKey, 3600);
    pipeline.expire(dayKey, 86400);
    
    return pipeline.exec();
  }
}
```

## 8. Error Handling and Resilience

### 8.1 Circuit Breaker Pattern

```javascript
class CircuitBreaker {
  constructor(options = {}) {
    this.failureThreshold = options.failureThreshold || 5;
    this.timeout = options.timeout || 60000;
    this.monitoringPeriod = options.monitoringPeriod || 10000;
    
    this.state = 'CLOSED';
    this.failures = 0;
    this.lastFailureTime = null;
    this.nextAttempt = null;
  }

  async execute(fn) {
    if (this.state === 'OPEN') {
      if (Date.now() < this.nextAttempt) {
        throw new Error('Circuit breaker is OPEN');
      }
      this.state = 'HALF_OPEN';
    }

    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  onSuccess() {
    this.failures = 0;
    this.state = 'CLOSED';
    this.nextAttempt = null;
  }

  onFailure() {
    this.failures++;
    this.lastFailureTime = Date.now();
    
    if (this.failures >= this.failureThreshold) {
      this.state = 'OPEN';
      this.nextAttempt = Date.now() + this.timeout;
    }
  }

  getState() {
    return {
      state: this.state,
      failures: this.failures,
      nextAttempt: this.nextAttempt
    };
  }
}
```

### 8.2 Graceful Degradation

```javascript
class GracefulRateLimiter {
  constructor(redis, options = {}) {
    this.redis = redis;
    this.circuitBreaker = new CircuitBreaker(options.circuitBreaker);
    this.fallbackEnabled = options.fallbackEnabled !== false;
    this.memoryCache = new Map();
    this.memoryCacheSize = options.memoryCacheSize || 10000;
  }

  async checkRateLimit(request, identifiers) {
    try {
      return await this.circuitBreaker.execute(async () => {
        return await this.checkWithRedis(request, identifiers);
      });
    } catch (error) {
      console.warn('Redis rate limiting failed:', error.message);
      
      if (this.fallbackEnabled) {
        return await this.checkWithMemoryFallback(request, identifiers);
      }
      
      // Fail open - allow request
      return { allowed: true, fallback: true };
    }
  }

  async checkWithRedis(request, identifiers) {
    // Normal Redis-based rate limiting
    const rateLimiter = new RateLimiter(this.redis);
    return rateLimiter.checkRateLimit(request, identifiers);
  }

  async checkWithMemoryFallback(request, identifiers) {
    // Simple memory-based rate limiting as fallback
    const key = `${identifiers.userId || identifiers.ipAddress}:${request.path}`;
    const now = Date.now();
    const windowSize = 60000; // 1 minute
    const limit = 100; // Basic limit
    
    let entry = this.memoryCache.get(key);
    if (!entry) {
      entry = { requests: [], created: now };
      this.memoryCache.set(key, entry);
    }
    
    // Remove old requests
    entry.requests = entry.requests.filter(time => now - time < windowSize);
    
    if (entry.requests.length >= limit) {
      return {
        allowed: false,
        fallback: true,
        retryAfter: Math.ceil((entry.requests[0] + windowSize - now) / 1000)
      };
    }
    
    entry.requests.push(now);
    
    // Clean up memory cache if too large
    if (this.memoryCache.size > this.memoryCacheSize) {
      this.cleanupMemoryCache();
    }
    
    return {
      allowed: true,
      fallback: true,
      remaining: limit - entry.requests.length
    };
  }

  cleanupMemoryCache() {
    const now = Date.now();
    const maxAge = 300000; // 5 minutes
    
    for (const [key, entry] of this.memoryCache.entries()) {
      if (now - entry.created > maxAge) {
        this.memoryCache.delete(key);
      }
    }
  }
}
```

This comprehensive technical design specification provides a solid foundation for implementing a robust, scalable, and flexible API rate limiting system with Redis-based distributed coordination and comprehensive monitoring capabilities.