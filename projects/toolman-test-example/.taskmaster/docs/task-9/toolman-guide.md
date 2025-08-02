# Task 9: Performance Optimization and Scaling - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement comprehensive performance optimizations and scaling capabilities. The tools focus on researching best practices, implementing caching strategies, and configuring infrastructure for high performance.

## Core Tools

### 1. brave_web_search
**Purpose**: Research performance optimization patterns and scaling strategies
**When to use**: 
- Before implementing new optimization techniques
- When investigating performance bottlenecks
- For finding best practices and benchmarks
- To research monitoring and testing tools

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "Redis caching strategies Node.js 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "Socket.io Redis adapter horizontal scaling"
- "PostgreSQL performance tuning high concurrency"
- "k6 load testing WebSocket applications"
- "Node.js clustering vs PM2 performance"
- "React virtual scrolling large lists performance"

### 2. query_npm_registry
**Purpose**: Find performance optimization packages
**When to use**:
- Searching for caching libraries
- Finding monitoring tools
- Looking for optimization utilities
- Discovering profiling packages

**How to use**:
```json
{
  "tool": "query_npm_registry",
  "query": "redis cache",
  "limit": 10
}
```

**Essential packages to research**:
- Redis clients and adapters
- Performance monitoring tools
- Compression middleware
- Load testing utilities
- Memory profiling tools

### 3. get_programming_language_docs
**Purpose**: Access language-specific optimization guides
**When to use**:
- Understanding V8 optimization techniques
- Learning about memory management
- Reviewing async performance patterns
- Studying profiling methodologies

**How to use**:
```json
{
  "tool": "get_programming_language_docs",
  "language": "javascript",
  "topic": "performance optimization"
}
```

### 4. create_directory
**Purpose**: Organize performance-related code
**When to use**:
- Setting up monitoring structure
- Creating cache service directories
- Organizing load test scripts
- Structuring optimization utilities

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/monitoring"
}
```

**Directory structure**:
```
/backend/src/
├── cache/
│   ├── redisService.ts
│   ├── cacheMiddleware.ts
│   └── invalidation.ts
├── monitoring/
│   ├── metrics.ts
│   ├── healthChecks.ts
│   └── profiler.ts
├── optimization/
│   ├── compression.ts
│   ├── clustering.ts
│   └── queryOptimizer.ts
└── loadtests/
    ├── scenarios/
    └── utils/
```

### 5. write_file
**Purpose**: Create optimization implementation files
**When to use**:
- Writing caching services
- Creating monitoring endpoints
- Implementing performance utilities
- Setting up configuration files

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/cache/redisService.ts",
  "content": "// Redis caching service implementation"
}
```

### 6. edit_file
**Purpose**: Modify existing code for optimization
**When to use**:
- Adding caching to endpoints
- Implementing connection pooling
- Integrating monitoring
- Optimizing queries

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/backend/src/app.ts",
  "old_string": "app.use(express.json());",
  "new_string": "app.use(compression());\napp.use(express.json());"
}
```

## Implementation Flow

### Phase 1: Research & Planning (20 minutes)
1. **Performance patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Node.js application performance optimization checklist 2024"
   }
   ```

2. **Caching strategies**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Redis caching patterns microservices best practices"
   }
   ```

3. **Load testing tools**:
   ```json
   {
     "tool": "query_npm_registry",
     "query": "load testing websocket"
   }
   ```

### Phase 2: Redis Setup (25 minutes)
1. **Create cache structure**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/cache"
   }
   ```

2. **Write Redis service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/cache/redisService.ts",
     "content": "// Redis connection and caching logic"
   }
   ```

3. **Update dependencies**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/package.json",
     "old_string": "\"dependencies\": {",
     "new_string": "\"dependencies\": {\n    \"redis\": \"^4.0.0\",\n    \"@socket.io/redis-adapter\": \"^8.0.0\","
   }
   ```

### Phase 3: Performance Monitoring (20 minutes)
1. **Create monitoring directory**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/monitoring"
   }
   ```

2. **Implement metrics collection**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/monitoring/metrics.ts",
     "content": "// Prometheus metrics implementation"
   }
   ```

3. **Add health checks**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/monitoring/healthChecks.ts",
     "content": "// Health check endpoints"
   }
   ```

### Phase 4: Database Optimization (15 minutes)
1. **Read current schema**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/backend/migrations/001_initial_schema.sql"
   }
   ```

2. **Create optimization script**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/migrations/005_performance_indexes.sql",
     "content": "-- Performance optimization indexes"
   }
   ```

3. **Update connection pool**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/src/config/database.ts",
     "old_string": "pool: {",
     "new_string": "pool: {\n    min: 10,\n    max: 50,"
   }
   ```

### Phase 5: Frontend Optimization (20 minutes)
1. **Implement virtual scrolling**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/hooks/useVirtualScroll.ts",
     "content": "// Virtual scrolling hook implementation"
   }
   ```

2. **Add API caching**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/services/apiCache.ts",
     "content": "// LRU cache for API responses"
   }
   ```

3. **Enable code splitting**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/frontend/src/App.tsx",
     "old_string": "import ChatRoom from './components/ChatRoom';",
     "new_string": "const ChatRoom = lazy(() => import('./components/ChatRoom'));"
   }
   ```

## Best Practices

### Cache Key Design
```typescript
// Consistent key patterns
const cacheKey = {
  user: (id: string) => `cache:user:${id}`,
  room: (id: string) => `cache:room:${id}`,
  messages: (roomId: string, page: number) => `cache:messages:${roomId}:${page}`,
  session: (id: string) => `session:${id}`,
};

// TTL configuration
const ttl = {
  static: 3600,    // 1 hour
  user: 300,       // 5 minutes
  messages: 30,    // 30 seconds
  session: 86400,  // 24 hours
};
```

### Performance Testing
```javascript
// k6 test scenario
export const options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 500 },
    { duration: '10m', target: 1000 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],
    ws_message_delivery: ['p(95)<100'],
  },
};
```

### Monitoring Setup
```typescript
// Prometheus metrics
const metrics = {
  httpDuration: new Histogram({
    name: 'http_request_duration_seconds',
    help: 'HTTP request latency',
    labelNames: ['method', 'route', 'status'],
  }),
  wsConnections: new Gauge({
    name: 'websocket_active_connections',
    help: 'Active WebSocket connections',
  }),
  cacheHitRate: new Counter({
    name: 'cache_hit_rate',
    help: 'Cache hit rate',
    labelNames: ['cache_type'],
  }),
};
```

## Common Patterns

### Research → Implement → Test
```javascript
// 1. Research optimization
const best_practices = await brave_web_search("Redis caching strategies");

// 2. Implement based on findings
await write_file("cache/strategy.ts", optimizedImplementation);

// 3. Test performance
await write_file("loadtests/cache-test.js", performanceTest);
```

### Progressive Enhancement
```javascript
// 1. Basic optimization
await edit_file("app.ts", addCompression);

// 2. Add caching layer
await write_file("middleware/cache.ts", cacheMiddleware);

// 3. Implement advanced features
await write_file("cache/warming.ts", cacheWarmingStrategy);
```

## Performance Patterns

### Caching Layers
```typescript
// L1: In-memory cache (application)
const memoryCache = new LRUCache({ max: 1000 });

// L2: Redis cache (shared)
const redisCache = new RedisCache({ ttl: 300 });

// L3: Database query cache
const queryCache = new QueryCache({ duration: 60 });
```

### Load Distribution
```nginx
upstream backend {
    least_conn;
    server backend1:3000 weight=1;
    server backend2:3000 weight=1;
    server backend3:3000 weight=1;
    keepalive 32;
}
```

## Troubleshooting

### Issue: High cache miss rate
**Solution**: Review key patterns, increase TTL, implement cache warming

### Issue: Socket.io scaling issues
**Solution**: Ensure Redis adapter configured, check sticky sessions

### Issue: Database connection exhaustion
**Solution**: Increase pool size, optimize query patterns, add read replicas

### Issue: Memory leaks in Node.js
**Solution**: Profile with Chrome DevTools, check event listener cleanup

## Testing Strategies

### Load Testing Script
```javascript
// Create comprehensive test
await write_file("loadtests/full-scenario.js", `
import { check } from 'k6';
import ws from 'k6/ws';

export default function() {
  // Test implementation
}
`);
```

### Performance Profiling
```javascript
// Add profiling endpoint
await edit_file("app.ts",
  "// Routes",
  "// Profiling\nif (process.env.NODE_ENV === 'development') {\n  app.get('/profile', profiler.collect);\n}\n\n// Routes"
);
```

## Optimization Checklist

### Backend
- [ ] Redis connection pooling
- [ ] Database query optimization
- [ ] API response compression
- [ ] Static asset caching
- [ ] WebSocket message batching

### Frontend
- [ ] Code splitting implemented
- [ ] Virtual scrolling active
- [ ] Service worker caching
- [ ] Image lazy loading
- [ ] Bundle optimization

### Infrastructure
- [ ] Load balancer configured
- [ ] CDN integration
- [ ] Monitoring dashboards
- [ ] Auto-scaling policies
- [ ] Backup strategies

## Security Considerations

### Rate Limiting
```typescript
// Implement distributed rate limiting
const rateLimiter = new RedisRateLimiter({
  points: 100,
  duration: 60,
  blockDuration: 300,
});
```

### Session Security
```typescript
// Secure session configuration
const sessionConfig = {
  store: new RedisStore({ client: redis }),
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: true,
    httpOnly: true,
    sameSite: 'strict',
    maxAge: 86400000, // 24 hours
  },
};
```

## Task Completion

Execute optimizations systematically:
1. Research best practices thoroughly
2. Implement incrementally with testing
3. Monitor performance improvements
4. Document configuration changes
5. Validate against acceptance criteria

The task is complete when the application handles 1000+ concurrent users with consistent sub-100ms message delivery and meets all performance targets.