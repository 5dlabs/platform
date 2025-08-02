# Task 9: Performance Optimization and Scaling - AI Agent Prompt

You are a senior performance engineer tasked with optimizing a chat application to support 1000+ concurrent users with sub-100ms message delivery. Your implementation must include comprehensive caching strategies, real-time communication optimization, horizontal scaling setup, and both backend and frontend performance improvements.

## Primary Objectives

1. **Caching Infrastructure**: Implement Redis caching for API responses, session management, and real-time event distribution.

2. **Socket.io Optimization**: Configure Socket.io for high performance with Redis adapter, proper heartbeat intervals, and room-based broadcasting.

3. **Database Optimization**: Create proper indexes, implement connection pooling, and optimize queries for high concurrency.

4. **Horizontal Scaling**: Set up stateless backend architecture with load balancing and session sharing across instances.

5. **Frontend Performance**: Implement virtual scrolling, code splitting, memoization, and API response caching.

## Required Actions

### Phase 1: Redis Setup (20 minutes)
1. Install dependencies:
   ```bash
   npm install redis @socket.io/redis-adapter ioredis
   npm install lru-cache prom-client compression
   npm install -D @types/redis @types/compression
   ```

2. Configure Redis connection:
   - Connection pooling
   - Retry strategies
   - Error handling
   - Pub/Sub for events

3. Implement caching service:
   - Get/Set operations
   - TTL management
   - Pattern invalidation
   - Cache warming

### Phase 2: Caching Middleware (15 minutes)
1. Create cache middleware:
   - Cache key generation
   - Response caching
   - Conditional requests
   - Cache headers

2. Implement invalidation:
   - Pattern-based clearing
   - Event-driven updates
   - Cascade invalidation

3. Configure cache rules:
   ```typescript
   // Static data: 1 hour
   // User data: 5 minutes
   // Room data: 10 minutes
   // Messages: 30 seconds
   ```

### Phase 3: Socket.io Optimization (20 minutes)
1. Redis adapter setup:
   - Pub/Sub clients
   - Sticky sessions
   - Cross-server events

2. Performance tuning:
   ```typescript
   {
     pingInterval: 25000,
     pingTimeout: 60000,
     maxHttpBufferSize: 1e6,
     perMessageDeflate: true
   }
   ```

3. Room optimization:
   - Efficient broadcasting
   - Binary transmission
   - Message batching

### Phase 4: Database Optimization (15 minutes)
1. Connection pool config:
   ```typescript
   {
     min: 10,
     max: 50,
     idleTimeoutMillis: 30000
   }
   ```

2. Create indexes:
   - messages: room_id, created_at
   - users: email, username, online_status
   - rooms: created_at, is_private
   - room_members: user_id, room_id

3. Query optimization:
   - Cursor pagination
   - Batch operations
   - Prepared statements

### Phase 5: Frontend Optimization (20 minutes)
1. Virtual scrolling:
   - Message list virtualization
   - Dynamic item heights
   - Smooth scrolling

2. Code splitting:
   ```typescript
   const ChatRoom = lazy(() => import('./ChatRoom'));
   const UserProfile = lazy(() => import('./UserProfile'));
   ```

3. Memoization:
   - React.memo components
   - useMemo for calculations
   - useCallback for handlers

4. API caching:
   - LRU cache
   - Conditional requests
   - Stale-while-revalidate

### Phase 6: Load Balancing (10 minutes)
1. NGINX configuration:
   - Least connections
   - Health checks
   - WebSocket support
   - Compression

2. Session management:
   - Redis session store
   - Cookie configuration
   - CORS headers

## Implementation Details

### Redis Caching Strategy
```typescript
// Cache keys structure
cache:api:/users/:id -> User data (TTL: 5m)
cache:api:/rooms/:id -> Room data (TTL: 10m)
cache:api:/messages/:roomId -> Messages (TTL: 30s)
cache:session:{sessionId} -> Session (TTL: 24h)

// Invalidation patterns
cache:api:/rooms/* -> When room updated
cache:api:/messages/:roomId -> When new message
cache:api:/users/:id -> When user updated
```

### Performance Metrics
```typescript
// Track these metrics
- HTTP request duration
- WebSocket connections
- Cache hit/miss ratio
- Message delivery time
- Database query time
- Memory usage
- CPU utilization
```

### Horizontal Scaling Architecture
```
Load Balancer (NGINX)
    ├── Server 1 (Node.js + Socket.io)
    ├── Server 2 (Node.js + Socket.io)
    └── Server N (Node.js + Socket.io)
         ↓
    Redis Cluster (Cache + Pub/Sub)
         ↓
    PostgreSQL (Primary + Replicas)
```

### Testing Requirements
1. Load testing with k6:
   - Ramp to 1000 users
   - Measure latencies
   - Check error rates
   - Monitor resources

2. Performance targets:
   - Message delivery < 100ms
   - API response < 500ms (p95)
   - Initial load < 3s
   - 99.9% uptime

## Error Handling

### Cache Failures
```typescript
try {
  const cached = await redis.get(key);
  if (cached) return JSON.parse(cached);
} catch (error) {
  // Log but don't fail
  console.error('Cache error:', error);
  // Continue to database
}
```

### Connection Issues
- Implement circuit breakers
- Graceful degradation
- Automatic reconnection
- Fallback strategies

## Security Considerations

### Rate Limiting
```typescript
const rateLimiter = new RateLimiter({
  windowMs: 60000, // 1 minute
  max: 100, // requests
  keyGenerator: (req) => req.userId || req.ip
});
```

### Session Security
- Secure cookies
- HttpOnly flag
- SameSite attribute
- Session rotation

## Monitoring Setup

### Prometheus Metrics
```typescript
// Key metrics to expose
http_request_duration_seconds
websocket_active_connections
cache_hits_total
cache_misses_total
message_delivery_time_ms
database_query_duration_seconds
```

### Health Checks
```typescript
GET /health -> Overall health
GET /health/db -> Database status
GET /health/redis -> Redis status
GET /health/ready -> Readiness probe
```

## Performance Best Practices

### Backend
1. Use streaming for large responses
2. Implement request coalescing
3. Enable HTTP/2
4. Use connection keep-alive
5. Implement graceful shutdown

### Frontend
1. Lazy load images
2. Debounce user inputs
3. Use Web Workers for heavy tasks
4. Implement offline support
5. Optimize bundle size

### Database
1. Use read replicas
2. Implement query caching
3. Batch similar operations
4. Use connection pooling
5. Regular VACUUM/ANALYZE

## Final Optimization Checklist

Before marking complete:
- [ ] Redis caching operational
- [ ] Socket.io with Redis adapter
- [ ] Database indexes created
- [ ] Connection pooling configured
- [ ] Virtual scrolling implemented
- [ ] Code splitting active
- [ ] API caching working
- [ ] Load balancer configured
- [ ] Metrics collection setup
- [ ] Load testing passed

Execute this optimization systematically, ensuring each component is properly configured and tested before moving to the next. The application must handle 1000+ concurrent users with consistent sub-100ms message delivery.