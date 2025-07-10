# AI Agent Prompt: Implement Real-time Notification System

## Task Context

You are tasked with implementing a comprehensive real-time notification system for a microservice architecture. This system must deliver instant updates to users through multiple channels (email, push, in-app) using WebSocket technology for real-time communication.

## Primary Objective

Build a scalable, real-time notification system that includes:
- WebSocket-based real-time communication
- Multi-channel notification delivery (email, push, in-app)
- User preference management
- Reliable delivery mechanisms with retry logic
- Notification history and status tracking
- Integration with existing authentication system

## Technical Requirements

### Core Components to Implement

1. **WebSocket Infrastructure**
   - WebSocket server with Socket.IO or native WebSocket support
   - Connection management with authentication
   - Room-based message routing
   - Connection persistence and recovery
   - Load balancing across multiple WebSocket servers

2. **Notification Management System**
   - Database schema for notifications and preferences
   - Notification templates and content rendering
   - Status tracking (pending, delivered, read, failed)
   - Scheduling and batching capabilities
   - User preference management

3. **Multi-Channel Delivery Service**
   - In-app notifications via WebSocket
   - Email notifications via SMTP service
   - Push notifications via mobile push services
   - Delivery confirmation and tracking
   - Retry logic for failed deliveries

4. **Message Queue System**
   - Redis pub/sub for real-time message broadcasting
   - Queue processing for reliable delivery
   - Cross-server communication for scaling
   - Dead letter queue for failed messages

### Performance Requirements

- Support 10,000+ concurrent WebSocket connections
- Notification delivery latency < 100ms for real-time notifications
- Message queue throughput of 1,000+ notifications per second
- WebSocket connection recovery within 5 seconds
- Database queries optimized for notification retrieval

### Scalability Requirements

- Horizontal scaling with multiple WebSocket servers
- Redis adapter for Socket.IO clustering
- Database partitioning for notification storage
- Caching layer for user preferences and templates
- Auto-scaling based on connection load

## Implementation Approach

### Phase 1: WebSocket Infrastructure
1. Set up WebSocket server with authentication middleware
2. Implement connection management and room handling
3. Create message broadcasting capabilities
4. Add connection persistence and recovery logic
5. Set up load balancing for multiple server instances

### Phase 2: Database and Schema Design
1. Design notification and preference database schema
2. Create notification templates system
3. Implement user preference management
4. Set up notification history and status tracking
5. Add database indexing and optimization

### Phase 3: Multi-Channel Delivery
1. Implement in-app notification delivery via WebSocket
2. Create email notification service integration
3. Add push notification service integration
4. Build delivery confirmation and tracking
5. Implement retry logic for failed deliveries

### Phase 4: Message Queue Integration
1. Set up Redis pub/sub for message broadcasting
2. Create queue processing system
3. Implement cross-server communication
4. Add dead letter queue handling
5. Create monitoring and alerting for queue health

## Code Structure Expectations

```
src/
├── websocket/
│   ├── server.js
│   ├── auth.middleware.js
│   ├── connection.manager.js
│   └── room.manager.js
├── notifications/
│   ├── controllers/
│   │   ├── notification.controller.js
│   │   └── preference.controller.js
│   ├── services/
│   │   ├── notification.service.js
│   │   ├── delivery.service.js
│   │   └── template.service.js
│   ├── models/
│   │   ├── notification.model.js
│   │   ├── preference.model.js
│   │   └── template.model.js
│   └── handlers/
│       ├── email.handler.js
│       ├── push.handler.js
│       └── websocket.handler.js
├── queue/
│   ├── producer.js
│   ├── consumer.js
│   └── retry.manager.js
└── tests/
    ├── unit/
    ├── integration/
    └── performance/
```

## WebSocket Implementation Details

### Connection Authentication
```javascript
// Authentication middleware
io.use(async (socket, next) => {
  try {
    const token = socket.handshake.auth.token;
    const user = await verifyJWT(token);
    socket.userId = user.sub;
    socket.userRoles = user.roles;
    next();
  } catch (error) {
    next(new Error('Authentication failed'));
  }
});
```

### Room Management
```javascript
// Join user-specific room
socket.join(`user:${socket.userId}`);

// Subscribe to notification channels
socket.on('subscribe', async (data) => {
  const { channels } = data;
  for (const channel of channels) {
    socket.join(`channel:${channel}`);
  }
});
```

### Message Broadcasting
```javascript
// Send to specific user
io.to(`user:${userId}`).emit('notification', notification);

// Send to channel subscribers
io.to(`channel:${channel}`).emit('notification', notification);

// Broadcast system-wide
io.emit('notification', notification);
```

## Database Schema Requirements

### Notifications Table
```sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    channel VARCHAR(50) NOT NULL,
    priority notification_priority DEFAULT 'medium',
    status notification_status DEFAULT 'pending',
    scheduled_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delivered_at TIMESTAMP,
    read_at TIMESTAMP,
    data JSONB
);
```

### User Preferences Table
```sql
CREATE TABLE user_notification_preferences (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    email_enabled BOOLEAN DEFAULT true,
    push_enabled BOOLEAN DEFAULT true,
    in_app_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, channel, notification_type)
);
```

## API Specifications

### WebSocket Events

#### Client Events
- `subscribe`: Subscribe to notification channels
- `ack`: Acknowledge notification receipt
- `updatePreferences`: Update notification preferences
- `markAsRead`: Mark notification as read

#### Server Events
- `notification`: New notification delivery
- `notificationStatus`: Notification status update
- `connectionStatus`: Connection health status
- `error`: Error messages and handling

### HTTP API Endpoints

#### Notification Management
- `GET /api/notifications` - Get user notifications
- `POST /api/notifications` - Create new notification
- `PUT /api/notifications/:id/read` - Mark as read
- `DELETE /api/notifications/:id` - Delete notification

#### Preference Management
- `GET /api/notifications/preferences` - Get user preferences
- `PUT /api/notifications/preferences` - Update preferences
- `POST /api/notifications/preferences/bulk` - Bulk update preferences

## Message Queue Implementation

### Redis Pub/Sub Setup
```javascript
const publisher = Redis.createClient(process.env.REDIS_URL);
const subscriber = Redis.createClient(process.env.REDIS_URL);

// Subscribe to notification events
subscriber.subscribe('notifications:send');
subscriber.on('message', async (channel, message) => {
  const notification = JSON.parse(message);
  await processNotification(notification);
});
```

### Queue Processing
```javascript
async function processNotification(notification) {
  try {
    const preferences = await getUserPreferences(notification.userId);
    
    if (preferences.inApp) await sendInAppNotification(notification);
    if (preferences.email) await sendEmailNotification(notification);
    if (preferences.push) await sendPushNotification(notification);
    
    await updateNotificationStatus(notification.id, 'delivered');
  } catch (error) {
    await handleDeliveryFailure(notification, error);
  }
}
```

## Error Handling and Retry Logic

### Delivery Retry Strategy
```javascript
const RETRY_DELAYS = [1000, 5000, 30000, 300000]; // 1s, 5s, 30s, 5m

async function retryNotification(notification) {
  const deliveryLog = await getDeliveryLog(notification.id);
  
  if (deliveryLog.attemptCount < RETRY_DELAYS.length) {
    const delay = RETRY_DELAYS[deliveryLog.attemptCount];
    setTimeout(async () => {
      await incrementAttemptCount(notification.id);
      await processNotification(notification);
    }, delay);
  } else {
    await updateNotificationStatus(notification.id, 'failed');
  }
}
```

### Connection Recovery
```javascript
// Client-side reconnection logic
socket.on('disconnect', (reason) => {
  if (reason === 'io server disconnect') {
    socket.connect();
  }
});

socket.on('connect', () => {
  socket.emit('subscribe', { 
    userId: currentUser.id, 
    channels: subscribedChannels 
  });
  socket.emit('requestMissedNotifications', { 
    since: lastReceivedTimestamp 
  });
});
```

## Security Implementation

### Authentication and Authorization
- JWT token validation for WebSocket connections
- User-specific notification access control
- Channel subscription permission checks
- Rate limiting for notification sending

### Data Protection
- Encrypt sensitive notification content
- Sanitize user-generated notification data
- Implement proper CORS policies
- Use secure WebSocket connections (WSS)

## Testing Requirements

### Unit Tests (Minimum Coverage: 90%)
- WebSocket connection and authentication
- Notification creation and validation
- Template rendering and variable substitution
- User preference handling
- Delivery retry logic

### Integration Tests
- End-to-end notification delivery flow
- WebSocket connection and messaging
- Database operations and consistency
- Message queue processing
- External service integration

### Performance Tests
- Concurrent WebSocket connections (10,000+)
- Notification throughput under load
- Message queue performance
- Database query optimization
- Memory usage and leak detection

## Environment Configuration

Required environment variables:
```env
# WebSocket Configuration
WEBSOCKET_PORT=3001
WEBSOCKET_CORS_ORIGIN=https://app.example.com

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=secure_password

# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=notifications_db
DB_USER=notifications_user
DB_PASSWORD=secure_password

# External Services
EMAIL_SERVICE_URL=http://email-service:3000
EMAIL_SERVICE_API_KEY=email_api_key
PUSH_SERVICE_URL=http://push-service:3000
PUSH_SERVICE_API_KEY=push_api_key

# JWT Configuration
JWT_SECRET=your-jwt-secret
JWT_PUBLIC_KEY=your-public-key

# Notification Configuration
MAX_RETRY_ATTEMPTS=4
RETRY_DELAYS=1000,5000,30000,300000
NOTIFICATION_BATCH_SIZE=100
```

## Quality Assurance Checklist

Before marking this task complete, ensure:

- [ ] WebSocket server supports 10,000+ concurrent connections
- [ ] Authentication middleware properly validates JWT tokens
- [ ] Room-based message routing works correctly
- [ ] Connection recovery mechanisms are functional
- [ ] Database schema supports all notification requirements
- [ ] User preference management is complete
- [ ] Multi-channel delivery (email, push, in-app) works
- [ ] Retry logic handles delivery failures gracefully
- [ ] Message queue processes notifications reliably
- [ ] Cross-server communication works for scaling
- [ ] Security measures protect user data
- [ ] Performance requirements are met under load
- [ ] Tests achieve minimum 90% coverage
- [ ] Documentation is complete and accurate
- [ ] Monitoring and alerting are properly configured

## Success Metrics

- WebSocket connection stability > 99.9%
- Notification delivery latency < 100ms
- Message queue throughput > 1,000 notifications/second
- Zero critical security vulnerabilities
- Test coverage > 90%
- All integration tests passing
- Performance requirements met under expected load

## Important Notes

1. **Real-time Priority**: Focus on minimizing latency for real-time notifications
2. **Scalability**: Design for horizontal scaling from the start
3. **Reliability**: Implement robust error handling and retry mechanisms
4. **Security**: Ensure all WebSocket connections are authenticated and authorized
5. **Performance**: Optimize database queries and caching strategies
6. **Monitoring**: Implement comprehensive monitoring and alerting
7. **Testing**: Thorough testing of concurrent connections and edge cases

Begin implementation by setting up the WebSocket infrastructure and authentication. Focus on reliability and scalability throughout the development process.