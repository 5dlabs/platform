# Task 5: Real-time Communication with Socket.io

## Overview

This task implements real-time communication features using Socket.io, enabling instant messaging, typing indicators, user presence tracking, and read receipts. The implementation includes JWT-based authentication, Redis adapter for horizontal scaling, and optimizations to achieve sub-100ms message delivery latency.

## Technical Requirements

### Dependencies
- **Task 3**: Authentication API (JWT tokens for socket authentication)
- **Task 4**: MongoDB Collections (message storage and user presence)

### Technology Stack
- Socket.io (v4.x) for WebSocket communication
- Redis adapter for Socket.io clustering
- JWT for socket authentication
- Redis for presence management and pub/sub

## Implementation Guide

### 1. Socket.io Server Setup and Configuration

```typescript
// src/socket/server.ts
import { Server as HttpServer } from 'http';
import { Server as SocketServer } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { createClient } from 'redis';
import { socketAuthMiddleware } from './middleware/auth';
import { registerEventHandlers } from './handlers';

export class SocketManager {
  private io: SocketServer;
  private redisPublisher: any;
  private redisSubscriber: any;

  constructor(httpServer: HttpServer) {
    this.setupRedisClients();
    this.initializeSocketServer(httpServer);
  }

  private async setupRedisClients() {
    this.redisPublisher = createClient({
      url: process.env.REDIS_URL || 'redis://localhost:6379',
      socket: {
        reconnectStrategy: (retries) => Math.min(retries * 50, 500)
      }
    });
    
    this.redisSubscriber = this.redisPublisher.duplicate();
    
    await Promise.all([
      this.redisPublisher.connect(),
      this.redisSubscriber.connect()
    ]);
  }

  private initializeSocketServer(httpServer: HttpServer) {
    this.io = new SocketServer(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL || 'http://localhost:3000',
        credentials: true
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000,
      upgradeTimeout: 10000,
      maxHttpBufferSize: 1e6 // 1MB
    });

    // Configure Redis adapter for horizontal scaling
    this.io.adapter(createAdapter(this.redisPublisher, this.redisSubscriber));

    // Apply authentication middleware
    this.io.use(socketAuthMiddleware);

    // Register event handlers
    this.io.on('connection', (socket) => {
      registerEventHandlers(socket, this.io);
    });
  }

  getIO(): SocketServer {
    return this.io;
  }
}
```

### 2. JWT Authentication for WebSocket Connections

```typescript
// src/socket/middleware/auth.ts
import { Socket } from 'socket.io';
import jwt from 'jsonwebtoken';
import { User } from '../../models/User';

interface SocketWithUser extends Socket {
  user?: any;
}

export const socketAuthMiddleware = async (
  socket: SocketWithUser,
  next: Function
) => {
  try {
    const token = socket.handshake.auth.token || socket.handshake.headers.authorization?.split(' ')[1];
    
    if (!token) {
      return next(new Error('Authentication required'));
    }

    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as any;
    
    // Verify user exists and is active
    const user = await User.findById(decoded.userId).select('-password');
    if (!user || !user.isActive) {
      return next(new Error('Invalid user'));
    }

    socket.user = {
      id: user._id.toString(),
      username: user.username,
      email: user.email
    };

    // Join user's personal room for direct notifications
    socket.join(`user:${socket.user.id}`);

    next();
  } catch (error) {
    next(new Error('Authentication failed'));
  }
};
```

### 3. Real-time Event Implementations

```typescript
// src/socket/handlers/index.ts
import { Socket, Server } from 'socket.io';
import { messageHandlers } from './messageHandlers';
import { presenceHandlers } from './presenceHandlers';
import { typingHandlers } from './typingHandlers';
import { roomHandlers } from './roomHandlers';

export function registerEventHandlers(socket: Socket, io: Server) {
  // Room management
  socket.on('join-room', roomHandlers.joinRoom(socket, io));
  socket.on('leave-room', roomHandlers.leaveRoom(socket, io));

  // Messaging
  socket.on('send-message', messageHandlers.sendMessage(socket, io));
  socket.on('edit-message', messageHandlers.editMessage(socket, io));
  socket.on('delete-message', messageHandlers.deleteMessage(socket, io));
  socket.on('mark-as-read', messageHandlers.markAsRead(socket, io));

  // Typing indicators
  socket.on('typing-start', typingHandlers.startTyping(socket, io));
  socket.on('typing-stop', typingHandlers.stopTyping(socket, io));

  // Presence
  socket.on('update-presence', presenceHandlers.updatePresence(socket, io));
  socket.on('get-room-presence', presenceHandlers.getRoomPresence(socket, io));

  // Connection lifecycle
  socket.on('disconnect', async () => {
    await presenceHandlers.handleDisconnect(socket, io);
  });
}
```

#### Message Handlers

```typescript
// src/socket/handlers/messageHandlers.ts
import { Socket, Server } from 'socket.io';
import { Message } from '../../models/Message';
import { Room } from '../../models/Room';
import { validateMessage } from '../validators';
import { rateLimiter } from '../utils/rateLimiter';

export const messageHandlers = {
  sendMessage: (socket: Socket, io: Server) => async (data: any) => {
    try {
      // Rate limiting
      if (!await rateLimiter.check(socket.user.id, 'message', 30, 60)) {
        return socket.emit('error', { message: 'Rate limit exceeded' });
      }

      // Validate message
      const validation = validateMessage(data);
      if (!validation.valid) {
        return socket.emit('error', { message: validation.error });
      }

      const { roomId, content, attachments } = data;

      // Verify user has access to room
      const room = await Room.findById(roomId);
      if (!room || !room.members.includes(socket.user.id)) {
        return socket.emit('error', { message: 'Access denied' });
      }

      // Create message
      const message = await Message.create({
        room: roomId,
        sender: socket.user.id,
        content,
        attachments,
        readBy: [socket.user.id],
        createdAt: new Date()
      });

      // Populate sender info
      await message.populate('sender', 'username avatar');

      // Emit to all room members with optimized payload
      io.to(`room:${roomId}`).emit('new-message', {
        id: message._id,
        room: roomId,
        sender: {
          id: message.sender._id,
          username: message.sender.username,
          avatar: message.sender.avatar
        },
        content: message.content,
        attachments: message.attachments,
        createdAt: message.createdAt,
        readBy: message.readBy
      });

      // Update room's last message
      await Room.findByIdAndUpdate(roomId, {
        lastMessage: message._id,
        lastActivity: new Date()
      });

    } catch (error) {
      console.error('Send message error:', error);
      socket.emit('error', { message: 'Failed to send message' });
    }
  },

  markAsRead: (socket: Socket, io: Server) => async (data: any) => {
    try {
      const { messageIds, roomId } = data;

      await Message.updateMany(
        {
          _id: { $in: messageIds },
          room: roomId,
          readBy: { $ne: socket.user.id }
        },
        {
          $addToSet: { readBy: socket.user.id }
        }
      );

      // Notify room members about read receipts
      socket.to(`room:${roomId}`).emit('messages-read', {
        messageIds,
        userId: socket.user.id,
        readAt: new Date()
      });

    } catch (error) {
      console.error('Mark as read error:', error);
    }
  }
};
```

#### Typing Indicators

```typescript
// src/socket/handlers/typingHandlers.ts
import { Socket, Server } from 'socket.io';

const typingUsers = new Map<string, Set<string>>();
const typingTimers = new Map<string, NodeJS.Timeout>();

export const typingHandlers = {
  startTyping: (socket: Socket, io: Server) => (data: any) => {
    const { roomId } = data;
    const key = `${roomId}:${socket.user.id}`;

    // Clear existing timer
    if (typingTimers.has(key)) {
      clearTimeout(typingTimers.get(key)!);
    }

    // Add to typing users
    if (!typingUsers.has(roomId)) {
      typingUsers.set(roomId, new Set());
    }
    typingUsers.get(roomId)!.add(socket.user.id);

    // Broadcast to room
    socket.to(`room:${roomId}`).emit('user-typing', {
      roomId,
      userId: socket.user.id,
      username: socket.user.username,
      isTyping: true
    });

    // Auto-stop after 3 seconds
    const timer = setTimeout(() => {
      typingHandlers.stopTyping(socket, io)({ roomId });
    }, 3000);
    typingTimers.set(key, timer);
  },

  stopTyping: (socket: Socket, io: Server) => (data: any) => {
    const { roomId } = data;
    const key = `${roomId}:${socket.user.id}`;

    // Clear timer
    if (typingTimers.has(key)) {
      clearTimeout(typingTimers.get(key)!);
      typingTimers.delete(key);
    }

    // Remove from typing users
    if (typingUsers.has(roomId)) {
      typingUsers.get(roomId)!.delete(socket.user.id);
      if (typingUsers.get(roomId)!.size === 0) {
        typingUsers.delete(roomId);
      }
    }

    // Broadcast to room
    socket.to(`room:${roomId}`).emit('user-typing', {
      roomId,
      userId: socket.user.id,
      username: socket.user.username,
      isTyping: false
    });
  }
};
```

### 4. Redis Adapter Configuration for Scaling

```typescript
// src/socket/config/redis.ts
import { createClient, RedisClientType } from 'redis';

export class RedisManager {
  private static instance: RedisManager;
  private publisher: RedisClientType;
  private subscriber: RedisClientType;
  private client: RedisClientType;

  private constructor() {
    const redisConfig = {
      url: process.env.REDIS_URL || 'redis://localhost:6379',
      socket: {
        keepAlive: true,
        reconnectStrategy: (retries: number) => {
          if (retries > 10) {
            console.error('Redis connection failed after 10 retries');
            return new Error('Redis connection failed');
          }
          return Math.min(retries * 100, 3000);
        }
      }
    };

    this.publisher = createClient(redisConfig);
    this.subscriber = createClient(redisConfig);
    this.client = createClient(redisConfig);

    this.setupEventHandlers();
  }

  private setupEventHandlers() {
    const clients = [this.publisher, this.subscriber, this.client];
    
    clients.forEach(client => {
      client.on('error', (err) => console.error('Redis error:', err));
      client.on('connect', () => console.log('Redis connected'));
      client.on('reconnecting', () => console.log('Redis reconnecting...'));
    });
  }

  static getInstance(): RedisManager {
    if (!RedisManager.instance) {
      RedisManager.instance = new RedisManager();
    }
    return RedisManager.instance;
  }

  async connect() {
    await Promise.all([
      this.publisher.connect(),
      this.subscriber.connect(),
      this.client.connect()
    ]);
  }

  getPublisher() {
    return this.publisher;
  }

  getSubscriber() {
    return this.subscriber;
  }

  getClient() {
    return this.client;
  }
}
```

### 5. Error Handling and Reconnection Strategies

```typescript
// src/socket/middleware/errorHandler.ts
import { Socket } from 'socket.io';

export const socketErrorHandler = (socket: Socket) => {
  socket.on('error', (error) => {
    console.error(`Socket error for user ${socket.user?.id}:`, error);
    
    // Send error to client
    socket.emit('error', {
      message: 'An error occurred',
      code: error.code || 'UNKNOWN_ERROR',
      timestamp: new Date()
    });
  });

  // Handle specific error types
  socket.on('connect_error', (error) => {
    if (error.message === 'Authentication required') {
      socket.emit('auth_error', { message: 'Please login again' });
    }
  });
};

// Client-side reconnection configuration
export const clientConfig = {
  reconnection: true,
  reconnectionAttempts: 5,
  reconnectionDelay: 1000,
  reconnectionDelayMax: 5000,
  randomizationFactor: 0.5,
  timeout: 20000,
  autoConnect: true,
  transports: ['websocket', 'polling']
};
```

### 6. Performance Optimization for Sub-100ms Latency

```typescript
// src/socket/optimizations/performance.ts
import { Server } from 'socket.io';
import compression from 'compression';

export class PerformanceOptimizer {
  private io: Server;
  private messageCache: Map<string, any>;
  private presenceCache: Map<string, any>;

  constructor(io: Server) {
    this.io = io;
    this.messageCache = new Map();
    this.presenceCache = new Map();
    this.setupOptimizations();
  }

  private setupOptimizations() {
    // 1. Enable binary compression
    this.io.engine.use(compression({
      threshold: 1024,
      level: 6
    }));

    // 2. Batch similar events
    this.setupEventBatching();

    // 3. Implement message caching
    this.setupMessageCaching();

    // 4. Optimize payload sizes
    this.setupPayloadOptimization();
  }

  private setupEventBatching() {
    const batchedEvents = new Map<string, any[]>();
    const batchTimers = new Map<string, NodeJS.Timeout>();

    this.io.use((socket, next) => {
      const originalEmit = socket.emit;
      
      socket.emit = function(event: string, ...args: any[]) {
        if (event.startsWith('batch:')) {
          const key = `${socket.id}:${event}`;
          
          if (!batchedEvents.has(key)) {
            batchedEvents.set(key, []);
          }
          
          batchedEvents.get(key)!.push(args[0]);
          
          if (!batchTimers.has(key)) {
            batchTimers.set(key, setTimeout(() => {
              const events = batchedEvents.get(key) || [];
              originalEmit.call(socket, event, events);
              batchedEvents.delete(key);
              batchTimers.delete(key);
            }, 50)); // 50ms batching window
          }
        } else {
          originalEmit.apply(socket, [event, ...args]);
        }
      };
      
      next();
    });
  }

  private setupMessageCaching() {
    // Cache recent messages to avoid database queries
    this.io.on('connection', (socket) => {
      socket.on('get-recent-messages', async (roomId: string) => {
        const cacheKey = `messages:${roomId}`;
        
        if (this.messageCache.has(cacheKey)) {
          const cached = this.messageCache.get(cacheKey);
          if (Date.now() - cached.timestamp < 5000) { // 5 second cache
            return socket.emit('recent-messages', cached.data);
          }
        }
        
        // Fetch from database if not cached
        const messages = await this.fetchRecentMessages(roomId);
        this.messageCache.set(cacheKey, {
          data: messages,
          timestamp: Date.now()
        });
        
        socket.emit('recent-messages', messages);
      });
    });
  }

  private setupPayloadOptimization() {
    // Minimize payload sizes
    this.io.on('connection', (socket) => {
      // Use short event names
      socket.on('m', (data) => socket.emit('msg', data)); // 'm' for message
      socket.on('t', (data) => socket.emit('typ', data)); // 't' for typing
      socket.on('p', (data) => socket.emit('prs', data)); // 'p' for presence
    });
  }

  private async fetchRecentMessages(roomId: string): Promise<any[]> {
    // Implementation for fetching messages
    return [];
  }
}
```

### 7. Security Considerations for WebSocket Connections

```typescript
// src/socket/security/index.ts
import { Server, Socket } from 'socket.io';
import { RateLimiter } from 'limiter';
import crypto from 'crypto';

export class SocketSecurity {
  private io: Server;
  private connectionLimiter: Map<string, RateLimiter>;
  private blacklistedIPs: Set<string>;

  constructor(io: Server) {
    this.io = io;
    this.connectionLimiter = new Map();
    this.blacklistedIPs = new Set();
    this.setupSecurity();
  }

  private setupSecurity() {
    // 1. Connection rate limiting
    this.io.use((socket, next) => {
      const ip = socket.handshake.address;
      
      if (this.blacklistedIPs.has(ip)) {
        return next(new Error('Blocked'));
      }

      if (!this.connectionLimiter.has(ip)) {
        this.connectionLimiter.set(ip, new RateLimiter({
          tokensPerInterval: 10,
          interval: 'minute'
        }));
      }

      const limiter = this.connectionLimiter.get(ip)!;
      if (!limiter.tryRemoveTokens(1)) {
        this.blacklistedIPs.add(ip);
        setTimeout(() => this.blacklistedIPs.delete(ip), 3600000); // 1 hour ban
        return next(new Error('Rate limit exceeded'));
      }

      next();
    });

    // 2. Message validation and sanitization
    this.io.on('connection', (socket) => {
      const originalEmit = socket.emit;
      
      socket.emit = function(event: string, ...args: any[]) {
        // Sanitize data before emitting
        const sanitizedArgs = args.map(arg => sanitizeData(arg));
        originalEmit.apply(socket, [event, ...sanitizedArgs]);
      };
    });

    // 3. CSRF protection for socket connections
    this.io.use((socket, next) => {
      const token = socket.handshake.auth.csrfToken;
      const sessionToken = socket.handshake.headers.cookie?.match(/csrf=([^;]+)/)?.[1];
      
      if (!token || !sessionToken || token !== sessionToken) {
        return next(new Error('CSRF validation failed'));
      }
      
      next();
    });
  }
}

function sanitizeData(data: any): any {
  if (typeof data === 'string') {
    // Remove potential XSS vectors
    return data
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;')
      .replace(/\//g, '&#x2F;');
  }
  
  if (typeof data === 'object' && data !== null) {
    const sanitized: any = Array.isArray(data) ? [] : {};
    for (const key in data) {
      sanitized[key] = sanitizeData(data[key]);
    }
    return sanitized;
  }
  
  return data;
}
```

## Testing Strategy

### Unit Tests

```typescript
// src/socket/__tests__/messageHandlers.test.ts
import { createServer } from 'http';
import { Server } from 'socket.io';
import Client from 'socket.io-client';
import { SocketManager } from '../server';

describe('Message Handlers', () => {
  let io: Server;
  let serverSocket: any;
  let clientSocket: any;
  let httpServer: any;

  beforeAll((done) => {
    httpServer = createServer();
    const socketManager = new SocketManager(httpServer);
    io = socketManager.getIO();
    
    httpServer.listen(() => {
      const port = httpServer.address().port;
      clientSocket = Client(`http://localhost:${port}`, {
        auth: { token: 'valid-jwt-token' }
      });
      
      io.on('connection', (socket) => {
        serverSocket = socket;
      });
      
      clientSocket.on('connect', done);
    });
  });

  afterAll(() => {
    io.close();
    clientSocket.close();
  });

  test('should handle message sending', (done) => {
    const testMessage = {
      roomId: 'test-room',
      content: 'Hello, world!'
    };

    clientSocket.emit('send-message', testMessage);

    clientSocket.on('new-message', (message: any) => {
      expect(message.content).toBe(testMessage.content);
      expect(message.room).toBe(testMessage.roomId);
      done();
    });
  });
});
```

### Performance Tests

```typescript
// src/socket/__tests__/performance.test.ts
describe('Socket.io Performance', () => {
  test('message delivery latency should be under 100ms', async () => {
    const latencies: number[] = [];
    
    for (let i = 0; i < 100; i++) {
      const start = Date.now();
      
      await new Promise((resolve) => {
        clientSocket.emit('ping', { timestamp: start });
        clientSocket.once('pong', (data: any) => {
          const latency = Date.now() - data.timestamp;
          latencies.push(latency);
          resolve(undefined);
        });
      });
    }
    
    const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
    const maxLatency = Math.max(...latencies);
    
    expect(avgLatency).toBeLessThan(100);
    expect(maxLatency).toBeLessThan(200);
  });
});
```

## Deployment Considerations

1. **Load Balancing**: Use sticky sessions or session affinity when deploying multiple Socket.io instances
2. **Redis Configuration**: Ensure Redis has sufficient memory and is configured for persistence
3. **Monitoring**: Implement Socket.io metrics collection for connection counts, message rates, and latency
4. **SSL/TLS**: Always use secure WebSocket connections (wss://) in production
5. **Firewall Rules**: Open required ports for WebSocket connections (typically 443 for wss)

## Environment Variables

```env
# Socket.io Configuration
SOCKET_PORT=3001
FRONTEND_URL=http://localhost:3000
SOCKET_PING_TIMEOUT=60000
SOCKET_PING_INTERVAL=25000

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=your-redis-password
REDIS_DB=0

# Security
JWT_SECRET=your-jwt-secret
CSRF_SECRET=your-csrf-secret
```

## Common Issues and Solutions

1. **CORS Issues**: Ensure frontend URL is properly configured in Socket.io options
2. **Authentication Failures**: Verify JWT token is being sent in handshake
3. **Scaling Issues**: Implement Redis adapter before deploying multiple instances
4. **Memory Leaks**: Properly clean up event listeners and clear caches
5. **High Latency**: Check network conditions and optimize payload sizes