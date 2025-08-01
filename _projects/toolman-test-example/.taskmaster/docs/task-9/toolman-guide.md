# Toolman Guide for Task 9: Performance Optimization and Scaling

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 9, which focuses on optimizing the chat application for high performance and scalability to support 1000+ concurrent users through caching strategies, database optimization, and horizontal scaling.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create performance optimization directory structure

**When to Use**: 
- At the beginning to organize performance-related code
- When creating caching middleware
- For organizing monitoring and metrics

**How to Use**:
```
# Create performance optimization structure
create_directory /chat-application/backend/src/cache
create_directory /chat-application/backend/src/cache/middleware
create_directory /chat-application/backend/src/cache/strategies
create_directory /chat-application/backend/src/performance
create_directory /chat-application/backend/src/performance/monitoring
create_directory /chat-application/frontend/src/optimization
create_directory /chat-application/frontend/src/optimization/hooks
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create caching middleware, optimization utilities, and performance configurations

**When to Use**: 
- To implement Redis caching middleware
- To create database optimization scripts
- To implement performance monitoring
- To create load balancing configurations

**How to Use**:
```
# Create Redis cache middleware
write_file /chat-application/backend/src/cache/middleware/cacheMiddleware.ts <cache-middleware>

# Create cache strategies
write_file /chat-application/backend/src/cache/strategies/apiCache.ts <api-cache>

# Create database indexes
write_file /chat-application/backend/src/database/migrations/004_add_performance_indexes.sql <indexes>

# Create performance monitor
write_file /chat-application/backend/src/performance/monitoring/metrics.ts <metrics>

# Create React optimization hooks
write_file /chat-application/frontend/src/optimization/hooks/useVirtualScroll.ts <virtual-scroll>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing code for optimization opportunities

**When to Use**: 
- To analyze current API endpoints
- To review Socket.io configuration
- To check database queries
- To examine React components

**How to Use**:
```
# Review API controllers
read_file /chat-application/backend/src/api/controllers/roomController.ts

# Check Socket.io setup
read_file /chat-application/backend/src/socket/socketServer.ts

# Review message queries
read_file /chat-application/backend/src/api/controllers/messageController.ts

# Check React components
read_file /chat-application/frontend/src/components/chat/messages/MessageList.tsx
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update existing files with performance optimizations

**When to Use**: 
- To add caching to API endpoints
- To optimize database queries
- To implement code splitting
- To add performance dependencies

**How to Use**:
```
# Add performance dependencies
edit_file /chat-application/backend/package.json
# Add: compression, helmet, cluster module

# Update Socket.io configuration
edit_file /chat-application/backend/src/socket/socketServer.ts
# Add optimization settings

# Add frontend optimization dependencies
edit_file /chat-application/frontend/package.json
# Add: react-window, @loadable/component

# Update webpack config for code splitting
edit_file /chat-application/frontend/webpack.config.js
# Add optimization settings
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify optimization structure creation

**When to Use**: 
- After creating cache directories
- To confirm file organization
- Before deployment testing

**How to Use**:
```
# Verify cache structure
list_directory /chat-application/backend/src/cache

# Check performance directory
list_directory /chat-application/backend/src/performance
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Caching Strategy Phase**
   - Use `write_file` to create Redis cache middleware
   - Implement cache key generation strategies
   - Add TTL configurations for different data types
   - Create cache invalidation logic

2. **Database Optimization Phase**
   - Use `write_file` to create index migration:
     ```sql
     -- Performance indexes
     CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC);
     CREATE INDEX idx_room_users_user ON room_users(user_id);
     CREATE INDEX idx_messages_user ON messages(user_id);
     ```
   - Optimize query patterns with pagination
   - Implement connection pooling configuration

3. **Socket.io Optimization Phase**
   - Use `edit_file` to update Socket.io configuration:
     - Adjust heartbeat intervals
     - Enable binary transmission
     - Configure proper transports
     - Implement room-based broadcasting

4. **Horizontal Scaling Setup**
   - Use `write_file` to create cluster configuration
   - Implement sticky sessions for Socket.io
   - Configure Redis adapter properly
   - Create health check endpoints

5. **Frontend Performance Phase**
   - Implement React.lazy for code splitting
   - Add virtual scrolling for message lists
   - Implement memoization strategies
   - Configure service workers for caching

6. **Monitoring Implementation**
   - Create performance metrics collection
   - Implement request timing middleware
   - Add memory usage monitoring
   - Create performance dashboards

## Best Practices

1. **Caching**: Cache at multiple levels (Redis, CDN, browser)
2. **Database**: Use read replicas for scaling reads
3. **Socket.io**: Minimize payload sizes
4. **Frontend**: Implement progressive loading
5. **Monitoring**: Track key performance metrics
6. **Testing**: Regular load testing with realistic scenarios

## Task-Specific Implementation Details

### Redis Cache Middleware Pattern
```typescript
// cacheMiddleware.ts
import { Request, Response, NextFunction } from 'express';
import { redisClient } from '../../database/config/redis.config';

export const cacheMiddleware = (keyPrefix: string, ttl: number = 300) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    const key = `${keyPrefix}:${req.originalUrl}`;
    
    try {
      const cached = await redisClient.get(key);
      if (cached) {
        return res.json(JSON.parse(cached));
      }
    } catch (error) {
      console.error('Cache error:', error);
    }

    // Store original send
    const originalSend = res.json;
    res.json = function(data) {
      // Cache the response
      redisClient.setex(key, ttl, JSON.stringify(data));
      return originalSend.call(this, data);
    };

    next();
  };
};
```

### Socket.io Optimization Pattern
```typescript
// socketServer.ts optimization
export const initializeSocket = (httpServer) => {
  const io = new Server(httpServer, {
    cors: corsOptions,
    // Performance optimizations
    pingTimeout: 60000,
    pingInterval: 25000,
    transports: ['websocket', 'polling'],
    allowUpgrades: true,
    perMessageDeflate: {
      threshold: 1024 // Compress if > 1kb
    }
  });

  // Enable binary transmission
  io.on('connection', (socket) => {
    socket.on('send-message', async (data, callback) => {
      // Use rooms for efficient broadcasting
      socket.to(data.roomId).emit('new-message', data);
    });
  });
};
```

### Virtual Scrolling Pattern
```typescript
// useVirtualScroll.ts
import { FixedSizeList } from 'react-window';

export const MessageList: React.FC<{ messages: Message[] }> = ({ messages }) => {
  const Row = ({ index, style }) => (
    <div style={style}>
      <MessageItem message={messages[index]} />
    </div>
  );

  return (
    <FixedSizeList
      height={600}
      itemCount={messages.length}
      itemSize={80}
      width="100%"
    >
      {Row}
    </FixedSizeList>
  );
};
```

### Database Query Optimization Pattern
```typescript
// Optimized pagination query
export const getMessages = async (roomId: string, cursor?: string, limit = 50) => {
  const query = `
    SELECT * FROM messages 
    WHERE room_id = $1 
    ${cursor ? 'AND created_at < $2' : ''}
    ORDER BY created_at DESC 
    LIMIT $${cursor ? '3' : '2'}
  `;
  
  const params = cursor ? [roomId, cursor, limit] : [roomId, limit];
  return db.query(query, params);
};
```

### Cluster Configuration Pattern
```typescript
// cluster.ts
import cluster from 'cluster';
import os from 'os';

if (cluster.isMaster) {
  const numCPUs = os.cpus().length;
  
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }
  
  cluster.on('exit', (worker) => {
    console.log(`Worker ${worker.process.pid} died`);
    cluster.fork(); // Restart worker
  });
} else {
  // Worker process - start server
  import('./server');
}
```

## Troubleshooting

- **Cache Invalidation**: Implement proper cache busting strategies
- **Memory Leaks**: Monitor memory usage in production
- **Connection Limits**: Configure database pool sizes appropriately
- **Load Balancing**: Ensure sticky sessions for Socket.io
- **Performance Regression**: Set up automated performance tests

## Testing Approach

1. **Load Tests**:
   - Test with 1000+ concurrent users
   - Measure message delivery latency
   - Monitor resource usage

2. **Cache Tests**:
   - Verify cache hit rates
   - Test cache invalidation
   - Measure response time improvements

3. **Scaling Tests**:
   - Test horizontal scaling
   - Verify session persistence
   - Test failover scenarios