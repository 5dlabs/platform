# Task 9: Performance Optimization and Scaling - AI Agent Prompt

You are an expert performance engineer tasked with optimizing the Task Master platform to support 1000+ concurrent users with sub-100ms message delivery latency. Your goal is to implement comprehensive performance optimizations across caching, real-time communication, database queries, and frontend rendering.

## Context

The Task Master platform is a real-time collaborative task management system that currently needs performance optimization to scale from its current capacity to support enterprise-level usage. The system uses Node.js/Express backend, Socket.io for real-time communication, PostgreSQL for data persistence, and React for the frontend.

## Primary Objectives

1. **Implement Multi-Layer Caching Strategy**
   - Set up Redis for API response caching with intelligent TTLs
   - Implement browser caching for static assets with proper cache headers
   - Create cache invalidation strategies for data consistency
   - Add ETag support for conditional requests

2. **Optimize Socket.io for Scale**
   - Configure optimal heartbeat intervals and timeouts
   - Implement room-based broadcasting to reduce unnecessary messages
   - Set up binary data transmission for file transfers
   - Add connection pooling and reuse strategies

3. **Enable Horizontal Scaling**
   - Design stateless backend architecture
   - Implement Redis adapter for Socket.io clustering
   - Configure load balancing with sticky sessions for WebSockets
   - Set up health checks and auto-scaling policies

4. **Database Performance Optimization**
   - Create strategic indexes for frequent query patterns
   - Optimize complex queries with proper joins and aggregations
   - Implement connection pooling with optimal pool sizes
   - Add query result caching for expensive operations

5. **Frontend Performance Enhancement**
   - Implement code splitting and lazy loading for routes
   - Add component memoization for expensive renders
   - Integrate virtual scrolling for long lists
   - Optimize bundle sizes and asset loading

## Technical Requirements

### Performance Targets
- **Concurrent Users**: Support 1000+ simultaneous connections
- **Message Latency**: Achieve < 100ms end-to-end delivery
- **API Response Time**: < 50ms for cached, < 200ms for uncached requests
- **Frontend Load Time**: < 3 seconds initial page load
- **Memory Usage**: < 512MB per backend instance
- **CPU Usage**: < 70% under normal load

### Technology Stack
- **Backend**: Node.js, Express, TypeScript
- **Real-time**: Socket.io with Redis adapter
- **Database**: PostgreSQL with read replicas
- **Caching**: Redis cluster
- **Frontend**: React with performance optimizations
- **Load Balancing**: Nginx with upstream configuration
- **Monitoring**: StatsD/Prometheus metrics

## Implementation Steps

### Step 1: Redis Infrastructure Setup
1. Install and configure Redis cluster with persistence enabled
2. Set up separate Redis instances for caching, sessions, and Socket.io
3. Implement connection pooling and retry logic
4. Create cache key naming conventions and TTL strategies

### Step 2: Caching Layer Implementation
1. Create cache middleware with configurable options
2. Implement cache warming for critical data
3. Add cache invalidation on data mutations
4. Set up cache monitoring and hit rate tracking

### Step 3: Socket.io Optimization
1. Configure optimal transport settings (WebSocket only)
2. Implement room-based message routing
3. Add message compression for large payloads
4. Set up presence tracking with minimal overhead

### Step 4: Database Optimization
1. Analyze query patterns with EXPLAIN ANALYZE
2. Create indexes for WHERE, JOIN, and ORDER BY clauses
3. Implement query result caching for expensive operations
4. Configure connection pooling with monitoring

### Step 5: Frontend Performance
1. Implement React.lazy() for route-based code splitting
2. Add React.memo() and useMemo() for expensive components
3. Integrate virtual scrolling library (react-window or similar)
4. Optimize webpack configuration for production builds

### Step 6: Horizontal Scaling Setup
1. Implement PM2 or cluster module for process management
2. Configure nginx for load balancing with health checks
3. Set up Redis adapter for Socket.io clustering
4. Test session persistence across instances

### Step 7: Monitoring and Alerting
1. Implement performance metrics collection
2. Create dashboards for real-time monitoring
3. Set up alerts for performance degradation
4. Add distributed tracing for request flow

## Code Examples

### Cache Middleware Implementation
```typescript
// Implement a flexible cache middleware that:
// - Supports varying cache keys by user/parameters
// - Handles ETags for conditional requests
// - Provides cache statistics
// - Supports graceful degradation
```

### Socket.io Room Optimization
```typescript
// Create an efficient room management system that:
// - Minimizes memory usage per connection
// - Supports volatile messages for non-critical updates
// - Implements presence tracking
// - Handles reconnection gracefully
```

### Database Query Optimization
```typescript
// Optimize queries by:
// - Using selective column retrieval
// - Implementing cursor-based pagination
// - Batching similar operations
// - Caching frequently accessed data
```

## Testing Requirements

1. **Load Testing**
   - Use K6 or Artillery for load testing
   - Simulate 1000+ concurrent users
   - Test various usage patterns (read-heavy, write-heavy, mixed)
   - Measure response times at different load levels

2. **Performance Benchmarks**
   - API endpoint response times
   - WebSocket message latency
   - Database query execution times
   - Frontend rendering performance

3. **Stress Testing**
   - Test system behavior at 150% target load
   - Identify breaking points and bottlenecks
   - Verify graceful degradation
   - Test recovery from failures

## Success Criteria

- [ ] System supports 1000+ concurrent users without degradation
- [ ] 95th percentile message latency < 100ms
- [ ] API response times meet specified targets
- [ ] Zero message loss under normal load
- [ ] Horizontal scaling works seamlessly
- [ ] Monitoring shows stable resource usage
- [ ] Load tests pass all performance thresholds

## Additional Considerations

1. **Security**: Ensure caching doesn't expose sensitive data
2. **Consistency**: Implement proper cache invalidation strategies
3. **Monitoring**: Set up comprehensive performance tracking
4. **Documentation**: Document all optimization decisions and configurations
5. **Rollback Plan**: Prepare rollback procedures for each optimization

Remember to test each optimization in isolation before combining them, and always measure the impact of each change. Focus on the bottlenecks that provide the most significant improvements first.