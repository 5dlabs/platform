# EXAMPLE - Chat Application - Architecture & Implementation Plan

## System Architecture

### High-Level Overview
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React SPA     │    │   Node.js API   │    │   PostgreSQL    │
│   (Frontend)    │◄──►│   (Backend)     │◄──►│   (Database)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐               │
         └──────────────►│   Socket.io     │               │
                        │   (WebSockets)  │               │
                        └─────────────────┘               │
                                 │                        │
                        ┌─────────────────┐               │
                        │      Redis      │◄──────────────┘
                        │   (Sessions)    │
                        └─────────────────┘
```

## Component Architecture

### Frontend Components
```
src/
├── components/
│   ├── auth/
│   │   ├── LoginForm.tsx
│   │   ├── RegisterForm.tsx
│   │   └── ProfileModal.tsx
│   ├── chat/
│   │   ├── ChatRoom.tsx
│   │   ├── MessageList.tsx
│   │   ├── MessageInput.tsx
│   │   └── UserList.tsx
│   └── common/
│       ├── Layout.tsx
│       ├── Navbar.tsx
│       └── ThemeToggle.tsx
├── hooks/
│   ├── useSocket.ts
│   ├── useAuth.ts
│   └── useChat.ts
├── services/
│   ├── api.ts
│   ├── socket.ts
│   └── auth.ts
└── types/
    ├── user.ts
    ├── message.ts
    └── room.ts
```

### Backend Architecture
```
backend/
├── controllers/
│   ├── authController.js
│   ├── roomController.js
│   └── messageController.js
├── middleware/
│   ├── auth.js
│   ├── validation.js
│   └── rateLimiting.js
├── models/
│   ├── User.js
│   ├── Room.js
│   └── Message.js
├── routes/
│   ├── auth.js
│   ├── rooms.js
│   └── messages.js
├── sockets/
│   ├── chatHandlers.js
│   └── authHandlers.js
└── utils/
    ├── database.js
    ├── redis.js
    └── validators.js
```

## Data Models

### User Model
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(500),
    is_online BOOLEAN DEFAULT false,
    last_seen TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### Room Model
```sql
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_private BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### Message Model
```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    content TEXT NOT NULL,
    message_type VARCHAR(20) DEFAULT 'text',
    is_edited BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

## Real-time Communication

### Socket.io Events
```javascript
// Client -> Server
socket.emit('join-room', { roomId, userId });
socket.emit('send-message', { roomId, content, type });
socket.emit('typing-start', { roomId, userId });
socket.emit('typing-stop', { roomId, userId });

// Server -> Client
socket.emit('message-received', { message, user });
socket.emit('user-joined', { user, roomId });
socket.emit('user-left', { userId, roomId });
socket.emit('typing-indicator', { userId, roomId, isTyping });
```

## Security Implementation

### Authentication Flow
1. User registers/logs in with credentials
2. Server validates and returns JWT access token + refresh token
3. Client stores tokens (access in memory, refresh in httpOnly cookie)
4. API requests include access token in Authorization header
5. Socket.io authenticates using token on connection

### Authorization Matrix
```
Resource          | Owner | Member | Public | Admin
------------------|-------|--------|--------|-------
View Messages     |   ✓   |   ✓    |   ✗    |   ✓
Send Messages     |   ✓   |   ✓    |   ✗    |   ✓
Edit Messages     |   ✓   |   ✗    |   ✗    |   ✓
Delete Messages   |   ✓   |   ✗    |   ✗    |   ✓
Manage Room       |   ✓   |   ✗    |   ✗    |   ✓
```

## Performance Optimizations

### Caching Strategy
- **Redis**: User sessions, room membership, online status
- **Browser**: Static assets with aggressive caching
- **Database**: Connection pooling, query optimization

### Scalability Considerations
- Horizontal scaling with load balancer
- Socket.io Redis adapter for multi-instance support
- Database read replicas for scaling reads
- CDN for static asset delivery

## Development Workflow

### Phase 1: Core Infrastructure (Week 1-2)
- Set up development environment
- Database schema creation
- Basic Express server with authentication
- React app scaffolding with routing

### Phase 2: Authentication System (Week 2-3)
- User registration/login API
- JWT token management
- Frontend auth components
- Protected route implementation

### Phase 3: Real-time Chat (Week 3-4)
- Socket.io integration
- Basic chat room functionality
- Message persistence
- Real-time message delivery

### Phase 4: Advanced Features (Week 4-5)
- Typing indicators
- Online status
- File sharing
- Message editing/deletion

### Phase 5: Polish & Deploy (Week 5-6)
- UI/UX improvements
- Performance optimization
- Testing and bug fixes
- Production deployment

## Testing Strategy

### Unit Testing
- Jest for JavaScript/TypeScript
- React Testing Library for components
- Mocha/Chai for backend

### Integration Testing
- API endpoint testing
- Socket.io event testing
- Database integration tests

### E2E Testing
- Cypress for full user workflows
- Authentication flows
- Real-time messaging scenarios

## Deployment Architecture

### Production Environment
```
Load Balancer
      │
   ┌──▼──┐     ┌─────────┐
   │ App │     │ App     │
   │ #1  │     │ #2      │
   └──┬──┘     └─────┬───┘
      │              │
   ┌──▼──────────────▼──┐
   │  Redis Cluster     │
   └────────┬───────────┘
           │
   ┌───────▼────────┐
   │ PostgreSQL     │
   │ Primary/Replica│
   └────────────────┘
```

### Infrastructure Requirements
- **Application**: 2x 2GB RAM, 2 CPU instances
- **Database**: PostgreSQL with 16GB RAM, SSD storage
- **Cache**: Redis cluster with 4GB RAM
- **Load Balancer**: Nginx with SSL termination
- **Monitoring**: Prometheus + Grafana stack

## Success Metrics & Monitoring

### Key Performance Indicators
- Message delivery latency < 100ms (95th percentile)
- Concurrent user support > 1000
- System uptime > 99.9%
- Average session duration > 15 minutes

### Monitoring Points
- API response times and error rates
- WebSocket connection stability
- Database query performance
- Redis hit/miss ratios
- User engagement metrics

This architecture ensures scalability, security, and performance while maintaining development efficiency and operational simplicity.