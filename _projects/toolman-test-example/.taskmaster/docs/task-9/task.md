# Task 9: Performance Optimization and Scaling

## Overview
Implement comprehensive performance optimizations and scaling strategies to support 1000+ concurrent users while maintaining sub-100ms message delivery latency. Focus on caching, database optimization, frontend performance, and horizontal scaling capabilities.

## Technical Implementation Guide

### Phase 1: Redis Caching Implementation

#### API Response Caching
```typescript
// backend/src/middleware/cache.ts
import { Request, Response, NextFunction } from 'express';
import redis from '../config/redis';
import crypto from 'crypto';

interface CacheOptions {
  duration: number; // seconds
  keyPrefix?: string;
  varyBy?: string[]; // headers or params to vary cache by
}

export const cacheMiddleware = (options: CacheOptions) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    // Skip caching for non-GET requests
    if (req.method !== 'GET') {
      return next();
    }

    // Generate cache key
    const cacheKey = generateCacheKey(req, options);

    try {
      // Check cache
      const cachedData = await redis.get(cacheKey);
      
      if (cachedData) {
        res.set('X-Cache', 'HIT');
        return res.json(JSON.parse(cachedData));
      }

      // Cache MISS
      res.set('X-Cache', 'MISS');

      // Store original json method
      const originalJson = res.json.bind(res);

      // Override json method to cache response
      res.json = (data: any) => {
        // Cache the response
        redis.setex(cacheKey, options.duration, JSON.stringify(data))
          .catch(err => console.error('Cache set error:', err));
        
        return originalJson(data);
      };

      next();
    } catch (error) {
      console.error('Cache middleware error:', error);
      next();
    }
  };
};

const generateCacheKey = (req: Request, options: CacheOptions): string => {
  const { keyPrefix = 'cache', varyBy = [] } = options;
  
  let keyParts = [keyPrefix, req.path];
  
  // Add query params
  const queryKeys = Object.keys(req.query).sort();
  if (queryKeys.length > 0) {
    const queryString = queryKeys
      .map(key => `${key}=${req.query[key]}`)
      .join('&');
    keyParts.push(crypto.createHash('md5').update(queryString).digest('hex'));
  }

  // Add vary by headers/params
  varyBy.forEach(header => {
    const value = req.headers[header.toLowerCase()] || req.params[header];
    if (value) {
      keyParts.push(value);
    }
  });

  return keyParts.join(':');
};

// Cache invalidation helper
export const invalidateCache = async (patterns: string[]) => {
  for (const pattern of patterns) {
    const keys = await redis.keys(pattern);
    if (keys.length > 0) {
      await redis.del(...keys);
    }
  }
};
```

#### Room and User Data Caching
```typescript
// backend/src/services/cacheService.ts
class CacheService {
  private readonly TTL = {
    ROOM_INFO: 300, // 5 minutes
    USER_PROFILE: 600, // 10 minutes
    ROOM_MEMBERS: 60, // 1 minute
    MESSAGE_COUNT: 30, // 30 seconds
  };

  async getRoomInfo(roomId: string): Promise<Room | null> {
    const cacheKey = `room:${roomId}`;
    
    // Try cache first
    const cached = await redis.get(cacheKey);
    if (cached) {
      return JSON.parse(cached);
    }

    // Load from database
    const room = await roomRepository.findById(roomId);
    if (room) {
      await redis.setex(cacheKey, this.TTL.ROOM_INFO, JSON.stringify(room));
    }

    return room;
  }

  async getUserProfile(userId: string): Promise<User | null> {
    const cacheKey = `user:${userId}`;
    
    const cached = await redis.get(cacheKey);
    if (cached) {
      return JSON.parse(cached);
    }

    const user = await userRepository.findById(userId);
    if (user) {
      await redis.setex(cacheKey, this.TTL.USER_PROFILE, JSON.stringify(user));
    }

    return user;
  }

  async getRoomMembers(roomId: string): Promise<string[]> {
    const cacheKey = `room_members:${roomId}`;
    
    const cached = await redis.get(cacheKey);
    if (cached) {
      return JSON.parse(cached);
    }

    const members = await roomUserRepository.getRoomMembers(roomId);
    await redis.setex(cacheKey, this.TTL.ROOM_MEMBERS, JSON.stringify(members));

    return members;
  }

  async invalidateRoom(roomId: string) {
    await redis.del(
      `room:${roomId}`,
      `room_members:${roomId}`,
      `message_count:${roomId}`
    );
  }

  async invalidateUser(userId: string) {
    await redis.del(`user:${userId}`);
  }
}

export const cacheService = new CacheService();
```

### Phase 2: Database Optimization

#### Query Optimization and Indexing
```typescript
// backend/src/repositories/optimizedMessageRepository.ts
export class OptimizedMessageRepository {
  // Optimized message query with proper indexing
  async findByRoomOptimized(
    roomId: string,
    options: {
      limit: number;
      cursor?: Date;
      direction: 'before' | 'after';
    }
  ) {
    const { limit, cursor, direction = 'before' } = options;

    // Use prepared statement for performance
    const query = `
      WITH ranked_messages AS (
        SELECT 
          m.*,
          u.username,
          u.avatar_url,
          COUNT(mr.user_id) as read_count,
          ROW_NUMBER() OVER (ORDER BY m.created_at ${direction === 'after' ? 'ASC' : 'DESC'}) as rn
        FROM messages m
        INNER JOIN users u ON m.user_id = u.id
        LEFT JOIN message_read_receipts mr ON m.id = mr.message_id
        WHERE m.room_id = $1
          AND m.deleted_at IS NULL
          ${cursor ? `AND m.created_at ${direction === 'before' ? '<' : '>'} $2` : ''}
        GROUP BY m.id, u.username, u.avatar_url
      )
      SELECT * FROM ranked_messages
      WHERE rn <= $${cursor ? '3' : '2'}
      ORDER BY created_at ${direction === 'after' ? 'ASC' : 'DESC'}
    `;

    const params = cursor 
      ? [roomId, cursor, limit]
      : [roomId, limit];

    const result = await pool.query(query, params);
    
    return result.rows.map(this.mapRowToMessage);
  }

  // Batch insert for read receipts
  async markMessagesAsReadBatch(userId: string, messageIds: string[]) {
    if (messageIds.length === 0) return;

    const values = messageIds.map((messageId, index) => 
      `($${index * 2 + 1}, $${index * 2 + 2}, NOW())`
    ).join(',');

    const params = messageIds.flatMap(messageId => [messageId, userId]);

    const query = `
      INSERT INTO message_read_receipts (message_id, user_id, read_at)
      VALUES ${values}
      ON CONFLICT (message_id, user_id) DO NOTHING
    `;

    await pool.query(query, params);
  }
}

// Database indexes for optimization
/*
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_messages_user_rooms ON messages(user_id, room_id);
CREATE INDEX idx_room_users_user_rooms ON room_users(user_id, room_id);
CREATE INDEX idx_read_receipts_message_user ON message_read_receipts(message_id, user_id);
*/
```

#### Connection Pool Optimization
```typescript
// backend/src/config/database.ts
import { Pool } from 'pg';
import { performance } from 'perf_hooks';

const pool = new Pool({
  host: process.env.DB_HOST,
  port: parseInt(process.env.DB_PORT || '5432'),
  database: process.env.DB_NAME,
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,
  
  // Optimized pool settings
  max: 20, // Maximum pool size
  min: 5,  // Minimum pool size
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
  
  // Statement timeout to prevent long-running queries
  statement_timeout: 10000, // 10 seconds
  
  // Query timeout
  query_timeout: 10000,
});

// Monitor query performance
if (process.env.NODE_ENV === 'development') {
  pool.on('connect', (client) => {
    client.query = new Proxy(client.query, {
      apply: async (target, thisArg, args) => {
        const start = performance.now();
        try {
          const result = await target.apply(thisArg, args);
          const duration = performance.now() - start;
          
          if (duration > 100) {
            console.warn(`Slow query (${duration.toFixed(2)}ms):`, args[0]);
          }
          
          return result;
        } catch (error) {
          const duration = performance.now() - start;
          console.error(`Query failed (${duration.toFixed(2)}ms):`, args[0], error);
          throw error;
        }
      }
    });
  });
}

export default pool;
```

### Phase 3: Socket.io Optimization

#### Optimized Socket.io Configuration
```typescript
// backend/src/socket/optimizedSocketServer.ts
import { Server as HTTPServer } from 'http';
import { Server } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { instrument } from '@socket.io/admin-ui';

export class OptimizedSocketServer {
  private io: Server;

  constructor(httpServer: HTTPServer) {
    this.io = new Server(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL,
        credentials: true
      },
      
      // Performance optimizations
      transports: ['websocket'], // Prefer WebSocket over polling
      perMessageDeflate: {
        threshold: 1024, // Compress messages > 1KB
        serverMaxWindowBits: 15,
        clientMaxWindowBits: 15,
        serverNoContextTakeover: true,
        clientNoContextTakeover: true
      },
      
      // Connection settings
      pingTimeout: 60000,
      pingInterval: 25000,
      upgradeTimeout: 10000,
      maxHttpBufferSize: 1e6, // 1MB
      
      // Limit concurrent connections per IP
      allowRequest: async (req, callback) => {
        const ip = req.headers['x-forwarded-for'] || req.connection.remoteAddress;
        const connectionCount = await this.getConnectionCountByIP(ip);
        
        if (connectionCount > 5) {
          callback('Too many connections', false);
        } else {
          callback(null, true);
        }
      }
    });

    // Enable admin UI in development
    if (process.env.NODE_ENV === 'development') {
      instrument(this.io, {
        auth: false,
        mode: 'development'
      });
    }
  }

  // Efficient room-based broadcasting
  broadcastToRoom(roomId: string, event: string, data: any, excludeSocketId?: string) {
    const room = this.io.to(`room:${roomId}`);
    
    if (excludeSocketId) {
      room.except(excludeSocketId).emit(event, data);
    } else {
      room.emit(event, data);
    }
  }

  // Batch events for better performance
  batchEmit(events: Array<{ room: string; event: string; data: any }>) {
    // Group events by room
    const eventsByRoom = events.reduce((acc, { room, event, data }) => {
      if (!acc[room]) acc[room] = [];
      acc[room].push({ event, data });
      return acc;
    }, {} as Record<string, Array<{ event: string; data: any }>>);

    // Emit batched events
    Object.entries(eventsByRoom).forEach(([room, roomEvents]) => {
      this.io.to(room).emit('batch-events', roomEvents);
    });
  }

  private async getConnectionCountByIP(ip: string): Promise<number> {
    const sockets = await this.io.fetchSockets();
    return sockets.filter(socket => 
      socket.handshake.address === ip
    ).length;
  }
}
```

### Phase 4: Frontend Performance Optimization

#### Virtual Scrolling for Messages
```typescript
// frontend/src/components/chat/VirtualMessageList.tsx
import React, { useCallback, useRef } from 'react';
import { VariableSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { Message } from '../../types';

interface VirtualMessageListProps {
  messages: Message[];
  currentUserId: string;
  onLoadMore: () => void;
}

export const VirtualMessageList: React.FC<VirtualMessageListProps> = ({
  messages,
  currentUserId,
  onLoadMore
}) => {
  const listRef = useRef<List>(null);
  const itemHeights = useRef<Record<number, number>>({});

  const getItemSize = useCallback((index: number) => {
    return itemHeights.current[index] || 80; // Estimated height
  }, []);

  const setItemSize = useCallback((index: number, size: number) => {
    itemHeights.current[index] = size;
    if (listRef.current) {
      listRef.current.resetAfterIndex(index);
    }
  }, []);

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const message = messages[index];
    const rowRef = useRef<HTMLDivElement>(null);

    React.useEffect(() => {
      if (rowRef.current) {
        const height = rowRef.current.getBoundingClientRect().height;
        setItemSize(index, height);
      }
    }, [index, message]);

    // Load more when near top
    if (index === 5) {
      onLoadMore();
    }

    return (
      <div style={style}>
        <div ref={rowRef}>
          <MessageBubble
            message={message}
            isOwn={message.userId === currentUserId}
          />
        </div>
      </div>
    );
  };

  return (
    <AutoSizer>
      {({ height, width }) => (
        <List
          ref={listRef}
          height={height}
          itemCount={messages.length}
          itemSize={getItemSize}
          width={width}
          overscanCount={5}
          initialScrollOffset={0}
          style={{ overflowX: 'hidden' }}
        >
          {Row}
        </List>
      )}
    </AutoSizer>
  );
};
```

#### Code Splitting and Lazy Loading
```typescript
// frontend/src/App.tsx
import React, { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';

// Lazy load heavy components
const ChatLayout = lazy(() => import('./components/chat/ChatLayout'));
const UserProfile = lazy(() => import('./components/profile/UserProfile'));
const AdminPanel = lazy(() => import('./components/admin/AdminPanel'));

// Loading component
const PageLoader = () => (
  <div className="flex items-center justify-center h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
  </div>
);

export const App: React.FC = () => {
  return (
    <Suspense fallback={<PageLoader />}>
      <Routes>
        <Route path="/chat/*" element={<ChatLayout />} />
        <Route path="/profile" element={<UserProfile />} />
        <Route path="/admin/*" element={<AdminPanel />} />
      </Routes>
    </Suspense>
  );
};
```

#### React Performance Optimizations
```typescript
// frontend/src/hooks/useOptimizedState.ts
import { useState, useCallback, useRef, useEffect } from 'react';
import { debounce, throttle } from 'lodash';

// Optimized state updates with batching
export const useBatchedState = <T>(initialState: T) => {
  const [state, setState] = useState(initialState);
  const pendingUpdates = useRef<Partial<T>>({});
  const timeoutRef = useRef<NodeJS.Timeout>();

  const batchUpdate = useCallback((updates: Partial<T>) => {
    pendingUpdates.current = { ...pendingUpdates.current, ...updates };

    if (timeoutRef.current) clearTimeout(timeoutRef.current);

    timeoutRef.current = setTimeout(() => {
      setState(prev => ({ ...prev, ...pendingUpdates.current }));
      pendingUpdates.current = {};
    }, 0);
  }, []);

  return [state, batchUpdate] as const;
};

// Memoized message list with proper dependencies
export const useOptimizedMessages = (roomId: string) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const messageCache = useRef<Map<string, Message[]>>(new Map());

  const loadMessages = useCallback(
    debounce(async (id: string) => {
      // Check cache first
      if (messageCache.current.has(id)) {
        setMessages(messageCache.current.get(id)!);
        return;
      }

      try {
        const response = await api.get(`/api/rooms/${id}/messages`);
        const data = response.data.data;
        
        messageCache.current.set(id, data);
        setMessages(data);
      } catch (error) {
        console.error('Failed to load messages:', error);
      }
    }, 300),
    []
  );

  useEffect(() => {
    loadMessages(roomId);
  }, [roomId, loadMessages]);

  const addMessage = useCallback((message: Message) => {
    setMessages(prev => [...prev, message]);
    
    // Update cache
    const cached = messageCache.current.get(roomId) || [];
    messageCache.current.set(roomId, [...cached, message]);
  }, [roomId]);

  return { messages, addMessage };
};
```

### Phase 5: Horizontal Scaling Configuration

#### Load Balancer Setup (Nginx)
```nginx
# /etc/nginx/sites-available/chat-app
upstream backend {
    least_conn;
    server backend1:3001 weight=1 max_fails=3 fail_timeout=30s;
    server backend2:3001 weight=1 max_fails=3 fail_timeout=30s;
    server backend3:3001 weight=1 max_fails=3 fail_timeout=30s;
    
    keepalive 32;
}

upstream socketio {
    ip_hash; # Sticky sessions for Socket.io
    server backend1:3001;
    server backend2:3001;
    server backend3:3001;
}

server {
    listen 80;
    server_name api.chatapp.com;

    # API routes
    location /api {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Caching
        proxy_cache_valid 200 302 1m;
        proxy_cache_valid 404 1m;
        proxy_cache_bypass $http_cache_control;
        add_header X-Cache-Status $upstream_cache_status;
    }

    # Socket.io
    location /socket.io {
        proxy_pass http://socketio;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Timeouts
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
    }
}
```

#### Docker Compose for Scaling
```yaml
# docker-compose.scale.yml
version: '3.8'

services:
  backend:
    image: chat-app/backend:latest
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
    environment:
      - NODE_ENV=production
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://user:pass@postgres:5432/chatdb
    depends_on:
      - redis
      - postgres

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - backend

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --maxmemory 512mb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: chatdb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    command: >
      postgres
      -c shared_buffers=256MB
      -c effective_cache_size=1GB
      -c maintenance_work_mem=64MB
      -c work_mem=4MB
      -c max_connections=200

volumes:
  redis_data:
  postgres_data:
```

## Performance Monitoring

```typescript
// backend/src/monitoring/performance.ts
import { performance } from 'perf_hooks';
import prometheus from 'prom-client';

// Prometheus metrics
const httpRequestDuration = new prometheus.Histogram({
  name: 'http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status_code']
});

const activeConnections = new prometheus.Gauge({
  name: 'websocket_active_connections',
  help: 'Number of active WebSocket connections'
});

const messageDeliveryLatency = new prometheus.Histogram({
  name: 'message_delivery_latency_ms',
  help: 'Message delivery latency in milliseconds',
  buckets: [10, 25, 50, 100, 250, 500, 1000]
});

// Register metrics
prometheus.register.registerMetric(httpRequestDuration);
prometheus.register.registerMetric(activeConnections);
prometheus.register.registerMetric(messageDeliveryLatency);

// Performance monitoring middleware
export const performanceMiddleware = (req: Request, res: Response, next: NextFunction) => {
  const start = performance.now();

  res.on('finish', () => {
    const duration = (performance.now() - start) / 1000;
    httpRequestDuration
      .labels(req.method, req.route?.path || req.path, res.statusCode.toString())
      .observe(duration);
  });

  next();
};
```

## Success Metrics

- API response time < 100ms (p95)
- Message delivery latency < 100ms
- Support 1000+ concurrent connections
- Memory usage < 1GB for 1000 users
- Cache hit rate > 80%
- Zero message loss under load
- Horizontal scaling verified
- Database query time < 50ms