# Autonomous AI Agent Prompt for Task 5: Real-time Communication with Socket.io

## Objective

Implement a comprehensive real-time communication system using Socket.io that enables instant messaging, typing indicators, user presence tracking, and read receipts. The system must support horizontal scaling through Redis, maintain sub-100ms message delivery latency, and provide secure WebSocket connections with JWT authentication.

## Context

You are implementing the real-time communication layer for a collaborative platform. This system must handle thousands of concurrent connections, support multiple chat rooms, and provide real-time updates for various user interactions. The implementation should be production-ready with proper error handling, reconnection strategies, and performance optimizations.

## Requirements

### Core Functionality

1. **Socket.io Server Setup**
   - Configure Socket.io server with appropriate CORS settings
   - Implement both WebSocket and polling transports
   - Set up proper ping/pong intervals for connection health
   - Configure maximum buffer sizes and timeout values

2. **Authentication System**
   - Implement JWT-based socket authentication middleware
   - Validate tokens on connection and reject unauthorized attempts
   - Store user information in socket instance for event handlers
   - Join authenticated users to their personal notification rooms

3. **Real-time Events**
   - **Room Management**: join-room, leave-room events
   - **Messaging**: send-message, edit-message, delete-message
   - **Read Receipts**: mark-as-read with batching support
   - **Typing Indicators**: typing-start, typing-stop with auto-timeout
   - **User Presence**: online/offline status, last seen timestamps

4. **Redis Integration**
   - Configure Redis adapter for Socket.io clustering
   - Implement pub/sub for cross-server communication
   - Set up connection retry strategies
   - Handle Redis connection failures gracefully

5. **Performance Optimization**
   - Achieve sub-100ms message delivery latency
   - Implement message payload compression
   - Use event batching for similar events
   - Cache frequently accessed data
   - Minimize payload sizes with optimized data structures

6. **Security Measures**
   - Implement rate limiting per user and IP
   - Sanitize all user inputs to prevent XSS
   - Add CSRF protection for socket connections
   - Maintain IP blacklists for abuse prevention
   - Validate all incoming event data

### Technical Specifications

#### Socket Events

```typescript
// Client to Server Events
interface ClientToServerEvents {
  'join-room': (roomId: string) => void;
  'leave-room': (roomId: string) => void;
  'send-message': (data: { roomId: string; content: string; attachments?: any[] }) => void;
  'edit-message': (data: { messageId: string; content: string }) => void;
  'delete-message': (messageId: string) => void;
  'mark-as-read': (data: { messageIds: string[]; roomId: string }) => void;
  'typing-start': (roomId: string) => void;
  'typing-stop': (roomId: string) => void;
  'update-presence': (status: 'online' | 'away' | 'busy' | 'offline') => void;
  'get-room-presence': (roomId: string) => void;
}

// Server to Client Events
interface ServerToClientEvents {
  'new-message': (message: MessagePayload) => void;
  'message-edited': (data: { messageId: string; content: string; editedAt: Date }) => void;
  'message-deleted': (messageId: string) => void;
  'messages-read': (data: { messageIds: string[]; userId: string; readAt: Date }) => void;
  'user-typing': (data: { roomId: string; userId: string; username: string; isTyping: boolean }) => void;
  'presence-update': (data: { userId: string; status: string; lastSeen?: Date }) => void;
  'room-presence': (data: { roomId: string; users: PresenceInfo[] }) => void;
  'error': (error: { message: string; code?: string }) => void;
}
```

#### Performance Requirements

- Message delivery latency: < 100ms (95th percentile)
- Connection establishment: < 2 seconds
- Typing indicator latency: < 50ms
- Presence update latency: < 200ms
- Support for 10,000+ concurrent connections per server
- Memory usage: < 100MB for 1,000 active connections

#### Security Requirements

- All connections must be authenticated via JWT
- Rate limits: 30 messages per minute per user
- Connection limits: 5 connections per IP address
- Message size limit: 1MB including attachments
- Automatic disconnection after 10 failed auth attempts

## Implementation Steps

1. **Setup Phase**
   - Install required packages: socket.io, @socket.io/redis-adapter, redis, jsonwebtoken
   - Create Socket.io server configuration with optimal settings
   - Set up Redis clients for pub/sub and adapter

2. **Authentication Implementation**
   - Create JWT validation middleware
   - Implement user session management
   - Add connection lifecycle handlers

3. **Event Handler Development**
   - Implement message handlers with validation
   - Create typing indicator system with timers
   - Build presence tracking with Redis
   - Add room management functionality

4. **Optimization Phase**
   - Implement compression middleware
   - Add event batching system
   - Create caching layer for hot data
   - Optimize payload structures

5. **Security Implementation**
   - Add rate limiting per user and IP
   - Implement input sanitization
   - Create CSRF protection
   - Build abuse prevention system

6. **Testing and Monitoring**
   - Write unit tests for all handlers
   - Create performance benchmarks
   - Implement monitoring endpoints
   - Add error tracking

## Expected Deliverables

1. **Server Implementation**
   - `src/socket/server.ts` - Main Socket.io server setup
   - `src/socket/middleware/auth.ts` - Authentication middleware
   - `src/socket/handlers/` - Event handler modules
   - `src/socket/config/redis.ts` - Redis configuration

2. **Security Layer**
   - `src/socket/security/` - Security implementations
   - Rate limiting configuration
   - Input validation schemas

3. **Performance Optimizations**
   - `src/socket/optimizations/` - Performance enhancements
   - Caching strategies
   - Payload optimization

4. **Tests**
   - Unit tests for all event handlers
   - Integration tests for Socket.io flows
   - Performance benchmarks
   - Security test suite

5. **Documentation**
   - API documentation for all events
   - Deployment guide
   - Performance tuning guide
   - Troubleshooting guide

## Success Criteria

- All socket events functioning correctly with proper error handling
- JWT authentication working for all connections
- Redis adapter configured and tested for horizontal scaling
- Message delivery latency consistently under 100ms
- All security measures implemented and tested
- Comprehensive test coverage (>80%)
- No memory leaks under sustained load
- Graceful handling of network interruptions

## Additional Considerations

- Implement graceful shutdown procedures
- Add metrics collection for monitoring
- Create client SDK for easier frontend integration
- Document all events and their payloads
- Prepare for geographic distribution with multiple Redis instances
- Consider implementing message history pagination
- Plan for binary data transmission (file uploads)
- Design for offline message queuing