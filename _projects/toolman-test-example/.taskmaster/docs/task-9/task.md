# Task 9: Performance Optimization and Scaling

## Overview
This task focuses on optimizing the Task Master platform for high performance and scalability to support 1000+ concurrent users with sub-100ms message delivery latency. The implementation covers caching strategies, Socket.io optimizations, horizontal scaling, database performance, and frontend optimizations.

## Technical Requirements

### Performance Targets
- **Concurrent Users**: 1000+ simultaneous connections
- **Message Latency**: < 100ms end-to-end delivery
- **API Response Time**: < 50ms for cached requests, < 200ms for uncached
- **Frontend Load Time**: < 3 seconds initial load
- **Memory Usage**: < 512MB per backend instance
- **CPU Usage**: < 70% under normal load

### Dependencies
- Task 5: Real-time Communication Infrastructure
- Task 7: Collaborative Features
- Task 8: Admin Dashboard and Analytics

## Implementation Guide

### 1. Caching Strategy Implementation

#### Redis Setup and Configuration
```typescript
// config/redis.ts
import Redis from 'ioredis';

export const redisClient = new Redis({
  host: process.env.REDIS_HOST || 'localhost',
  port: parseInt(process.env.REDIS_PORT || '6379'),
  password: process.env.REDIS_PASSWORD,
  maxRetriesPerRequest: 3,
  enableReadyCheck: true,
  lazyConnect: true,
});

export const redisPubClient = redisClient.duplicate();
export const redisSubClient = redisClient.duplicate();

// Connection pool for high throughput
export const redisPool = new Redis.Cluster([
  { host: process.env.REDIS_HOST, port: 6379 }
], {
  redisOptions: {
    password: process.env.REDIS_PASSWORD,
  },
  enableReadyCheck: true,
  maxRedirections: 3,
  retryDelayOnFailover: 100,
  retryDelayOnClusterDown: 300,
});
```

#### Cache Middleware Implementation
```typescript
// middleware/cache.ts
import { Request, Response, NextFunction } from 'express';
import { redisClient } from '../config/redis';
import crypto from 'crypto';

interface CacheOptions {
  duration: number;
  prefix?: string;
  varyBy?: (req: Request) => string;
}

export const cacheMiddleware = (options: CacheOptions) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    if (req.method !== 'GET') return next();
    
    const varyKey = options.varyBy ? options.varyBy(req) : '';
    const key = `${options.prefix || 'cache'}:${req.originalUrl}:${varyKey}`;
    const etag = crypto.createHash('md5').update(key).digest('hex');
    
    try {
      // Check ETag
      if (req.headers['if-none-match'] === etag) {
        return res.status(304).end();
      }
      
      const cachedData = await redisClient.get(key);
      
      if (cachedData) {
        const data = JSON.parse(cachedData);
        res.setHeader('X-Cache', 'HIT');
        res.setHeader('ETag', etag);
        return res.json(data);
      }
      
      // Cache miss - intercept response
      const originalJson = res.json;
      res.json = function(data) {
        res.setHeader('X-Cache', 'MISS');
        res.setHeader('ETag', etag);
        
        // Async cache write
        redisClient.set(key, JSON.stringify(data), 'EX', options.duration)
          .catch(err => console.error('Cache write error:', err));
        
        return originalJson.call(this, data);
      };
      
      next();
    } catch (error) {
      console.error('Cache middleware error:', error);
      next();
    }
  };
};

// Cache invalidation helper
export const invalidateCache = async (pattern: string) => {
  const keys = await redisClient.keys(pattern);
  if (keys.length > 0) {
    await redisClient.del(...keys);
  }
};
```

#### API Response Caching
```typescript
// routes/projects.ts
router.get('/projects', 
  cacheMiddleware({ 
    duration: 300, // 5 minutes
    prefix: 'projects',
    varyBy: (req) => req.user?.id || 'anonymous'
  }),
  async (req, res) => {
    const projects = await projectService.getProjects(req.user.id);
    res.json(projects);
  }
);

// Invalidate on updates
router.post('/projects', async (req, res) => {
  const project = await projectService.createProject(req.body);
  await invalidateCache('projects:*');
  res.json(project);
});
```

### 2. Socket.io Optimization

#### Optimized Socket.io Configuration
```typescript
// config/socketio.ts
import { Server } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { redisPubClient, redisSubClient } from './redis';

export const configureSocketIO = (io: Server) => {
  // Redis adapter for horizontal scaling
  io.adapter(createAdapter(redisPubClient, redisSubClient));
  
  // Optimization settings
  io.engine.opts = {
    ...io.engine.opts,
    pingTimeout: 25000,
    pingInterval: 10000,
    upgradeTimeout: 10000,
    maxHttpBufferSize: 1e6, // 1MB
    transports: ['websocket'], // Disable polling
    allowEIO3: true,
    cors: {
      origin: process.env.CLIENT_URL,
      credentials: true,
    },
  };
  
  // Connection middleware for auth and rate limiting
  io.use(socketAuthMiddleware);
  io.use(socketRateLimitMiddleware);
};
```

#### Room-based Broadcasting
```typescript
// services/realtimeService.ts
export class RealtimeService {
  private io: Server;
  private roomSubscriptions = new Map<string, Set<string>>();
  
  constructor(io: Server) {
    this.io = io;
  }
  
  // Optimized room management
  async joinRoom(socketId: string, roomId: string) {
    const socket = this.io.sockets.sockets.get(socketId);
    if (!socket) return;
    
    // Leave other rooms to reduce memory usage
    const currentRooms = Array.from(socket.rooms);
    for (const room of currentRooms) {
      if (room !== socketId && room !== roomId) {
        socket.leave(room);
      }
    }
    
    socket.join(roomId);
    
    // Track subscriptions
    if (!this.roomSubscriptions.has(roomId)) {
      this.roomSubscriptions.set(roomId, new Set());
    }
    this.roomSubscriptions.get(roomId)!.add(socketId);
  }
  
  // Efficient broadcasting
  async broadcastToRoom(roomId: string, event: string, data: any) {
    // Use volatile for non-critical updates
    const isVolatile = event.includes('typing') || event.includes('presence');
    
    const emitter = isVolatile 
      ? this.io.to(roomId).volatile 
      : this.io.to(roomId);
    
    // Compress large payloads
    if (JSON.stringify(data).length > 1024) {
      emitter.compress(true);
    }
    
    emitter.emit(event, data);
  }
  
  // Binary data transmission for files
  async sendBinaryData(roomId: string, data: Buffer, metadata: any) {
    this.io.to(roomId).emit('binary-data', {
      metadata,
      data: data.toString('base64')
    });
  }
}
```

### 3. Horizontal Scaling Architecture

#### Stateless Backend Design
```typescript
// app.ts
import cluster from 'cluster';
import os from 'os';

if (cluster.isPrimary && process.env.NODE_ENV === 'production') {
  const numCPUs = os.cpus().length;
  
  // Fork workers
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }
  
  cluster.on('exit', (worker, code, signal) => {
    console.log(`Worker ${worker.process.pid} died`);
    cluster.fork(); // Restart worker
  });
} else {
  // Worker process
  startServer();
}

// Stateless session management
app.use(session({
  store: new RedisStore({ client: redisClient }),
  secret: process.env.SESSION_SECRET!,
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 1000 * 60 * 60 * 24 * 7 // 7 days
  }
}));
```

#### Load Balancer Configuration (nginx)
```nginx
# /etc/nginx/sites-available/taskmaster
upstream taskmaster_backend {
    least_conn;
    server 127.0.0.1:3001 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3002 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3003 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3004 weight=1 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    listen 80;
    server_name taskmaster.example.com;
    
    # Enable gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;
    
    # WebSocket support
    location /socket.io/ {
        proxy_pass http://taskmaster_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Socket.io specific
        proxy_buffering off;
        proxy_read_timeout 86400;
    }
    
    # API routes
    location /api/ {
        proxy_pass http://taskmaster_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Performance optimizations
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
        proxy_busy_buffers_size 8k;
        
        # Caching for GET requests
        proxy_cache_methods GET HEAD;
        proxy_cache_valid 200 1m;
        proxy_cache_bypass $http_cache_control;
        add_header X-Cache-Status $upstream_cache_status;
    }
    
    # Static assets
    location / {
        root /var/www/taskmaster/dist;
        try_files $uri $uri/ /index.html;
        
        # Browser caching
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
```

### 4. Database Optimizations

#### Indexing Strategy
```sql
-- User queries optimization
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active, created_at);

-- Project queries optimization
CREATE INDEX idx_projects_user ON projects(user_id, created_at);
CREATE INDEX idx_projects_status ON projects(status, updated_at);

-- Task queries optimization
CREATE INDEX idx_tasks_project ON tasks(project_id, status, priority);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id, status, due_date);
CREATE INDEX idx_tasks_search ON tasks USING GIN(to_tsvector('english', title || ' ' || description));

-- Message queries optimization
CREATE INDEX idx_messages_room ON messages(room_id, created_at DESC);
CREATE INDEX idx_messages_user ON messages(user_id, created_at DESC);

-- Composite indexes for common queries
CREATE INDEX idx_tasks_project_status_priority ON tasks(project_id, status, priority, created_at);
CREATE INDEX idx_messages_room_unread ON messages(room_id, is_read, created_at DESC);
```

#### Query Optimization
```typescript
// repositories/taskRepository.ts
export class TaskRepository {
  // Optimized task retrieval with pagination
  async getTasksByProject(projectId: string, options: PaginationOptions) {
    const query = this.db
      .select(
        'tasks.id',
        'tasks.title',
        'tasks.status',
        'tasks.priority',
        'tasks.due_date',
        'users.name as assignee_name',
        'users.avatar_url as assignee_avatar'
      )
      .from('tasks')
      .leftJoin('users', 'tasks.assignee_id', 'users.id')
      .where('tasks.project_id', projectId)
      .orderBy('tasks.priority', 'desc')
      .orderBy('tasks.created_at', 'desc')
      .limit(options.limit)
      .offset(options.offset);
    
    // Use query result cache
    const cacheKey = `tasks:project:${projectId}:${options.offset}:${options.limit}`;
    return this.cacheQuery(cacheKey, query, 300); // 5 min cache
  }
  
  // Batch operations for efficiency
  async updateTasksBatch(updates: TaskUpdate[]) {
    const chunks = chunk(updates, 100); // Process in chunks
    
    await Promise.all(chunks.map(chunk => 
      this.db.transaction(async trx => {
        for (const update of chunk) {
          await trx('tasks')
            .where('id', update.id)
            .update(update.data);
        }
      })
    ));
  }
}
```

#### Connection Pooling
```typescript
// config/database.ts
import knex from 'knex';

export const db = knex({
  client: 'postgresql',
  connection: {
    host: process.env.DB_HOST,
    port: parseInt(process.env.DB_PORT || '5432'),
    user: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: process.env.DB_NAME,
  },
  pool: {
    min: 2,
    max: 10,
    acquireTimeoutMillis: 30000,
    createTimeoutMillis: 30000,
    destroyTimeoutMillis: 5000,
    idleTimeoutMillis: 30000,
    reapIntervalMillis: 1000,
    createRetryIntervalMillis: 100,
  },
  asyncStackTraces: process.env.NODE_ENV !== 'production',
});

// Monitor pool health
db.on('query', (query) => {
  if (process.env.LOG_QUERIES === 'true') {
    console.log('Query:', query.sql);
  }
});
```

### 5. Frontend Performance Optimizations

#### Code Splitting and Lazy Loading
```typescript
// routes/AppRoutes.tsx
import { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';

// Lazy load route components
const Dashboard = lazy(() => import('../pages/Dashboard'));
const ProjectView = lazy(() => import('../pages/ProjectView'));
const TaskDetails = lazy(() => import('../pages/TaskDetails'));
const AdminPanel = lazy(() => import('../pages/AdminPanel'));

export const AppRoutes = () => {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/project/:id" element={<ProjectView />} />
        <Route path="/task/:id" element={<TaskDetails />} />
        <Route path="/admin/*" element={<AdminPanel />} />
      </Routes>
    </Suspense>
  );
};
```

#### Component Memoization
```typescript
// components/TaskList.tsx
import { memo, useMemo, useCallback } from 'react';

interface TaskListProps {
  tasks: Task[];
  onTaskClick: (id: string) => void;
  filters: TaskFilters;
}

export const TaskList = memo(({ tasks, onTaskClick, filters }: TaskListProps) => {
  // Memoize filtered and sorted tasks
  const filteredTasks = useMemo(() => {
    return tasks
      .filter(task => {
        if (filters.status && task.status !== filters.status) return false;
        if (filters.priority && task.priority !== filters.priority) return false;
        if (filters.assignee && task.assignee_id !== filters.assignee) return false;
        return true;
      })
      .sort((a, b) => {
        // Priority sort
        const priorityOrder = { high: 3, medium: 2, low: 1 };
        return priorityOrder[b.priority] - priorityOrder[a.priority];
      });
  }, [tasks, filters]);
  
  // Memoize click handler
  const handleTaskClick = useCallback((id: string) => {
    onTaskClick(id);
  }, [onTaskClick]);
  
  return (
    <VirtualList
      items={filteredTasks}
      height={600}
      itemHeight={80}
      renderItem={(task) => (
        <TaskItem
          key={task.id}
          task={task}
          onClick={() => handleTaskClick(task.id)}
        />
      )}
    />
  );
}, (prevProps, nextProps) => {
  // Custom comparison for deep equality
  return (
    prevProps.tasks === nextProps.tasks &&
    prevProps.onTaskClick === nextProps.onTaskClick &&
    JSON.stringify(prevProps.filters) === JSON.stringify(nextProps.filters)
  );
});
```

#### Virtual Scrolling Implementation
```typescript
// components/VirtualList.tsx
import { useVirtualizer } from '@tanstack/react-virtual';
import { useRef } from 'react';

interface VirtualListProps<T> {
  items: T[];
  height: number;
  itemHeight: number;
  renderItem: (item: T) => React.ReactNode;
}

export function VirtualList<T>({ items, height, itemHeight, renderItem }: VirtualListProps<T>) {
  const parentRef = useRef<HTMLDivElement>(null);
  
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => itemHeight,
    overscan: 5,
  });
  
  return (
    <div
      ref={parentRef}
      style={{
        height: `${height}px`,
        overflow: 'auto',
      }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => (
          <div
            key={virtualItem.key}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }}
          >
            {renderItem(items[virtualItem.index])}
          </div>
        ))}
      </div>
    </div>
  );
}
```

### 6. Monitoring and Metrics

#### Performance Monitoring Setup
```typescript
// monitoring/performance.ts
import { StatsD } from 'node-statsd';
import { performance } from 'perf_hooks';

const statsd = new StatsD({
  host: process.env.STATSD_HOST || 'localhost',
  port: 8125,
  prefix: 'taskmaster.',
});

export class PerformanceMonitor {
  // Track API response times
  static trackApiCall(route: string, method: string) {
    return (req: Request, res: Response, next: NextFunction) => {
      const start = performance.now();
      
      res.on('finish', () => {
        const duration = performance.now() - start;
        statsd.timing(`api.${method}.${route}`, duration);
        statsd.increment(`api.${method}.${route}.${res.statusCode}`);
      });
      
      next();
    };
  }
  
  // Track Socket.io events
  static trackSocketEvent(event: string, start: number) {
    const duration = performance.now() - start;
    statsd.timing(`socket.${event}`, duration);
    statsd.increment(`socket.${event}.count`);
  }
  
  // Track cache performance
  static trackCacheHit(key: string, hit: boolean) {
    statsd.increment(`cache.${hit ? 'hit' : 'miss'}`);
    statsd.increment(`cache.${key}.${hit ? 'hit' : 'miss'}`);
  }
  
  // System metrics
  static startSystemMetrics() {
    setInterval(() => {
      const usage = process.memoryUsage();
      statsd.gauge('memory.rss', usage.rss);
      statsd.gauge('memory.heap_used', usage.heapUsed);
      statsd.gauge('memory.heap_total', usage.heapTotal);
      
      const cpuUsage = process.cpuUsage();
      statsd.gauge('cpu.user', cpuUsage.user);
      statsd.gauge('cpu.system', cpuUsage.system);
    }, 10000); // Every 10 seconds
  }
}
```

## Testing Strategy

### Load Testing Script
```javascript
// tests/load/k6-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { WebSocket } from 'k6/experimental/websockets';

export let options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up to 100 users
    { duration: '5m', target: 500 },   // Ramp up to 500 users
    { duration: '10m', target: 1000 }, // Ramp up to 1000 users
    { duration: '5m', target: 1000 },  // Stay at 1000 users
    { duration: '2m', target: 0 },     // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<200', 'p(99)<500'], // 95% of requests under 200ms
    websocket_ping: ['p(95)<100'],                  // 95% of WebSocket pings under 100ms
    http_req_failed: ['rate<0.01'],                 // Error rate under 1%
  },
};

export default function() {
  // Test API endpoints
  const apiResponse = http.get('https://taskmaster.example.com/api/projects');
  check(apiResponse, {
    'API status is 200': (r) => r.status === 200,
    'API response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  // Test WebSocket connections
  const ws = new WebSocket('wss://taskmaster.example.com/socket.io/');
  
  ws.on('open', () => {
    ws.send(JSON.stringify({ type: 'join_room', room_id: 'test-room' }));
  });
  
  ws.on('message', (data) => {
    const message = JSON.parse(data);
    check(message, {
      'WebSocket message received': () => true,
    });
  });
  
  sleep(1);
}
```

## Deployment Checklist

1. **Infrastructure Setup**
   - [ ] Redis cluster configured with persistence
   - [ ] PostgreSQL with read replicas
   - [ ] Load balancer (nginx/HAProxy) configured
   - [ ] CDN for static assets
   - [ ] Monitoring stack (Prometheus/Grafana)

2. **Application Configuration**
   - [ ] Environment variables for all services
   - [ ] Connection pooling configured
   - [ ] Cache TTLs optimized
   - [ ] Socket.io adapter configured
   - [ ] Rate limiting enabled

3. **Performance Validation**
   - [ ] Load tests passing with 1000+ users
   - [ ] Message latency < 100ms verified
   - [ ] API response times meeting targets
   - [ ] Memory usage within limits
   - [ ] Zero downtime deployment tested

4. **Monitoring Setup**
   - [ ] APM (Application Performance Monitoring) configured
   - [ ] Alerts for performance degradation
   - [ ] Dashboard for real-time metrics
   - [ ] Log aggregation configured
   - [ ] Error tracking enabled