# Task 2: Database Setup and Schema Design

## Overview
Implement a robust database architecture using PostgreSQL for persistent data storage and Redis for session management and real-time features. This task establishes the data foundation for the chat application with proper schema design, optimized indexing, and connection pooling.

## Technical Architecture

### Database Stack
- **Primary Database**: PostgreSQL 14+ for relational data
- **Cache/Session Store**: Redis 6+ for sessions and real-time features
- **ORM**: TypeORM or Knex.js for database operations
- **Migration Tool**: Database migrations with versioning
- **Connection Pooling**: pg-pool for PostgreSQL

### Schema Design

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(500),
    is_online BOOLEAN DEFAULT false,
    last_seen TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

#### Rooms Table
```sql
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_private BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rooms_created_by ON rooms(created_by);
CREATE INDEX idx_rooms_name ON rooms(name);
```

#### Messages Table
```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    message_type VARCHAR(20) DEFAULT 'text',
    is_edited BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_messages_room_id ON messages(room_id);
CREATE INDEX idx_messages_user_id ON messages(user_id);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
```

#### Room Users Junction Table
```sql
CREATE TABLE room_users (
    room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    role VARCHAR(20) DEFAULT 'member',
    last_read_message_id UUID REFERENCES messages(id),
    PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_room_users_user_id ON room_users(user_id);
```

#### Message Read Receipts Table
```sql
CREATE TABLE message_read_receipts (
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (message_id, user_id)
);
```

## Implementation Steps

### 1. Database Configuration

#### PostgreSQL Setup
```typescript
// backend/src/config/database.ts
import { Pool } from 'pg';
import { config } from 'dotenv';

config();

const pool = new Pool({
  host: process.env.DB_HOST,
  port: parseInt(process.env.DB_PORT || '5432'),
  database: process.env.DB_NAME,
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,
  max: 20, // Maximum pool size
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});

export default pool;
```

#### Redis Configuration
```typescript
// backend/src/config/redis.ts
import Redis from 'ioredis';

const redis = new Redis({
  host: process.env.REDIS_HOST,
  port: parseInt(process.env.REDIS_PORT || '6379'),
  password: process.env.REDIS_PASSWORD,
  db: 0,
  retryStrategy: (times) => {
    const delay = Math.min(times * 50, 2000);
    return delay;
  },
});

const pubClient = redis.duplicate();
const subClient = redis.duplicate();

export { redis, pubClient, subClient };
```

### 2. Migration System

#### Migration Setup
```bash
# Install migration tool
npm install -D knex

# Initialize migrations
npx knex init

# Create migration files
npx knex migrate:make create_users_table
npx knex migrate:make create_rooms_table
npx knex migrate:make create_messages_table
npx knex migrate:make create_room_users_table
```

#### Example Migration File
```javascript
// migrations/001_create_users_table.js
exports.up = function(knex) {
  return knex.schema.createTable('users', table => {
    table.uuid('id').primary().defaultTo(knex.raw('gen_random_uuid()'));
    table.string('email', 255).unique().notNullable();
    table.string('username', 50).unique().notNullable();
    table.string('password_hash', 255).notNullable();
    table.string('avatar_url', 500);
    table.boolean('is_online').defaultTo(false);
    table.timestamp('last_seen');
    table.timestamps(true, true);
    
    table.index('email');
    table.index('username');
  });
};

exports.down = function(knex) {
  return knex.schema.dropTable('users');
};
```

### 3. Data Models

#### TypeScript Interfaces
```typescript
// backend/src/types/models.ts
export interface User {
  id: string;
  email: string;
  username: string;
  passwordHash: string;
  avatarUrl?: string;
  isOnline: boolean;
  lastSeen?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface Room {
  id: string;
  name: string;
  description?: string;
  isPrivate: boolean;
  createdBy: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface Message {
  id: string;
  roomId: string;
  userId: string;
  content: string;
  messageType: 'text' | 'image' | 'file';
  isEdited: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface RoomUser {
  roomId: string;
  userId: string;
  joinedAt: Date;
  role: 'admin' | 'moderator' | 'member';
  lastReadMessageId?: string;
}
```

### 4. Repository Pattern

#### User Repository
```typescript
// backend/src/repositories/userRepository.ts
import pool from '../config/database';
import { User } from '../types/models';

export class UserRepository {
  async create(userData: Omit<User, 'id' | 'createdAt' | 'updatedAt'>): Promise<User> {
    const query = `
      INSERT INTO users (email, username, password_hash, avatar_url)
      VALUES ($1, $2, $3, $4)
      RETURNING *
    `;
    
    const values = [userData.email, userData.username, userData.passwordHash, userData.avatarUrl];
    const result = await pool.query(query, values);
    return this.mapToUser(result.rows[0]);
  }

  async findByEmail(email: string): Promise<User | null> {
    const query = 'SELECT * FROM users WHERE email = $1';
    const result = await pool.query(query, [email]);
    return result.rows[0] ? this.mapToUser(result.rows[0]) : null;
  }

  async updateOnlineStatus(userId: string, isOnline: boolean): Promise<void> {
    const query = `
      UPDATE users 
      SET is_online = $2, last_seen = CURRENT_TIMESTAMP 
      WHERE id = $1
    `;
    await pool.query(query, [userId, isOnline]);
  }

  private mapToUser(row: any): User {
    return {
      id: row.id,
      email: row.email,
      username: row.username,
      passwordHash: row.password_hash,
      avatarUrl: row.avatar_url,
      isOnline: row.is_online,
      lastSeen: row.last_seen,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }
}
```

### 5. Redis Integration

#### Session Management
```typescript
// backend/src/services/sessionService.ts
import { redis } from '../config/redis';

export class SessionService {
  private readonly SESSION_PREFIX = 'session:';
  private readonly SESSION_TTL = 86400; // 24 hours

  async createSession(userId: string, sessionData: any): Promise<string> {
    const sessionId = generateSessionId();
    const key = `${this.SESSION_PREFIX}${sessionId}`;
    
    await redis.setex(
      key,
      this.SESSION_TTL,
      JSON.stringify({ userId, ...sessionData })
    );
    
    return sessionId;
  }

  async getSession(sessionId: string): Promise<any | null> {
    const key = `${this.SESSION_PREFIX}${sessionId}`;
    const data = await redis.get(key);
    return data ? JSON.parse(data) : null;
  }

  async deleteSession(sessionId: string): Promise<void> {
    const key = `${this.SESSION_PREFIX}${sessionId}`;
    await redis.del(key);
  }
}
```

#### Real-time Features
```typescript
// backend/src/services/realtimeService.ts
import { pubClient, subClient } from '../config/redis';

export class RealtimeService {
  async publishTypingIndicator(roomId: string, userId: string, isTyping: boolean): Promise<void> {
    const channel = `typing:${roomId}`;
    await pubClient.publish(channel, JSON.stringify({ userId, isTyping }));
  }

  async publishUserPresence(userId: string, isOnline: boolean): Promise<void> {
    const channel = 'presence';
    await pubClient.publish(channel, JSON.stringify({ userId, isOnline }));
  }

  subscribeToChannel(channel: string, callback: (message: string) => void): void {
    subClient.subscribe(channel);
    subClient.on('message', (receivedChannel, message) => {
      if (receivedChannel === channel) {
        callback(message);
      }
    });
  }
}
```

## Performance Optimizations

### Database Indexing Strategy
- Primary key indexes on all tables
- Foreign key indexes for join operations
- Composite indexes for common query patterns
- Partial indexes for filtered queries

### Connection Pooling
- PostgreSQL: 20 connections max
- Redis: Connection reuse with ioredis
- Idle timeout: 30 seconds
- Connection timeout: 2 seconds

### Query Optimization
- Use prepared statements
- Batch operations where possible
- Implement pagination for large datasets
- Cache frequently accessed data in Redis

## Docker Configuration

### docker-compose.yml Addition
```yaml
services:
  postgres:
    image: postgres:14-alpine
    environment:
      POSTGRES_DB: chatapp
      POSTGRES_USER: chatuser
      POSTGRES_PASSWORD: chatpass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:6-alpine
    command: redis-server --requirepass redispass
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"

volumes:
  postgres_data:
  redis_data:
```

## Security Considerations

### Password Security
- Use bcrypt for password hashing
- Minimum 10 rounds for bcrypt
- Never store plain text passwords

### SQL Injection Prevention
- Use parameterized queries
- Input validation and sanitization
- Escape special characters

### Redis Security
- Password protection enabled
- Bind to localhost only in production
- Use SSL/TLS for connections

## Testing Strategy

### Unit Tests
- Test all repository methods
- Mock database connections
- Test error handling

### Integration Tests
- Test actual database operations
- Verify foreign key constraints
- Test transaction rollbacks

### Performance Tests
- Load test connection pooling
- Benchmark query performance
- Test Redis pub/sub throughput