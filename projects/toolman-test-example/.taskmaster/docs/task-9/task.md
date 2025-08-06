# Task 9: Performance Optimization and Scaling

## Overview
Optimize the chat application for high performance and implement scaling strategies to support 1000+ concurrent users. This task focuses on implementing caching mechanisms, optimizing real-time communication, setting up horizontal scaling infrastructure, and fine-tuning both backend and frontend performance for optimal user experience under heavy load.

## Technical Architecture

### Performance Stack
- **Caching Layer**: Redis for API responses and session data
- **Message Queue**: Redis Pub/Sub for real-time events
- **Load Balancer**: NGINX or HAProxy for distribution
- **Monitoring**: Prometheus + Grafana for metrics
- **CDN**: CloudFront for static asset delivery

### Scaling Architecture
```
                    ┌─────────────┐
                    │Load Balancer│
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐        ┌────▼────┐       ┌────▼────┐
   │Server 1 │        │Server 2 │       │Server N │
   └────┬────┘        └────┬────┘       └────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    ┌──────▼──────┐
                    │Redis Cluster│
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  PostgreSQL │
                    └─────────────┘
```

## Implementation Details

### 1. Redis Caching Implementation

```typescript
// backend/src/config/redis.ts
import { createClient, RedisClientType } from 'redis';
import { promisify } from 'util';

export class RedisService {
  private client: RedisClientType;
  private isConnected: boolean = false;

  constructor() {
    this.client = createClient({
      url: process.env.REDIS_URL || 'redis://localhost:6379',
      socket: {
        reconnectStrategy: (retries) => {
          if (retries > 10) {
            console.error('Redis connection failed after 10 retries');
            return new Error('Redis connection failed');
          }
          return Math.min(retries * 100, 3000);
        },
      },
    });

    this.client.on('error', (err) => {
      console.error('Redis error:', err);
      this.isConnected = false;
    });

    this.client.on('connect', () => {
      console.log('Redis connected');
      this.isConnected = true;
    });
  }

  async connect(): Promise<void> {
    if (!this.isConnected) {
      await this.client.connect();
    }
  }

  async disconnect(): Promise<void> {
    if (this.isConnected) {
      await this.client.quit();
      this.isConnected = false;
    }
  }

  // Cache methods
  async get(key: string): Promise<string | null> {
    try {
      return await this.client.get(key);
    } catch (error) {
      console.error('Redis get error:', error);
      return null;
    }
  }

  async set(key: string, value: string, ttlSeconds?: number): Promise<void> {
    try {
      if (ttlSeconds) {
        await this.client.set(key, value, { EX: ttlSeconds });
      } else {
        await this.client.set(key, value);
      }
    } catch (error) {
      console.error('Redis set error:', error);
    }
  }

  async del(key: string): Promise<void> {
    try {
      await this.client.del(key);
    } catch (error) {
      console.error('Redis delete error:', error);
    }
  }

  async invalidatePattern(pattern: string): Promise<void> {
    try {
      const keys = await this.client.keys(pattern);
      if (keys.length > 0) {
        await this.client.del(keys);
      }
    } catch (error) {
      console.error('Redis pattern delete error:', error);
    }
  }

  // Pub/Sub methods for Socket.io adapter
  async publish(channel: string, message: string): Promise<void> {
    await this.client.publish(channel, message);
  }

  async subscribe(channel: string, callback: (message: string) => void): Promise<void> {
    const subscriber = this.client.duplicate();
    await subscriber.connect();
    await subscriber.subscribe(channel, callback);
  }
}

export const redisService = new RedisService();
```

### 2. Caching Middleware

```typescript
// backend/src/middleware/cache.ts
import { Request, Response, NextFunction } from 'express';
import { redisService } from '../config/redis';
import crypto from 'crypto';

interface CacheOptions {
  duration: number; // seconds
  keyGenerator?: (req: Request) => string;
  invalidatePattern?: string;
}

export const cacheMiddleware = (options: CacheOptions) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    // Skip caching for non-GET requests
    if (req.method !== 'GET') {
      return next();
    }

    // Generate cache key
    const key = options.keyGenerator 
      ? options.keyGenerator(req)
      : `cache:${req.originalUrl}:${crypto.createHash('md5').update(JSON.stringify(req.query)).digest('hex')}`;

    try {
      // Check cache
      const cachedData = await redisService.get(key);
      
      if (cachedData) {
        res.setHeader('X-Cache', 'HIT');
        res.setHeader('Cache-Control', `private, max-age=${options.duration}`);
        return res.json(JSON.parse(cachedData));
      }

      // Cache miss
      res.setHeader('X-Cache', 'MISS');
      
      // Override res.json to cache the response
      const originalJson = res.json.bind(res);
      res.json = function(data: any) {
        // Cache the response
        redisService.set(key, JSON.stringify(data), options.duration)
          .catch(err => console.error('Cache set error:', err));
        
        // Invalidate related cache if pattern provided
        if (options.invalidatePattern) {
          redisService.invalidatePattern(options.invalidatePattern)
            .catch(err => console.error('Cache invalidation error:', err));
        }
        
        return originalJson(data);
      };
      
      next();
    } catch (error) {
      console.error('Cache middleware error:', error);
      next();
    }
  };
};

// Invalidation middleware for mutations
export const invalidateCache = (patterns: string[]) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    const originalJson = res.json.bind(res);
    
    res.json = function(data: any) {
      // Invalidate cache patterns after successful response
      if (res.statusCode < 400) {
        Promise.all(
          patterns.map(pattern => 
            redisService.invalidatePattern(pattern)
          )
        ).catch(err => console.error('Cache invalidation error:', err));
      }
      
      return originalJson(data);
    };
    
    next();
  };
};
```

### 3. Socket.io Optimization with Redis Adapter

```typescript
// backend/src/config/socketio.ts
import { Server as HttpServer } from 'http';
import { Server as SocketServer } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { createClient } from 'redis';
import { instrument } from '@socket.io/admin-ui';

export class SocketService {
  private io: SocketServer;
  private pubClient: any;
  private subClient: any;

  constructor(httpServer: HttpServer) {
    // Configure Socket.io with optimizations
    this.io = new SocketServer(httpServer, {
      cors: {
        origin: process.env.CLIENT_URL,
        credentials: true,
      },
      // Performance optimizations
      pingInterval: 25000, // 25 seconds
      pingTimeout: 60000, // 60 seconds
      upgradeTimeout: 30000, // 30 seconds
      maxHttpBufferSize: 1e6, // 1MB
      // Enable binary transmission
      perMessageDeflate: {
        threshold: 1024, // Compress if message > 1KB
      },
      // Connection state recovery
      connectionStateRecovery: {
        maxDisconnectionDuration: 2 * 60 * 1000, // 2 minutes
        skipMiddlewares: true,
      },
    });

    // Set up Redis adapter for horizontal scaling
    this.setupRedisAdapter();

    // Enable admin UI for monitoring
    if (process.env.NODE_ENV === 'development') {
      instrument(this.io, {
        auth: false,
        mode: 'development',
      });
    }
  }

  private async setupRedisAdapter() {
    this.pubClient = createClient({ 
      url: process.env.REDIS_URL,
      socket: {
        reconnectStrategy: (retries) => Math.min(retries * 100, 3000),
      },
    });
    
    this.subClient = this.pubClient.duplicate();

    await Promise.all([
      this.pubClient.connect(),
      this.subClient.connect(),
    ]);

    this.io.adapter(createAdapter(this.pubClient, this.subClient));
    
    console.log('Socket.io Redis adapter configured');
  }

  // Optimized room-based broadcasting
  broadcastToRoom(roomId: string, event: string, data: any, excludeSocketId?: string) {
    const room = this.io.to(roomId);
    
    if (excludeSocketId) {
      room.except(excludeSocketId).emit(event, data);
    } else {
      room.emit(event, data);
    }
  }

  // Binary data transmission for files
  sendBinaryData(socketId: string, event: string, buffer: Buffer, metadata: any) {
    this.io.to(socketId).emit(event, {
      metadata,
      data: buffer,
    });
  }

  // Get room statistics
  async getRoomStats(roomId: string) {
    const sockets = await this.io.in(roomId).fetchSockets();
    return {
      connectedUsers: sockets.length,
      socketIds: sockets.map(s => s.id),
    };
  }

  getIO(): SocketServer {
    return this.io;
  }
}
```

### 4. Database Optimization

```typescript
// backend/src/config/database.ts
import { Pool, PoolConfig } from 'pg';
import { Knex, knex } from 'knex';

// Optimized connection pool configuration
const poolConfig: PoolConfig = {
  connectionString: process.env.DATABASE_URL,
  // Connection pool settings for high concurrency
  min: 10, // Minimum connections
  max: 50, // Maximum connections
  idleTimeoutMillis: 30000, // 30 seconds
  connectionTimeoutMillis: 5000, // 5 seconds
  // Enable statement timeout
  statement_timeout: 30000, // 30 seconds
  // Enable query timeout
  query_timeout: 30000, // 30 seconds
};

// Create optimized Knex instance
export const db: Knex = knex({
  client: 'pg',
  connection: poolConfig,
  pool: {
    min: 10,
    max: 50,
    createTimeoutMillis: 5000,
    acquireTimeoutMillis: 30000,
    idleTimeoutMillis: 30000,
    reapIntervalMillis: 1000,
    createRetryIntervalMillis: 100,
  },
  // Enable query logging in development
  debug: process.env.NODE_ENV === 'development',
  // Acquire connection timeout
  acquireConnectionTimeout: 60000,
});

// Database indexes for performance
export async function createOptimizedIndexes() {
  // Messages table indexes
  await db.schema.alterTable('messages', (table) => {
    table.index(['room_id', 'created_at'], 'idx_messages_room_created');
    table.index(['user_id', 'created_at'], 'idx_messages_user_created');
    table.index('created_at', 'idx_messages_created_at');
  });

  // Users table indexes
  await db.schema.alterTable('users', (table) => {
    table.index('email', 'idx_users_email');
    table.index('username', 'idx_users_username');
    table.index(['is_online', 'last_seen'], 'idx_users_online_status');
  });

  // Rooms table indexes
  await db.schema.alterTable('rooms', (table) => {
    table.index('created_at', 'idx_rooms_created_at');
    table.index(['is_private', 'created_at'], 'idx_rooms_private_created');
  });

  // Room members table indexes
  await db.schema.alterTable('room_members', (table) => {
    table.index(['user_id', 'joined_at'], 'idx_room_members_user_joined');
    table.index(['room_id', 'role'], 'idx_room_members_room_role');
  });

  console.log('Database indexes created');
}

// Query optimization helpers
export class QueryOptimizer {
  // Paginated query with cursor
  static async paginateWithCursor<T>(
    query: Knex.QueryBuilder,
    cursor: string | null,
    limit: number,
    orderBy: string = 'created_at',
    direction: 'asc' | 'desc' = 'desc'
  ): Promise<{ data: T[]; nextCursor: string | null }> {
    if (cursor) {
      query = query.where(orderBy, direction === 'desc' ? '<' : '>', cursor);
    }

    const data = await query
      .orderBy(orderBy, direction)
      .limit(limit + 1);

    let nextCursor = null;
    if (data.length > limit) {
      const lastItem = data.pop();
      nextCursor = lastItem[orderBy];
    }

    return { data, nextCursor };
  }

  // Batch insert optimization
  static async batchInsert<T>(
    tableName: string,
    records: T[],
    batchSize: number = 1000
  ): Promise<void> {
    for (let i = 0; i < records.length; i += batchSize) {
      const batch = records.slice(i, i + batchSize);
      await db(tableName).insert(batch);
    }
  }

  // Optimized count query
  static async getEstimatedCount(tableName: string): Promise<number> {
    const result = await db.raw(`
      SELECT reltuples::BIGINT AS estimate
      FROM pg_class
      WHERE relname = ?;
    `, [tableName]);

    return result.rows[0]?.estimate || 0;
  }
}
```

### 5. Frontend Performance Optimizations

```typescript
// frontend/src/hooks/useVirtualScroll.tsx
import { useCallback, useEffect, useRef, useState } from 'react';

interface VirtualScrollOptions {
  itemHeight: number;
  containerHeight: number;
  overscan?: number;
  estimateItemHeight?: (index: number) => number;
}

export function useVirtualScroll<T>(
  items: T[],
  options: VirtualScrollOptions
) {
  const {
    itemHeight,
    containerHeight,
    overscan = 3,
    estimateItemHeight,
  } = options;

  const [scrollTop, setScrollTop] = useState(0);
  const scrollElementRef = useRef<HTMLDivElement>(null);

  // Calculate visible range
  const startIndex = Math.max(
    0,
    Math.floor(scrollTop / itemHeight) - overscan
  );
  
  const endIndex = Math.min(
    items.length - 1,
    Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan
  );

  const visibleItems = items.slice(startIndex, endIndex + 1);

  // Calculate heights
  const totalHeight = items.length * itemHeight;
  const offsetY = startIndex * itemHeight;

  const handleScroll = useCallback((e: Event) => {
    const target = e.target as HTMLDivElement;
    setScrollTop(target.scrollTop);
  }, []);

  useEffect(() => {
    const scrollElement = scrollElementRef.current;
    if (scrollElement) {
      scrollElement.addEventListener('scroll', handleScroll, { passive: true });
      return () => scrollElement.removeEventListener('scroll', handleScroll);
    }
  }, [handleScroll]);

  return {
    scrollElementRef,
    visibleItems,
    totalHeight,
    offsetY,
    startIndex,
    endIndex,
  };
}

// Usage in MessageList component
export const VirtualMessageList: React.FC<{ messages: Message[] }> = ({ messages }) => {
  const { 
    scrollElementRef, 
    visibleItems, 
    totalHeight, 
    offsetY 
  } = useVirtualScroll(messages, {
    itemHeight: 80, // Estimated message height
    containerHeight: 600, // Container height
    overscan: 5,
  });

  return (
    <div
      ref={scrollElementRef}
      style={{
        height: 600,
        overflow: 'auto',
      }}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        <div
          style={{
            transform: `translateY(${offsetY}px)`,
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
          }}
        >
          {visibleItems.map((message, index) => (
            <MessageItem
              key={message.id}
              message={message}
              style={{ height: 80 }}
            />
          ))}
        </div>
      </div>
    </div>
  );
};
```

### 6. API Response Caching

```typescript
// frontend/src/services/apiCache.ts
import { LRUCache } from 'lru-cache';

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  etag?: string;
}

export class APICache {
  private cache: LRUCache<string, CacheEntry<any>>;
  
  constructor(options?: { max?: number; ttl?: number }) {
    this.cache = new LRUCache({
      max: options?.max || 100,
      ttl: options?.ttl || 5 * 60 * 1000, // 5 minutes default
    });
  }

  // Generate cache key from request params
  private getCacheKey(url: string, params?: any): string {
    const paramString = params ? JSON.stringify(params) : '';
    return `${url}:${paramString}`;
  }

  // Get cached data
  get<T>(url: string, params?: any): T | null {
    const key = this.getCacheKey(url, params);
    const entry = this.cache.get(key);
    
    if (entry) {
      return entry.data as T;
    }
    
    return null;
  }

  // Set cache data
  set<T>(url: string, data: T, params?: any, etag?: string): void {
    const key = this.getCacheKey(url, params);
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      etag,
    });
  }

  // Get ETag for conditional requests
  getETag(url: string, params?: any): string | undefined {
    const key = this.getCacheKey(url, params);
    const entry = this.cache.get(key);
    return entry?.etag;
  }

  // Invalidate cache entries
  invalidate(pattern?: string): void {
    if (pattern) {
      // Invalidate entries matching pattern
      for (const key of this.cache.keys()) {
        if (key.includes(pattern)) {
          this.cache.delete(key);
        }
      }
    } else {
      // Clear all cache
      this.cache.clear();
    }
  }

  // Check if data is stale
  isStale(url: string, params?: any, maxAge: number = 60000): boolean {
    const key = this.getCacheKey(url, params);
    const entry = this.cache.get(key);
    
    if (!entry) return true;
    
    return Date.now() - entry.timestamp > maxAge;
  }
}

// Singleton instance
export const apiCache = new APICache({
  max: 200,
  ttl: 10 * 60 * 1000, // 10 minutes
});

// Enhanced fetch with caching
export async function cachedFetch<T>(
  url: string,
  options?: RequestInit & { 
    params?: any; 
    maxAge?: number;
    forceRefresh?: boolean;
  }
): Promise<T> {
  const { params, maxAge, forceRefresh, ...fetchOptions } = options || {};

  // Check cache first
  if (!forceRefresh) {
    const cached = apiCache.get<T>(url, params);
    if (cached && !apiCache.isStale(url, params, maxAge)) {
      return cached;
    }
  }

  // Add conditional headers
  const etag = apiCache.getETag(url, params);
  if (etag && !forceRefresh) {
    fetchOptions.headers = {
      ...fetchOptions.headers,
      'If-None-Match': etag,
    };
  }

  const response = await fetch(url, fetchOptions);

  // Handle 304 Not Modified
  if (response.status === 304) {
    const cached = apiCache.get<T>(url, params);
    if (cached) return cached;
  }

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const data = await response.json();
  const responseEtag = response.headers.get('ETag') || undefined;

  // Cache the response
  apiCache.set(url, data, params, responseEtag);

  return data;
}
```

### 7. Load Balancer Configuration

```nginx
# nginx.conf for load balancing
upstream chat_backend {
    least_conn; # Use least connections algorithm
    
    # Backend servers
    server backend1:3000 weight=1 max_fails=3 fail_timeout=30s;
    server backend2:3000 weight=1 max_fails=3 fail_timeout=30s;
    server backend3:3000 weight=1 max_fails=3 fail_timeout=30s;
    
    # Keepalive connections for performance
    keepalive 32;
}

# WebSocket upstream
upstream chat_websocket {
    ip_hash; # Sticky sessions for WebSocket
    
    server backend1:3000;
    server backend2:3000;
    server backend3:3000;
}

server {
    listen 80;
    server_name chat.example.com;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;

    # API endpoints
    location /api {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        
        # Headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Caching headers
        proxy_set_header Cache-Control "no-cache, no-store, must-revalidate";
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # WebSocket endpoint
    location /socket.io {
        proxy_pass http://chat_websocket;
        proxy_http_version 1.1;
        
        # WebSocket headers
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Timeouts for WebSocket
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
    }

    # Static files with caching
    location /static {
        root /var/www/chat;
        expires 1y;
        add_header Cache-Control "public, immutable";
        
        # Gzip static files
        gzip_static on;
    }

    # Health check endpoint
    location /health {
        access_log off;
        proxy_pass http://chat_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
    }
}
```

## Performance Monitoring

### Metrics Collection

```typescript
// backend/src/monitoring/metrics.ts
import { register, Counter, Histogram, Gauge } from 'prom-client';

// Define metrics
export const httpRequestDuration = new Histogram({
  name: 'http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status'],
  buckets: [0.1, 0.5, 1, 2, 5],
});

export const httpRequestTotal = new Counter({
  name: 'http_requests_total',
  help: 'Total number of HTTP requests',
  labelNames: ['method', 'route', 'status'],
});

export const activeConnections = new Gauge({
  name: 'websocket_active_connections',
  help: 'Number of active WebSocket connections',
});

export const messagesSent = new Counter({
  name: 'messages_sent_total',
  help: 'Total number of messages sent',
  labelNames: ['room_type'],
});

export const cacheHits = new Counter({
  name: 'cache_hits_total',
  help: 'Total number of cache hits',
  labelNames: ['cache_type'],
});

export const cacheMisses = new Counter({
  name: 'cache_misses_total',
  help: 'Total number of cache misses',
  labelNames: ['cache_type'],
});

// Metrics middleware
export const metricsMiddleware = (req: Request, res: Response, next: NextFunction) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = (Date.now() - start) / 1000;
    const route = req.route?.path || 'unknown';
    const labels = {
      method: req.method,
      route,
      status: res.statusCode.toString(),
    };
    
    httpRequestDuration.observe(labels, duration);
    httpRequestTotal.inc(labels);
  });
  
  next();
};

// Export metrics endpoint
export const metricsEndpoint = async (req: Request, res: Response) => {
  res.set('Content-Type', register.contentType);
  const metrics = await register.metrics();
  res.send(metrics);
};
```

## Performance Testing

### Load Testing Script

```javascript
// loadtest/k6-script.js
import http from 'k6/http';
import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const messageDeliveryTime = new Trend('message_delivery_time');
const connectionErrors = new Rate('connection_errors');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 500 },  // Ramp up to 500 users
    { duration: '10m', target: 1000 }, // Ramp up to 1000 users
    { duration: '5m', target: 1000 },  // Stay at 1000 users
    { duration: '2m', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
    message_delivery_time: ['p(95)<100'], // 95% of messages under 100ms
    connection_errors: ['rate<0.01'], // Less than 1% connection errors
  },
};

const BASE_URL = 'https://chat.example.com';

export default function () {
  // Simulate user login
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, {
    email: `user${__VU}@example.com`,
    password: 'password123',
  });

  check(loginRes, {
    'login successful': (r) => r.status === 200,
  });

  const token = loginRes.json('token');

  // WebSocket connection for real-time chat
  const wsUrl = `wss://chat.example.com/socket.io/?token=${token}`;
  
  ws.connect(wsUrl, {}, function (socket) {
    socket.on('open', () => {
      console.log('WebSocket connected');
      
      // Join a room
      socket.send(JSON.stringify({
        event: 'join_room',
        data: { roomId: 'general' },
      }));
    });

    socket.on('message', (data) => {
      const message = JSON.parse(data);
      
      if (message.event === 'new_message') {
        const deliveryTime = Date.now() - message.timestamp;
        messageDeliveryTime.add(deliveryTime);
      }
    });

    socket.on('error', (e) => {
      connectionErrors.add(1);
      console.error('WebSocket error:', e);
    });

    // Send messages periodically
    socket.setInterval(() => {
      socket.send(JSON.stringify({
        event: 'send_message',
        data: {
          roomId: 'general',
          content: `Test message from user ${__VU}`,
          timestamp: Date.now(),
        },
      }));
    }, 5000);

    // Keep connection alive for test duration
    socket.setTimeout(() => {
      socket.close();
    }, 60000);
  });

  sleep(1);
}
```

## Optimization Checklist

### Backend Optimizations
- [x] Redis caching for API responses
- [x] Connection pooling for database
- [x] Optimized database indexes
- [x] Socket.io Redis adapter
- [x] Compression middleware
- [x] Rate limiting
- [x] Query optimization

### Frontend Optimizations
- [x] Virtual scrolling for messages
- [x] React.memo for components
- [x] Lazy loading routes
- [x] API response caching
- [x] Image optimization
- [x] Bundle size optimization
- [x] Service worker caching

### Infrastructure
- [x] Load balancer configuration
- [x] Horizontal scaling setup
- [x] CDN for static assets
- [x] Health check endpoints
- [x] Monitoring and metrics
- [x] Auto-scaling policies

### Performance Targets
- [x] < 100ms message delivery
- [x] < 500ms API response time
- [x] Support 1000+ concurrent users
- [x] < 3s initial page load
- [x] 99.9% uptime