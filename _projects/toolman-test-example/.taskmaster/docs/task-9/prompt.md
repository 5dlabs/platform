# Autonomous Agent Prompt: Performance Optimization and Scaling

You are tasked with optimizing the chat application for high performance and implementing scaling strategies to support 1000+ concurrent users with sub-100ms message delivery latency.

## Objective
Implement comprehensive performance optimizations including caching strategies, database optimization, frontend performance improvements, and horizontal scaling capabilities to ensure the application can handle high loads efficiently.

## Detailed Requirements

### 1. Redis Caching Strategy
Implement multi-level caching:
- API response caching with smart invalidation
- Room information caching (5 min TTL)
- User profile caching (10 min TTL)
- Room membership caching (1 min TTL)
- Message count caching (30 sec TTL)
- Cache key generation with versioning

### 2. Cache Middleware
Create flexible caching middleware:
- Support for GET requests only
- Configurable TTL per endpoint
- Cache key variations (headers, params)
- Cache hit/miss headers
- Automatic cache invalidation
- Error handling fallback

### 3. Database Optimization
Optimize queries and connections:
- Add proper indexes for common queries
- Optimize message pagination queries
- Implement connection pooling
- Set query timeouts
- Monitor slow queries
- Batch operations where possible

Required indexes:
```sql
-- Message queries
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC);
-- User lookups
CREATE INDEX idx_users_email ON users(email);
-- Room membership
CREATE INDEX idx_room_users_lookup ON room_users(user_id, room_id);
```

### 4. Socket.io Optimization
Configure for performance:
- Use WebSocket transport only
- Enable message compression (perMessageDeflate)
- Optimize heartbeat intervals
- Implement connection limits per IP
- Use Redis adapter for scaling
- Batch event emissions

### 5. Frontend Performance
Implement React optimizations:
- Virtual scrolling for message lists
- Code splitting with lazy loading
- Component memoization
- Debounced/throttled updates
- Optimized re-renders
- Image lazy loading

### 6. Message List Virtualization
Use react-window for large lists:
- Variable height rows
- Overscan for smooth scrolling
- Dynamic height calculation
- Load more on scroll
- Maintain scroll position
- Keyboard navigation support

### 7. State Management Optimization
Efficient state updates:
- Batch state updates
- Use refs for non-render data
- Implement message caching
- Debounce expensive operations
- Memoize computed values
- Avoid unnecessary re-renders

### 8. Horizontal Scaling Setup
Configure for multiple instances:
- Stateless backend design
- Redis for shared state
- Sticky sessions for Socket.io
- Load balancer configuration
- Health check endpoints
- Graceful shutdown handling

### 9. Load Balancer Configuration
Nginx setup for scaling:
- Least connections algorithm
- WebSocket support
- Health checks
- Connection keepalive
- Request caching
- Compression

### 10. Performance Monitoring
Implement metrics collection:
- Request duration histograms
- Active connection gauges
- Message latency tracking
- Cache hit rates
- Memory usage monitoring
- CPU usage tracking

## Expected Deliverables

1. Cache middleware implementation
2. Cache service with invalidation
3. Optimized database queries
4. Database indexes SQL
5. Socket.io performance config
6. Virtual message list component
7. Code splitting setup
8. Load balancer configuration
9. Docker compose for scaling
10. Performance monitoring setup

## Performance Targets

### Response Times
- API endpoints: < 100ms (p95)
- Message delivery: < 100ms
- Room switching: < 200ms
- Initial page load: < 2s
- Time to interactive: < 3s

### Scalability
- Concurrent users: 1000+
- Messages per second: 1000+
- Active rooms: 100+
- Memory per user: < 1MB
- CPU usage: < 70% at peak

### Cache Performance
- Hit rate: > 80%
- Invalidation: < 10ms
- Redis operations: < 5ms
- Cache warm-up: < 30s

## Implementation Checklist

### Backend Optimizations
- [ ] Redis caching middleware
- [ ] Cache service implementation
- [ ] Database query optimization
- [ ] Connection pool tuning
- [ ] Socket.io compression
- [ ] Batch operations
- [ ] Rate limiting

### Frontend Optimizations
- [ ] Virtual scrolling
- [ ] Code splitting
- [ ] Lazy loading
- [ ] Memoization
- [ ] Debouncing
- [ ] Asset optimization

### Infrastructure
- [ ] Load balancer setup
- [ ] Horizontal scaling config
- [ ] Redis cluster mode
- [ ] Database read replicas
- [ ] CDN integration
- [ ] Monitoring setup

## Testing Requirements

### Load Testing
Use tools like Artillery or K6:
- Test 1000 concurrent users
- Measure message latency
- Monitor resource usage
- Test failover scenarios
- Verify no message loss

### Performance Testing
- Lighthouse scores > 90
- Core Web Vitals pass
- Memory leak detection
- Bundle size analysis
- Runtime performance profiling

## Monitoring and Metrics

Implement dashboards for:
- Request latency (p50, p95, p99)
- Active connections
- Cache hit rates
- Database query times
- Memory usage
- CPU utilization
- Error rates
- Message throughput

## Common Bottlenecks to Address

- N+1 database queries
- Unbounded message lists
- Large bundle sizes
- Synchronous operations
- Memory leaks
- Inefficient algorithms
- Missing indexes
- Cold cache starts

Begin by implementing the caching layer, then optimize database queries, followed by frontend performance improvements and finally horizontal scaling configuration.