# Toolman Usage Guide for Task 5: Real-time Communication with Socket.io

## Overview

This guide provides comprehensive instructions for using Toolman to implement Socket.io real-time communication features. It covers the complete workflow from initial setup to production deployment, including best practices for testing, debugging, and performance optimization.

## Prerequisites

Before starting, ensure you have:
- Node.js 16+ installed
- Redis server available (local or remote)
- MongoDB connection from Task 4
- JWT authentication system from Task 3
- Basic understanding of WebSocket protocols

## Implementation Workflow

### Phase 1: Initial Setup

#### 1.1 Install Dependencies

```bash
# Core Socket.io packages
npm install socket.io @socket.io/redis-adapter redis jsonwebtoken

# Type definitions for TypeScript
npm install --save-dev @types/socket.io socket.io-client @types/redis

# Additional utilities
npm install compression limiter
npm install --save-dev @types/compression
```

#### 1.2 Project Structure Setup

Create the following directory structure:

```
src/
├── socket/
│   ├── server.ts              # Main Socket.io server
│   ├── types.ts               # TypeScript interfaces
│   ├── middleware/
│   │   ├── auth.ts           # JWT authentication
│   │   └── errorHandler.ts   # Error handling
│   ├── handlers/
│   │   ├── index.ts          # Handler registration
│   │   ├── messageHandlers.ts # Message events
│   │   ├── typingHandlers.ts  # Typing indicators
│   │   ├── presenceHandlers.ts # User presence
│   │   └── roomHandlers.ts    # Room management
│   ├── config/
│   │   └── redis.ts          # Redis configuration
│   ├── security/
│   │   ├── index.ts          # Security measures
│   │   └── rateLimiter.ts    # Rate limiting
│   ├── optimizations/
│   │   └── performance.ts    # Performance enhancements
│   └── utils/
│       ├── validators.ts     # Input validation
│       └── sanitizer.ts      # Data sanitization
```

#### 1.3 Environment Configuration

Update `.env` file:

```env
# Socket.io Configuration
SOCKET_PORT=3001
SOCKET_PATH=/socket.io
FRONTEND_URL=http://localhost:3000

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=
REDIS_DB=0

# Performance Settings
SOCKET_PING_TIMEOUT=60000
SOCKET_PING_INTERVAL=25000
SOCKET_MAX_HTTP_BUFFER_SIZE=1000000

# Security Settings
SOCKET_RATE_LIMIT_MESSAGES=30
SOCKET_RATE_LIMIT_WINDOW=60
SOCKET_MAX_CONNECTIONS_PER_IP=5
```

### Phase 2: Core Implementation

#### 2.1 Socket.io Server Setup

Start with the main server configuration:

```typescript
// src/socket/server.ts
import { Server as HttpServer } from 'http';
import { Server as SocketServer } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { RedisManager } from './config/redis';
import { socketAuthMiddleware } from './middleware/auth';
import { registerEventHandlers } from './handlers';
import { PerformanceOptimizer } from './optimizations/performance';
import { SocketSecurity } from './security';

export class SocketManager {
  private io: SocketServer;
  private redisManager: RedisManager;
  private performanceOptimizer: PerformanceOptimizer;
  private security: SocketSecurity;

  constructor(httpServer: HttpServer) {
    this.initialize(httpServer);
  }

  private async initialize(httpServer: HttpServer) {
    // 1. Setup Redis
    this.redisManager = RedisManager.getInstance();
    await this.redisManager.connect();

    // 2. Create Socket.io server
    this.io = new SocketServer(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL,
        credentials: true
      },
      path: process.env.SOCKET_PATH || '/socket.io',
      transports: ['websocket', 'polling'],
      pingTimeout: parseInt(process.env.SOCKET_PING_TIMEOUT || '60000'),
      pingInterval: parseInt(process.env.SOCKET_PING_INTERVAL || '25000'),
      maxHttpBufferSize: parseInt(process.env.SOCKET_MAX_HTTP_BUFFER_SIZE || '1000000')
    });

    // 3. Configure Redis adapter
    this.io.adapter(createAdapter(
      this.redisManager.getPublisher(),
      this.redisManager.getSubscriber()
    ));

    // 4. Apply middleware
    this.io.use(socketAuthMiddleware);

    // 5. Setup security
    this.security = new SocketSecurity(this.io);

    // 6. Setup performance optimizations
    this.performanceOptimizer = new PerformanceOptimizer(this.io);

    // 7. Register handlers
    this.io.on('connection', (socket) => {
      console.log(`User connected: ${socket.user.id}`);
      registerEventHandlers(socket, this.io);
    });
  }
}
```

#### 2.2 Authentication Implementation

Implement JWT-based authentication:

```typescript
// src/socket/middleware/auth.ts
export const socketAuthMiddleware = async (socket: SocketWithUser, next: Function) => {
  try {
    // Extract token from handshake
    const token = socket.handshake.auth.token || 
                  socket.handshake.headers.authorization?.split(' ')[1];

    if (!token) {
      return next(new Error('Authentication required'));
    }

    // Verify JWT
    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload;

    // Verify user exists
    const user = await User.findById(decoded.userId).select('-password');
    if (!user || !user.isActive) {
      return next(new Error('Invalid user'));
    }

    // Attach user to socket
    socket.user = {
      id: user._id.toString(),
      username: user.username,
      email: user.email
    };

    // Join personal notification room
    socket.join(`user:${socket.user.id}`);

    next();
  } catch (error) {
    next(new Error('Authentication failed'));
  }
};
```

### Phase 3: Event Handler Patterns

#### 3.1 Message Handler Pattern

Implement a consistent pattern for all handlers:

```typescript
// Handler factory pattern
export const createHandler = (
  name: string,
  validator: (data: any) => ValidationResult,
  handler: (socket: Socket, io: Server, data: any) => Promise<void>
) => {
  return (socket: Socket, io: Server) => async (data: any) => {
    try {
      // 1. Validate input
      const validation = validator(data);
      if (!validation.valid) {
        return socket.emit('error', { 
          event: name,
          message: validation.error 
        });
      }

      // 2. Check rate limits
      if (!await checkRateLimit(socket.user.id, name)) {
        return socket.emit('error', { 
          event: name,
          message: 'Rate limit exceeded' 
        });
      }

      // 3. Execute handler
      await handler(socket, io, data);

    } catch (error) {
      console.error(`Error in ${name}:`, error);
      socket.emit('error', { 
        event: name,
        message: 'Internal server error' 
      });
    }
  };
};
```

#### 3.2 Implementing Specific Handlers

```typescript
// Example: Message handler
export const sendMessage = createHandler(
  'send-message',
  validateMessage,
  async (socket, io, data) => {
    const { roomId, content, attachments } = data;

    // Verify room access
    const hasAccess = await verifyRoomAccess(socket.user.id, roomId);
    if (!hasAccess) {
      throw new Error('Access denied');
    }

    // Create message
    const message = await Message.create({
      room: roomId,
      sender: socket.user.id,
      content: sanitizeContent(content),
      attachments: sanitizeAttachments(attachments),
      createdAt: new Date()
    });

    // Populate and emit
    await message.populate('sender', 'username avatar');
    io.to(`room:${roomId}`).emit('new-message', formatMessage(message));
  }
);
```

### Phase 4: Testing Real-time Features

#### 4.1 Unit Testing Setup

```typescript
// src/socket/__tests__/setup.ts
import { createServer } from 'http';
import { Server } from 'socket.io';
import { AddressInfo } from 'net';
import Client from 'socket.io-client';

export async function createTestServer() {
  const httpServer = createServer();
  const io = new Server(httpServer);
  
  await new Promise((resolve) => {
    httpServer.listen(0, () => resolve(null));
  });

  const port = (httpServer.address() as AddressInfo).port;
  const serverUrl = `http://localhost:${port}`;

  return { httpServer, io, serverUrl };
}

export function createTestClient(url: string, auth: any) {
  return Client(url, {
    auth,
    transports: ['websocket'],
    autoConnect: false
  });
}
```

#### 4.2 Testing Event Handlers

```typescript
describe('Message Handlers', () => {
  let server: TestServer;
  let client1: any;
  let client2: any;

  beforeEach(async () => {
    server = await createTestServer();
    
    // Create authenticated clients
    client1 = createTestClient(server.url, { token: 'user1-token' });
    client2 = createTestClient(server.url, { token: 'user2-token' });
    
    await Promise.all([
      client1.connect(),
      client2.connect()
    ]);
  });

  test('should deliver messages to room members', async () => {
    // Join room
    await Promise.all([
      client1.emit('join-room', 'test-room'),
      client2.emit('join-room', 'test-room')
    ]);

    // Setup listener
    const messagePromise = new Promise((resolve) => {
      client2.on('new-message', resolve);
    });

    // Send message
    client1.emit('send-message', {
      roomId: 'test-room',
      content: 'Hello, World!'
    });

    // Verify delivery
    const message = await messagePromise;
    expect(message.content).toBe('Hello, World!');
  });
});
```

### Phase 5: Performance Optimization Techniques

#### 5.1 Latency Measurement

```typescript
// Performance monitoring
export class LatencyMonitor {
  private metrics: Map<string, number[]> = new Map();

  measureEvent(event: string, startTime: number) {
    const latency = Date.now() - startTime;
    
    if (!this.metrics.has(event)) {
      this.metrics.set(event, []);
    }
    
    this.metrics.get(event)!.push(latency);
    
    // Keep only last 1000 measurements
    if (this.metrics.get(event)!.length > 1000) {
      this.metrics.get(event)!.shift();
    }
  }

  getStats(event: string) {
    const measurements = this.metrics.get(event) || [];
    if (measurements.length === 0) return null;

    const sorted = [...measurements].sort((a, b) => a - b);
    return {
      avg: measurements.reduce((a, b) => a + b) / measurements.length,
      p50: sorted[Math.floor(sorted.length * 0.5)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)]
    };
  }
}
```

#### 5.2 Event Batching Implementation

```typescript
// Batch similar events to reduce overhead
export class EventBatcher {
  private batches: Map<string, any[]> = new Map();
  private timers: Map<string, NodeJS.Timeout> = new Map();
  
  constructor(
    private io: Server,
    private batchSize: number = 50,
    private batchWindow: number = 100
  ) {}

  addEvent(roomId: string, event: string, data: any) {
    const key = `${roomId}:${event}`;
    
    if (!this.batches.has(key)) {
      this.batches.set(key, []);
    }
    
    this.batches.get(key)!.push(data);
    
    // Send immediately if batch is full
    if (this.batches.get(key)!.length >= this.batchSize) {
      this.flush(key);
      return;
    }
    
    // Otherwise, set timer
    if (!this.timers.has(key)) {
      this.timers.set(key, setTimeout(() => {
        this.flush(key);
      }, this.batchWindow));
    }
  }

  private flush(key: string) {
    const [roomId, event] = key.split(':');
    const batch = this.batches.get(key) || [];
    
    if (batch.length > 0) {
      this.io.to(roomId).emit(`batch:${event}`, batch);
    }
    
    this.batches.delete(key);
    if (this.timers.has(key)) {
      clearTimeout(this.timers.get(key)!);
      this.timers.delete(key);
    }
  }
}
```

### Phase 6: Debugging WebSocket Connections

#### 6.1 Debug Logging

```typescript
// Enable detailed logging for debugging
export function enableDebugMode(io: Server) {
  io.on('connection', (socket) => {
    console.log(`[DEBUG] New connection: ${socket.id}`);
    console.log(`[DEBUG] User: ${JSON.stringify(socket.user)}`);
    console.log(`[DEBUG] Transport: ${socket.conn.transport.name}`);
    
    // Log all events
    socket.onAny((event, ...args) => {
      console.log(`[DEBUG] Event: ${event}`, args);
    });
    
    socket.on('disconnect', (reason) => {
      console.log(`[DEBUG] Disconnect: ${socket.id}, Reason: ${reason}`);
    });
  });
}
```

#### 6.2 Connection Diagnostics

```typescript
// Diagnostic endpoint
app.get('/debug/socket/connections', authenticate, (req, res) => {
  const io = getSocketIO();
  const sockets = await io.fetchSockets();
  
  const connections = sockets.map(socket => ({
    id: socket.id,
    userId: socket.user?.id,
    transport: socket.conn.transport.name,
    rooms: Array.from(socket.rooms),
    connected: socket.connected,
    handshakeTime: socket.handshake.time
  }));
  
  res.json({
    total: connections.length,
    byTransport: {
      websocket: connections.filter(c => c.transport === 'websocket').length,
      polling: connections.filter(c => c.transport === 'polling').length
    },
    connections
  });
});
```

### Phase 7: Scaling Considerations

#### 7.1 Multi-Instance Deployment

```typescript
// PM2 ecosystem configuration
module.exports = {
  apps: [{
    name: 'socket-server',
    script: './dist/server.js',
    instances: 4, // Number of instances
    exec_mode: 'cluster',
    env: {
      NODE_ENV: 'production',
      SOCKET_PORT: 3001
    },
    error_file: './logs/socket-error.log',
    out_file: './logs/socket-out.log',
    merge_logs: true
  }]
};
```

#### 7.2 Load Balancing Configuration

```nginx
# Nginx configuration for Socket.io
upstream socket_backend {
    ip_hash; # Important for Socket.io
    server localhost:3001;
    server localhost:3002;
    server localhost:3003;
    server localhost:3004;
}

server {
    listen 80;
    server_name api.example.com;

    location /socket.io/ {
        proxy_pass http://socket_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Socket.io specific
        proxy_buffering off;
        proxy_set_header X-NginX-Proxy true;
        proxy_redirect off;
        
        # Timeouts
        proxy_connect_timeout 86400s;
        proxy_send_timeout 86400s;
        proxy_read_timeout 86400s;
    }
}
```

### Phase 8: Production Deployment

#### 8.1 Pre-deployment Checklist

```bash
# 1. Run all tests
npm run test:socket

# 2. Check for vulnerabilities
npm audit fix

# 3. Build for production
npm run build

# 4. Verify Redis connection
redis-cli ping

# 5. Check port availability
lsof -i :3001

# 6. Review environment variables
cat .env | grep SOCKET
```

#### 8.2 Monitoring Setup

```typescript
// Metrics collection
export class SocketMetrics {
  private io: Server;
  
  async getMetrics() {
    const sockets = await this.io.fetchSockets();
    
    return {
      connections: {
        total: sockets.length,
        authenticated: sockets.filter(s => s.user).length,
        byTransport: this.getTransportStats(sockets)
      },
      rooms: {
        total: this.io.sockets.adapter.rooms.size,
        averageSize: this.getAverageRoomSize()
      },
      performance: {
        messageLatency: latencyMonitor.getStats('message'),
        typingLatency: latencyMonitor.getStats('typing'),
        presenceLatency: latencyMonitor.getStats('presence')
      },
      errors: errorCounter.getStats()
    };
  }
}
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Connection Failures

**Problem**: Clients cannot connect to Socket.io server

**Solutions**:
```typescript
// Check CORS configuration
io.on('connection_error', (err) => {
  console.log('Connection error:', err.message);
  console.log('Origin:', err.context.origin);
});

// Verify authentication
socket.on('connect_error', (error) => {
  if (error.message === 'Authentication required') {
    // Refresh token and retry
    refreshToken().then(token => {
      socket.auth = { token };
      socket.connect();
    });
  }
});
```

#### 2. High Latency

**Problem**: Messages take too long to deliver

**Solutions**:
```typescript
// 1. Check Redis latency
const start = Date.now();
await redisClient.ping();
console.log('Redis latency:', Date.now() - start);

// 2. Monitor event queue
console.log('Pending events:', io.engine.clientsCount);

// 3. Enable compression
io.engine.use(compression({
  threshold: 1024,
  level: 6
}));
```

#### 3. Memory Leaks

**Problem**: Memory usage increases over time

**Solutions**:
```typescript
// 1. Proper cleanup
socket.on('disconnect', () => {
  // Remove all listeners
  socket.removeAllListeners();
  
  // Clear any timers
  clearTypingTimers(socket.user.id);
  
  // Clear cache entries
  messageCache.delete(`user:${socket.user.id}`);
});

// 2. Monitor memory usage
setInterval(() => {
  const usage = process.memoryUsage();
  console.log('Memory usage:', {
    rss: `${Math.round(usage.rss / 1024 / 1024)}MB`,
    heap: `${Math.round(usage.heapUsed / 1024 / 1024)}MB`
  });
}, 60000);
```

## Best Practices

### 1. Error Handling

Always implement comprehensive error handling:

```typescript
// Global error handler
io.on('connection', (socket) => {
  socket.on('error', (error) => {
    logger.error('Socket error:', {
      userId: socket.user?.id,
      error: error.message,
      stack: error.stack
    });
    
    socket.emit('error', {
      message: 'An error occurred',
      retry: true
    });
  });
});
```

### 2. Security

Implement defense in depth:

```typescript
// Input validation
const validateMessage = (data: any): ValidationResult => {
  if (!data || typeof data !== 'object') {
    return { valid: false, error: 'Invalid data format' };
  }
  
  if (!data.roomId || typeof data.roomId !== 'string') {
    return { valid: false, error: 'Invalid room ID' };
  }
  
  if (!data.content || data.content.length > 5000) {
    return { valid: false, error: 'Invalid message content' };
  }
  
  return { valid: true };
};
```

### 3. Performance

Optimize for scale:

```typescript
// Use rooms efficiently
io.to(`room:${roomId}`).emit('event', data); // Good
io.emit('event', { roomId, data }); // Bad - sends to everyone

// Minimize payload size
const compactMessage = {
  i: message._id, // Use short keys
  c: message.content,
  s: message.sender.id,
  t: message.createdAt.getTime()
};
```

## Conclusion

This guide covers the complete implementation of Socket.io real-time features. Remember to:

1. Test thoroughly at each phase
2. Monitor performance metrics
3. Implement proper error handling
4. Follow security best practices
5. Optimize for your specific use case

For additional support, refer to the official Socket.io documentation and the troubleshooting section of this guide.