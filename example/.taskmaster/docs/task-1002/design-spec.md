# Technical Design Specification: Real-time Notification System

## 1. System Architecture Overview

### 1.1 High-Level Architecture

The real-time notification system follows a microservice architecture with WebSocket-based real-time communication:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Apps   │    │   Load Balancer │    │  WebSocket      │
│  (Web/Mobile)   │◄──►│  (Sticky Sessions)│◄──►│  Servers (N)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                       ┌─────────────────┐             │
                       │   API Gateway   │◄────────────┤
                       │ (REST Endpoints)│             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  Redis Cluster  │◄────────────┤
                       │ (Pub/Sub/Cache) │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  Email Service  │◄────────────┤
                       │   (SMTP/API)    │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  Push Service   │◄────────────┤
                       │ (FCM/APN/etc)   │             │
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │  PostgreSQL     │◄────────────┘
                       │  (Notifications)│
                       └─────────────────┘
```

### 1.2 Component Responsibilities

- **WebSocket Servers**: Real-time communication, connection management, message broadcasting
- **Load Balancer**: Distribute connections across WebSocket servers with sticky sessions
- **API Gateway**: HTTP REST endpoints for notification management
- **Redis Cluster**: Pub/sub messaging, caching, cross-server communication
- **Email Service**: Email notification delivery via SMTP or API
- **Push Service**: Mobile push notification delivery (FCM, APN, etc.)
- **PostgreSQL**: Persistent storage for notifications, preferences, and audit logs

## 2. WebSocket Server Architecture

### 2.1 Connection Management

```javascript
// WebSocket Server Setup
const io = require('socket.io')(server, {
  cors: {
    origin: process.env.FRONTEND_URL,
    methods: ["GET", "POST"],
    credentials: true
  },
  transports: ['websocket', 'polling'],
  upgradeTimeout: 10000,
  pingTimeout: 5000,
  pingInterval: 25000
});

// Redis Adapter for Clustering
const { createAdapter } = require('@socket.io/redis-adapter');
const pubClient = redis.createClient(process.env.REDIS_URL);
const subClient = pubClient.duplicate();
io.adapter(createAdapter(pubClient, subClient));

// Authentication Middleware
io.use(async (socket, next) => {
  try {
    const token = socket.handshake.auth.token;
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    
    socket.userId = decoded.sub;
    socket.userRoles = decoded.roles;
    socket.userPermissions = decoded.permissions;
    
    next();
  } catch (error) {
    next(new Error('Authentication failed'));
  }
});
```

### 2.2 Room Management System

```javascript
class RoomManager {
  constructor(io) {
    this.io = io;
    this.userRooms = new Map(); // userId -> Set of rooms
    this.roomUsers = new Map(); // roomId -> Set of userIds
  }

  async joinUserRoom(socket) {
    const userRoom = `user:${socket.userId}`;
    socket.join(userRoom);
    this.addUserToRoom(socket.userId, userRoom);
    
    // Load user's notification preferences to join relevant channels
    const preferences = await this.getUserNotificationChannels(socket.userId);
    for (const channel of preferences) {
      const channelRoom = `channel:${channel}`;
      socket.join(channelRoom);
      this.addUserToRoom(socket.userId, channelRoom);
    }
  }

  async subscribeToChannels(socket, channels) {
    for (const channel of channels) {
      // Check if user has permission to subscribe to this channel
      if (await this.hasChannelPermission(socket.userId, channel)) {
        const channelRoom = `channel:${channel}`;
        socket.join(channelRoom);
        this.addUserToRoom(socket.userId, channelRoom);
      }
    }
  }

  leaveAllRooms(socket) {
    const userRooms = this.userRooms.get(socket.userId) || new Set();
    for (const room of userRooms) {
      socket.leave(room);
      this.removeUserFromRoom(socket.userId, room);
    }
  }

  addUserToRoom(userId, roomId) {
    if (!this.userRooms.has(userId)) {
      this.userRooms.set(userId, new Set());
    }
    this.userRooms.get(userId).add(roomId);

    if (!this.roomUsers.has(roomId)) {
      this.roomUsers.set(roomId, new Set());
    }
    this.roomUsers.get(roomId).add(userId);
  }

  removeUserFromRoom(userId, roomId) {
    if (this.userRooms.has(userId)) {
      this.userRooms.get(userId).delete(roomId);
    }
    if (this.roomUsers.has(roomId)) {
      this.roomUsers.get(roomId).delete(userId);
    }
  }
}
```

### 2.3 Connection Event Handlers

```javascript
io.on('connection', (socket) => {
  const roomManager = new RoomManager(io);
  
  console.log(`User ${socket.userId} connected`);
  
  // Join user-specific room and channel rooms
  roomManager.joinUserRoom(socket);
  
  // Handle channel subscription
  socket.on('subscribe', async (data) => {
    const { channels } = data;
    await roomManager.subscribeToChannels(socket, channels);
    socket.emit('subscribed', { channels });
  });
  
  // Handle notification acknowledgment
  socket.on('ack', async (data) => {
    const { notificationId } = data;
    await markNotificationAsDelivered(notificationId, socket.userId);
  });
  
  // Handle marking notification as read
  socket.on('markAsRead', async (data) => {
    const { notificationId } = data;
    await markNotificationAsRead(notificationId, socket.userId);
  });
  
  // Handle preference updates
  socket.on('updatePreferences', async (data) => {
    await updateUserPreferences(socket.userId, data);
    socket.emit('preferencesUpdated', { success: true });
  });
  
  // Handle requesting missed notifications
  socket.on('requestMissedNotifications', async (data) => {
    const { since } = data;
    const missedNotifications = await getMissedNotifications(socket.userId, since);
    socket.emit('missedNotifications', { notifications: missedNotifications });
  });
  
  // Handle disconnection
  socket.on('disconnect', (reason) => {
    console.log(`User ${socket.userId} disconnected: ${reason}`);
    roomManager.leaveAllRooms(socket);
  });
  
  // Handle connection errors
  socket.on('error', (error) => {
    console.error(`Socket error for user ${socket.userId}:`, error);
  });
});
```

## 3. Database Design

### 3.1 Notification Tables

```sql
-- Notification types enum
CREATE TYPE notification_type AS ENUM (
  'system', 'user_action', 'order_update', 'message', 'reminder', 
  'security', 'marketing', 'social', 'payment', 'custom'
);

-- Notification priority enum
CREATE TYPE notification_priority AS ENUM ('low', 'medium', 'high', 'urgent');

-- Notification status enum
CREATE TYPE notification_status AS ENUM ('pending', 'delivered', 'read', 'failed');

-- Main notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type notification_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    channel VARCHAR(50) NOT NULL,
    priority notification_priority DEFAULT 'medium',
    status notification_status DEFAULT 'pending',
    scheduled_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delivered_at TIMESTAMP,
    read_at TIMESTAMP,
    expires_at TIMESTAMP,
    data JSONB,
    
    -- Indexes
    CONSTRAINT notifications_user_id_idx
);

-- Indexes for performance
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_user_status ON notifications(user_id, status);
CREATE INDEX idx_notifications_scheduled ON notifications(scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX idx_notifications_expires ON notifications(expires_at) WHERE expires_at IS NOT NULL;
```

### 3.2 Notification Templates

```sql
-- Notification templates
CREATE TABLE notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(50) NOT NULL,
    channel VARCHAR(50) NOT NULL,
    language VARCHAR(10) DEFAULT 'en',
    subject_template TEXT NOT NULL,
    content_template TEXT NOT NULL,
    variables JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(type, channel, language)
);

-- Template variables for dynamic content
CREATE TABLE template_variables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES notification_templates(id) ON DELETE CASCADE,
    variable_name VARCHAR(100) NOT NULL,
    variable_type VARCHAR(50) NOT NULL,
    default_value TEXT,
    is_required BOOLEAN DEFAULT false,
    description TEXT,
    
    UNIQUE(template_id, variable_name)
);
```

### 3.3 User Preferences

```sql
-- User notification preferences
CREATE TABLE user_notification_preferences (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL,
    notification_type notification_type NOT NULL,
    email_enabled BOOLEAN DEFAULT true,
    push_enabled BOOLEAN DEFAULT true,
    in_app_enabled BOOLEAN DEFAULT true,
    frequency VARCHAR(20) DEFAULT 'immediate', -- immediate, hourly, daily, weekly
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, channel, notification_type)
);

-- User global preferences
CREATE TABLE user_global_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_notifications_enabled BOOLEAN DEFAULT true,
    push_notifications_enabled BOOLEAN DEFAULT true,
    in_app_notifications_enabled BOOLEAN DEFAULT true,
    marketing_notifications_enabled BOOLEAN DEFAULT false,
    digest_frequency VARCHAR(20) DEFAULT 'daily',
    digest_time TIME DEFAULT '09:00:00',
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 3.4 Delivery Tracking

```sql
-- Notification delivery log
CREATE TABLE notification_delivery_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    channel VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    provider VARCHAR(50),
    provider_message_id VARCHAR(255),
    error_code VARCHAR(50),
    error_message TEXT,
    attempt_count INTEGER DEFAULT 1,
    delivered_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_delivery_log_notification_id (notification_id),
    INDEX idx_delivery_log_status (status),
    INDEX idx_delivery_log_created_at (created_at)
);

-- Notification engagement tracking
CREATE TABLE notification_engagement (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL, -- delivered, read, clicked, dismissed
    channel VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB,
    
    INDEX idx_engagement_notification_id (notification_id),
    INDEX idx_engagement_user_id (user_id),
    INDEX idx_engagement_action (action)
);
```

## 4. Message Queue System

### 4.1 Redis Pub/Sub Architecture

```javascript
class NotificationQueue {
  constructor() {
    this.publisher = redis.createClient(process.env.REDIS_URL);
    this.subscriber = redis.createClient(process.env.REDIS_URL);
    this.setupSubscriptions();
  }

  setupSubscriptions() {
    // Subscribe to notification channels
    this.subscriber.subscribe('notifications:send');
    this.subscriber.subscribe('notifications:retry');
    this.subscriber.subscribe('notifications:failed');
    
    this.subscriber.on('message', (channel, message) => {
      this.handleMessage(channel, message);
    });
  }

  async handleMessage(channel, message) {
    try {
      const data = JSON.parse(message);
      
      switch (channel) {
        case 'notifications:send':
          await this.processNotification(data);
          break;
        case 'notifications:retry':
          await this.retryNotification(data);
          break;
        case 'notifications:failed':
          await this.handleFailedNotification(data);
          break;
      }
    } catch (error) {
      console.error(`Error handling message from ${channel}:`, error);
    }
  }

  async queueNotification(notification) {
    await this.publisher.publish('notifications:send', JSON.stringify(notification));
  }

  async queueRetry(notification) {
    await this.publisher.publish('notifications:retry', JSON.stringify(notification));
  }

  async queueFailure(notification) {
    await this.publisher.publish('notifications:failed', JSON.stringify(notification));
  }
}
```

### 4.2 Notification Processing Pipeline

```javascript
class NotificationProcessor {
  constructor(queue, deliveryService) {
    this.queue = queue;
    this.deliveryService = deliveryService;
    this.retryDelays = [1000, 5000, 30000, 300000, 1800000]; // 1s, 5s, 30s, 5m, 30m
  }

  async processNotification(notification) {
    try {
      // Check if notification is scheduled for future delivery
      if (notification.scheduledAt && new Date(notification.scheduledAt) > new Date()) {
        await this.scheduleNotification(notification);
        return;
      }

      // Get user preferences
      const preferences = await this.getUserPreferences(notification.userId);
      
      // Filter based on user preferences
      if (!this.shouldDeliverNotification(notification, preferences)) {
        await this.markNotificationAsSkipped(notification.id);
        return;
      }

      // Deliver through enabled channels
      const deliveryPromises = [];
      
      if (preferences.inApp) {
        deliveryPromises.push(this.deliverInApp(notification));
      }
      
      if (preferences.email) {
        deliveryPromises.push(this.deliverEmail(notification));
      }
      
      if (preferences.push) {
        deliveryPromises.push(this.deliverPush(notification));
      }

      // Wait for all deliveries to complete
      const results = await Promise.allSettled(deliveryPromises);
      
      // Process delivery results
      await this.processDeliveryResults(notification, results);
      
    } catch (error) {
      console.error(`Error processing notification ${notification.id}:`, error);
      await this.handleProcessingError(notification, error);
    }
  }

  async retryNotification(notification) {
    const deliveryLog = await this.getDeliveryLog(notification.id);
    
    if (deliveryLog.attemptCount >= this.retryDelays.length) {
      await this.markNotificationAsFailed(notification.id);
      await this.queue.queueFailure(notification);
      return;
    }

    const delay = this.retryDelays[deliveryLog.attemptCount];
    
    setTimeout(async () => {
      await this.incrementAttemptCount(notification.id);
      await this.processNotification(notification);
    }, delay);
  }

  shouldDeliverNotification(notification, preferences) {
    // Check global preferences
    if (!preferences.global[`${notification.channel}_enabled`]) {
      return false;
    }

    // Check specific type preferences
    const typePreference = preferences.types[notification.type];
    if (!typePreference) {
      return false;
    }

    // Check quiet hours
    if (this.isInQuietHours(preferences.quietHours)) {
      return notification.priority === 'urgent';
    }

    return true;
  }

  async deliverInApp(notification) {
    // Send via WebSocket
    io.to(`user:${notification.userId}`).emit('notification', {
      id: notification.id,
      type: notification.type,
      title: notification.title,
      message: notification.message,
      channel: notification.channel,
      priority: notification.priority,
      timestamp: new Date().toISOString(),
      data: notification.data
    });

    await this.logDelivery(notification.id, 'in_app', 'delivered');
  }

  async deliverEmail(notification) {
    const template = await this.getEmailTemplate(notification.type);
    const content = await this.renderTemplate(template, notification);
    
    const emailData = {
      to: await this.getUserEmail(notification.userId),
      subject: content.subject,
      html: content.html,
      text: content.text
    };

    const result = await this.deliveryService.sendEmail(emailData);
    await this.logDelivery(notification.id, 'email', 'delivered', result.messageId);
  }

  async deliverPush(notification) {
    const pushTokens = await this.getUserPushTokens(notification.userId);
    
    const pushData = {
      tokens: pushTokens,
      title: notification.title,
      body: notification.message,
      data: notification.data
    };

    const result = await this.deliveryService.sendPush(pushData);
    await this.logDelivery(notification.id, 'push', 'delivered', result.messageId);
  }
}
```

## 5. Delivery Service Implementation

### 5.1 Email Delivery Service

```javascript
class EmailDeliveryService {
  constructor() {
    this.transporter = nodemailer.createTransporter({
      host: process.env.SMTP_HOST,
      port: process.env.SMTP_PORT,
      secure: process.env.SMTP_SECURE === 'true',
      auth: {
        user: process.env.SMTP_USER,
        pass: process.env.SMTP_PASSWORD
      }
    });
  }

  async sendEmail(emailData) {
    const mailOptions = {
      from: process.env.FROM_EMAIL,
      to: emailData.to,
      subject: emailData.subject,
      html: emailData.html,
      text: emailData.text
    };

    try {
      const result = await this.transporter.sendMail(mailOptions);
      return { success: true, messageId: result.messageId };
    } catch (error) {
      console.error('Email delivery failed:', error);
      throw new Error(`Email delivery failed: ${error.message}`);
    }
  }

  async verifyConnection() {
    try {
      await this.transporter.verify();
      return true;
    } catch (error) {
      console.error('Email service connection failed:', error);
      return false;
    }
  }
}
```

### 5.2 Push Notification Service

```javascript
class PushDeliveryService {
  constructor() {
    this.fcm = admin.messaging();
  }

  async sendPush(pushData) {
    const message = {
      notification: {
        title: pushData.title,
        body: pushData.body
      },
      data: pushData.data || {},
      tokens: pushData.tokens
    };

    try {
      const result = await this.fcm.sendMulticast(message);
      
      // Handle partial failures
      if (result.failureCount > 0) {
        await this.handlePartialFailures(result, pushData.tokens);
      }
      
      return { 
        success: true, 
        successCount: result.successCount,
        failureCount: result.failureCount
      };
    } catch (error) {
      console.error('Push notification delivery failed:', error);
      throw new Error(`Push delivery failed: ${error.message}`);
    }
  }

  async handlePartialFailures(result, tokens) {
    result.responses.forEach((response, index) => {
      if (!response.success) {
        const token = tokens[index];
        console.error(`Push failed for token ${token}:`, response.error);
        
        // Handle invalid tokens
        if (response.error.code === 'messaging/invalid-registration-token') {
          this.removeInvalidToken(token);
        }
      }
    });
  }
}
```

## 6. Template System

### 6.1 Template Rendering Engine

```javascript
class TemplateEngine {
  constructor() {
    this.handlebars = require('handlebars');
    this.registerHelpers();
  }

  registerHelpers() {
    // Date formatting helper
    this.handlebars.registerHelper('formatDate', (date, format) => {
      return moment(date).format(format);
    });

    // Conditional helper
    this.handlebars.registerHelper('ifEquals', (arg1, arg2, options) => {
      return (arg1 === arg2) ? options.fn(this) : options.inverse(this);
    });

    // URL helper
    this.handlebars.registerHelper('url', (path) => {
      return `${process.env.FRONTEND_URL}${path}`;
    });
  }

  async renderTemplate(template, data) {
    try {
      const subjectTemplate = this.handlebars.compile(template.subjectTemplate);
      const contentTemplate = this.handlebars.compile(template.contentTemplate);
      
      const templateData = {
        ...data,
        user: await this.getUserData(data.userId),
        app: {
          name: process.env.APP_NAME,
          url: process.env.FRONTEND_URL,
          support: process.env.SUPPORT_EMAIL
        }
      };

      return {
        subject: subjectTemplate(templateData),
        html: contentTemplate(templateData),
        text: this.htmlToText(contentTemplate(templateData))
      };
    } catch (error) {
      console.error('Template rendering failed:', error);
      throw new Error(`Template rendering failed: ${error.message}`);
    }
  }

  htmlToText(html) {
    return html.replace(/<[^>]*>/g, '');
  }
}
```

### 6.2 Template Management

```javascript
class TemplateManager {
  constructor() {
    this.cache = new Map();
    this.cacheTimeout = 5 * 60 * 1000; // 5 minutes
  }

  async getTemplate(type, channel, language = 'en') {
    const cacheKey = `${type}:${channel}:${language}`;
    
    if (this.cache.has(cacheKey)) {
      const cached = this.cache.get(cacheKey);
      if (Date.now() - cached.timestamp < this.cacheTimeout) {
        return cached.template;
      }
    }

    const template = await this.loadTemplate(type, channel, language);
    
    this.cache.set(cacheKey, {
      template,
      timestamp: Date.now()
    });

    return template;
  }

  async loadTemplate(type, channel, language) {
    const query = `
      SELECT * FROM notification_templates 
      WHERE type = $1 AND channel = $2 AND language = $3 AND is_active = true
    `;
    
    const result = await db.query(query, [type, channel, language]);
    
    if (result.rows.length === 0) {
      // Fallback to default language
      if (language !== 'en') {
        return this.loadTemplate(type, channel, 'en');
      }
      throw new Error(`Template not found: ${type}:${channel}:${language}`);
    }

    return result.rows[0];
  }

  async createTemplate(templateData) {
    const query = `
      INSERT INTO notification_templates (type, channel, language, subject_template, content_template, variables)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
    `;
    
    const result = await db.query(query, [
      templateData.type,
      templateData.channel,
      templateData.language,
      templateData.subjectTemplate,
      templateData.contentTemplate,
      templateData.variables
    ]);

    // Clear cache
    this.cache.clear();

    return result.rows[0];
  }
}
```

## 7. Performance Optimization

### 7.1 Connection Pool Management

```javascript
class ConnectionPool {
  constructor(maxConnections = 10000) {
    this.maxConnections = maxConnections;
    this.connections = new Map();
    this.connectionCount = 0;
    this.setupMonitoring();
  }

  addConnection(socketId, socket) {
    if (this.connectionCount >= this.maxConnections) {
      throw new Error('Maximum connections reached');
    }

    this.connections.set(socketId, {
      socket,
      userId: socket.userId,
      connectedAt: Date.now(),
      lastActivity: Date.now()
    });

    this.connectionCount++;
    this.updateMetrics();
  }

  removeConnection(socketId) {
    if (this.connections.has(socketId)) {
      this.connections.delete(socketId);
      this.connectionCount--;
      this.updateMetrics();
    }
  }

  updateActivity(socketId) {
    const connection = this.connections.get(socketId);
    if (connection) {
      connection.lastActivity = Date.now();
    }
  }

  getConnectionStats() {
    return {
      totalConnections: this.connectionCount,
      maxConnections: this.maxConnections,
      utilizationPercent: (this.connectionCount / this.maxConnections) * 100
    };
  }

  setupMonitoring() {
    // Monitor connection health every 30 seconds
    setInterval(() => {
      this.cleanupStaleConnections();
      this.reportMetrics();
    }, 30000);
  }

  cleanupStaleConnections() {
    const staleThreshold = 5 * 60 * 1000; // 5 minutes
    const now = Date.now();

    for (const [socketId, connection] of this.connections) {
      if (now - connection.lastActivity > staleThreshold) {
        connection.socket.disconnect();
        this.removeConnection(socketId);
      }
    }
  }
}
```

### 7.2 Caching Strategy

```javascript
class NotificationCache {
  constructor() {
    this.redis = redis.createClient(process.env.REDIS_URL);
    this.defaultTTL = 3600; // 1 hour
  }

  async cacheUserPreferences(userId, preferences) {
    const key = `user:${userId}:preferences`;
    await this.redis.setex(key, this.defaultTTL, JSON.stringify(preferences));
  }

  async getUserPreferences(userId) {
    const key = `user:${userId}:preferences`;
    const cached = await this.redis.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  async cacheNotificationTemplate(type, channel, language, template) {
    const key = `template:${type}:${channel}:${language}`;
    await this.redis.setex(key, this.defaultTTL * 6, JSON.stringify(template));
  }

  async getNotificationTemplate(type, channel, language) {
    const key = `template:${type}:${channel}:${language}`;
    const cached = await this.redis.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  async cacheDeliveryStatus(notificationId, status) {
    const key = `delivery:${notificationId}:status`;
    await this.redis.setex(key, 3600, status);
  }

  async getDeliveryStatus(notificationId) {
    const key = `delivery:${notificationId}:status`;
    return await this.redis.get(key);
  }

  async invalidateUserCache(userId) {
    const keys = await this.redis.keys(`user:${userId}:*`);
    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
  }
}
```

## 8. Monitoring and Metrics

### 8.1 Metrics Collection

```javascript
const prometheus = require('prom-client');

const notificationMetrics = {
  // Connection metrics
  activeConnections: new prometheus.Gauge({
    name: 'websocket_active_connections',
    help: 'Number of active WebSocket connections'
  }),

  // Notification metrics
  notificationsSent: new prometheus.Counter({
    name: 'notifications_sent_total',
    help: 'Total number of notifications sent',
    labelNames: ['type', 'channel', 'status']
  }),

  notificationDeliveryTime: new prometheus.Histogram({
    name: 'notification_delivery_time_seconds',
    help: 'Time taken to deliver notifications',
    labelNames: ['channel']
  }),

  // Queue metrics
  queueSize: new prometheus.Gauge({
    name: 'notification_queue_size',
    help: 'Number of notifications in queue'
  }),

  // Error metrics
  deliveryErrors: new prometheus.Counter({
    name: 'notification_delivery_errors_total',
    help: 'Total number of delivery errors',
    labelNames: ['channel', 'error_type']
  })
};

class MetricsCollector {
  constructor() {
    this.metrics = notificationMetrics;
    this.startCollection();
  }

  recordNotificationSent(type, channel, status) {
    this.metrics.notificationsSent.inc({ type, channel, status });
  }

  recordDeliveryTime(channel, duration) {
    this.metrics.notificationDeliveryTime.observe({ channel }, duration);
  }

  recordDeliveryError(channel, errorType) {
    this.metrics.deliveryErrors.inc({ channel, error_type: errorType });
  }

  updateActiveConnections(count) {
    this.metrics.activeConnections.set(count);
  }

  updateQueueSize(size) {
    this.metrics.queueSize.set(size);
  }

  startCollection() {
    // Collect metrics every 30 seconds
    setInterval(() => {
      this.collectSystemMetrics();
    }, 30000);
  }

  async collectSystemMetrics() {
    // Collect active connections
    const connectionCount = await this.getActiveConnectionCount();
    this.updateActiveConnections(connectionCount);

    // Collect queue size
    const queueSize = await this.getQueueSize();
    this.updateQueueSize(queueSize);
  }
}
```

### 8.2 Health Checks

```javascript
class HealthChecker {
  constructor() {
    this.checks = {
      database: this.checkDatabase.bind(this),
      redis: this.checkRedis.bind(this),
      websocket: this.checkWebSocket.bind(this),
      email: this.checkEmailService.bind(this),
      push: this.checkPushService.bind(this)
    };
  }

  async checkAll() {
    const results = {};
    
    for (const [name, checkFn] of Object.entries(this.checks)) {
      try {
        const result = await Promise.race([
          checkFn(),
          new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout')), 5000))
        ]);
        results[name] = { status: 'healthy', ...result };
      } catch (error) {
        results[name] = { status: 'unhealthy', error: error.message };
      }
    }

    const overallStatus = Object.values(results).every(r => r.status === 'healthy') 
      ? 'healthy' : 'unhealthy';

    return { status: overallStatus, checks: results };
  }

  async checkDatabase() {
    const result = await db.query('SELECT 1 as health');
    return { response_time: Date.now() };
  }

  async checkRedis() {
    const start = Date.now();
    await redis.ping();
    return { response_time: Date.now() - start };
  }

  async checkWebSocket() {
    const connectionCount = await this.getActiveConnectionCount();
    return { active_connections: connectionCount };
  }

  async checkEmailService() {
    const start = Date.now();
    const isConnected = await emailService.verifyConnection();
    return { connected: isConnected, response_time: Date.now() - start };
  }

  async checkPushService() {
    // Simple ping to push service
    return { status: 'connected' };
  }
}
```

## 9. Security Implementation

### 9.1 Authentication and Authorization

```javascript
class SecurityManager {
  constructor() {
    this.jwtSecret = process.env.JWT_SECRET;
    this.rateLimiter = new Map();
  }

  async authenticateWebSocket(socket, next) {
    try {
      const token = socket.handshake.auth.token;
      
      if (!token) {
        return next(new Error('No token provided'));
      }

      const decoded = jwt.verify(token, this.jwtSecret);
      
      // Check if token is blacklisted
      const isBlacklisted = await this.isTokenBlacklisted(token);
      if (isBlacklisted) {
        return next(new Error('Token is blacklisted'));
      }

      socket.userId = decoded.sub;
      socket.userRoles = decoded.roles;
      socket.token = token;
      
      next();
    } catch (error) {
      next(new Error('Authentication failed'));
    }
  }

  async authorizeChannelAccess(userId, channel) {
    const user = await this.getUserWithRoles(userId);
    
    // Check if user has permission to access this channel
    const hasPermission = await this.checkChannelPermission(user, channel);
    
    return hasPermission;
  }

  async checkChannelPermission(user, channel) {
    // Define channel access rules
    const channelRules = {
      'public': () => true,
      'user': (user) => user.id === user.id,
      'admin': (user) => user.roles.includes('admin'),
      'orders': (user) => user.permissions.includes('read:orders'),
      'system': (user) => user.roles.includes('admin') || user.roles.includes('moderator')
    };

    const rule = channelRules[channel];
    return rule ? rule(user) : false;
  }

  async rateLimit(userId, action, limit = 100, windowMs = 60000) {
    const key = `${userId}:${action}`;
    const now = Date.now();
    
    if (!this.rateLimiter.has(key)) {
      this.rateLimiter.set(key, { count: 0, window: now });
    }

    const data = this.rateLimiter.get(key);
    
    // Reset counter if window has passed
    if (now - data.window > windowMs) {
      data.count = 0;
      data.window = now;
    }

    data.count++;
    
    if (data.count > limit) {
      throw new Error('Rate limit exceeded');
    }

    return true;
  }

  async isTokenBlacklisted(token) {
    const key = `blacklist:${token}`;
    const result = await redis.get(key);
    return result !== null;
  }

  async blacklistToken(token) {
    const key = `blacklist:${token}`;
    // Set expiration to match token expiration
    await redis.setex(key, 3600, 'blacklisted');
  }
}
```

### 9.2 Input Validation

```javascript
const Joi = require('joi');

class ValidationSchemas {
  static notification = Joi.object({
    type: Joi.string().valid('system', 'user_action', 'order_update', 'message', 'reminder').required(),
    title: Joi.string().max(255).required(),
    message: Joi.string().max(1000).required(),
    channel: Joi.string().max(50).required(),
    priority: Joi.string().valid('low', 'medium', 'high', 'urgent').default('medium'),
    scheduledAt: Joi.date().iso().optional(),
    data: Joi.object().optional()
  });

  static preferences = Joi.object({
    email: Joi.boolean().default(true),
    push: Joi.boolean().default(true),
    inApp: Joi.boolean().default(true),
    channels: Joi.object().pattern(
      Joi.string(),
      Joi.object({
        email: Joi.boolean(),
        push: Joi.boolean(),
        inApp: Joi.boolean()
      })
    ).optional()
  });

  static subscription = Joi.object({
    channels: Joi.array().items(Joi.string().max(50)).required()
  });
}

function validateInput(schema) {
  return (req, res, next) => {
    const { error } = schema.validate(req.body);
    if (error) {
      return res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: error.details[0].message
        }
      });
    }
    next();
  };
}
```

## 10. Error Handling and Recovery

### 10.1 Error Classification

```javascript
class NotificationError extends Error {
  constructor(message, code, type, retryable = false) {
    super(message);
    this.name = 'NotificationError';
    this.code = code;
    this.type = type;
    this.retryable = retryable;
  }
}

class ErrorHandler {
  static handleDeliveryError(error, notification) {
    const errorMap = {
      'SMTP_ERROR': new NotificationError('SMTP server error', 'SMTP_ERROR', 'email', true),
      'INVALID_EMAIL': new NotificationError('Invalid email address', 'INVALID_EMAIL', 'email', false),
      'PUSH_TOKEN_INVALID': new NotificationError('Invalid push token', 'PUSH_TOKEN_INVALID', 'push', false),
      'RATE_LIMIT_EXCEEDED': new NotificationError('Rate limit exceeded', 'RATE_LIMIT_EXCEEDED', 'system', true),
      'WEBSOCKET_DISCONNECTED': new NotificationError('WebSocket disconnected', 'WEBSOCKET_DISCONNECTED', 'websocket', true)
    };

    return errorMap[error.code] || new NotificationError(error.message, 'UNKNOWN_ERROR', 'system', false);
  }

  static async handleError(error, notification) {
    const notificationError = this.handleDeliveryError(error, notification);
    
    // Log error
    await this.logError(notification.id, notificationError);
    
    // Determine if retry is warranted
    if (notificationError.retryable) {
      await this.scheduleRetry(notification);
    } else {
      await this.markAsFailed(notification.id);
    }
  }
}
```

### 10.2 Circuit Breaker Pattern

```javascript
class CircuitBreaker {
  constructor(threshold = 5, timeout = 60000) {
    this.threshold = threshold;
    this.timeout = timeout;
    this.failureCount = 0;
    this.lastFailureTime = null;
    this.state = 'CLOSED'; // CLOSED, OPEN, HALF_OPEN
  }

  async execute(fn) {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime > this.timeout) {
        this.state = 'HALF_OPEN';
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  onSuccess() {
    this.failureCount = 0;
    this.state = 'CLOSED';
  }

  onFailure() {
    this.failureCount++;
    this.lastFailureTime = Date.now();
    
    if (this.failureCount >= this.threshold) {
      this.state = 'OPEN';
    }
  }

  getState() {
    return this.state;
  }
}
```

This comprehensive technical design specification provides a solid foundation for implementing a scalable, secure, and reliable real-time notification system with multi-channel delivery capabilities.