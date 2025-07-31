# Task 2: Database Setup and Schema Design

## Overview
Configure PostgreSQL as the primary database and Redis for session management and real-time features in the chat application. Design and implement a scalable schema supporting users, rooms, messages, and relationships.

## Technical Implementation Guide

### Phase 1: PostgreSQL Setup

#### Database Creation
```sql
-- Create database
CREATE DATABASE chatdb;

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
```

#### Connection Configuration
```typescript
// backend/src/config/database.ts
import { Pool } from 'pg';

const pool = new Pool({
  host: process.env.DB_HOST || 'localhost',
  port: parseInt(process.env.DB_PORT || '5432'),
  user: process.env.DB_USER || 'chatuser',
  password: process.env.DB_PASSWORD,
  database: process.env.DB_NAME || 'chatdb',
  max: 20, // Connection pool size
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});

export default pool;
```

### Phase 2: Database Schema Design

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(500),
    is_online BOOLEAN DEFAULT false,
    last_seen TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_is_online ON users(is_online);
```

#### Rooms Table
```sql
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_private BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for room queries
CREATE INDEX idx_rooms_created_by ON rooms(created_by);
CREATE INDEX idx_rooms_is_private ON rooms(is_private);
```

#### Messages Table
```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    message_type VARCHAR(20) DEFAULT 'text',
    is_edited BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for message queries
CREATE INDEX idx_messages_room_id ON messages(room_id);
CREATE INDEX idx_messages_user_id ON messages(user_id);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
```

#### Room Users Junction Table
```sql
CREATE TABLE room_users (
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    role VARCHAR(20) DEFAULT 'member',
    last_read_message_id UUID REFERENCES messages(id),
    PRIMARY KEY (room_id, user_id)
);

-- Indexes for junction table
CREATE INDEX idx_room_users_user_id ON room_users(user_id);
CREATE INDEX idx_room_users_room_id ON room_users(room_id);
```

#### Message Read Receipts
```sql
CREATE TABLE message_read_receipts (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (message_id, user_id)
);

-- Index for read receipt queries
CREATE INDEX idx_read_receipts_user_id ON message_read_receipts(user_id);
```

### Phase 3: Redis Configuration

#### Redis Connection Setup
```typescript
// backend/src/config/redis.ts
import Redis from 'ioredis';

const redis = new Redis({
  host: process.env.REDIS_HOST || 'localhost',
  port: parseInt(process.env.REDIS_PORT || '6379'),
  password: process.env.REDIS_PASSWORD,
  db: parseInt(process.env.REDIS_DB || '0'),
  retryStrategy: (times) => {
    const delay = Math.min(times * 50, 2000);
    return delay;
  },
});

export default redis;
```

#### Redis Data Structures

##### Session Management
```typescript
// Session storage pattern
// Key: session:{sessionId}
// Value: JSON with user data and expiry
await redis.setex(
  `session:${sessionId}`,
  3600, // 1 hour TTL
  JSON.stringify({ userId, username, permissions })
);
```

##### Typing Indicators
```typescript
// Typing indicator pattern
// Key: typing:{roomId}
// Value: Set of user IDs currently typing
await redis.sadd(`typing:${roomId}`, userId);
await redis.expire(`typing:${roomId}`, 5); // Auto-expire after 5 seconds
```

##### Online Presence
```typescript
// User presence pattern
// Key: presence:{userId}
// Value: Timestamp of last activity
await redis.setex(`presence:${userId}`, 300, Date.now()); // 5 minute TTL
```

##### Socket.io Room Management
```typescript
// Active socket connections
// Key: socket:{userId}
// Value: Set of socket IDs
await redis.sadd(`socket:${userId}`, socketId);
```

### Phase 4: Database Migrations

#### Migration Tool Setup (using node-pg-migrate)
```bash
npm install --save-dev node-pg-migrate
```

#### Migration Configuration
```json
// package.json
{
  "scripts": {
    "migrate:create": "node-pg-migrate create",
    "migrate:up": "node-pg-migrate up",
    "migrate:down": "node-pg-migrate down"
  }
}
```

#### Example Migration File
```javascript
// migrations/1_initial_schema.js
exports.up = (pgm) => {
  // Create users table
  pgm.createTable('users', {
    id: { type: 'uuid', primaryKey: true, default: pgm.func('uuid_generate_v4()') },
    email: { type: 'varchar(255)', notNull: true, unique: true },
    username: { type: 'varchar(50)', notNull: true, unique: true },
    password_hash: { type: 'varchar(255)', notNull: true },
    avatar_url: { type: 'varchar(500)' },
    is_online: { type: 'boolean', default: false },
    last_seen: { type: 'timestamptz' },
    created_at: { type: 'timestamptz', notNull: true, default: pgm.func('current_timestamp') },
    updated_at: { type: 'timestamptz', notNull: true, default: pgm.func('current_timestamp') }
  });
  
  // Create indexes
  pgm.createIndex('users', 'email');
  pgm.createIndex('users', 'username');
};

exports.down = (pgm) => {
  pgm.dropTable('users');
};
```

### Phase 5: Data Models and Repositories

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
```

#### Repository Pattern Implementation
```typescript
// backend/src/repositories/userRepository.ts
import pool from '../config/database';
import { User } from '../types/models';

export class UserRepository {
  async create(user: Omit<User, 'id' | 'createdAt' | 'updatedAt'>): Promise<User> {
    const query = `
      INSERT INTO users (email, username, password_hash, avatar_url)
      VALUES ($1, $2, $3, $4)
      RETURNING *
    `;
    const values = [user.email, user.username, user.passwordHash, user.avatarUrl];
    const result = await pool.query(query, values);
    return this.mapRowToUser(result.rows[0]);
  }

  async findById(id: string): Promise<User | null> {
    const query = 'SELECT * FROM users WHERE id = $1';
    const result = await pool.query(query, [id]);
    return result.rows[0] ? this.mapRowToUser(result.rows[0]) : null;
  }

  async findByEmail(email: string): Promise<User | null> {
    const query = 'SELECT * FROM users WHERE email = $1';
    const result = await pool.query(query, [email]);
    return result.rows[0] ? this.mapRowToUser(result.rows[0]) : null;
  }

  private mapRowToUser(row: any): User {
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

### Phase 6: Performance Optimization

#### Database Indexing Strategy
1. **Primary Keys**: Automatic B-tree indexes
2. **Foreign Keys**: Index on frequently joined columns
3. **Search Columns**: Index on email, username for quick lookups
4. **Time-based Queries**: Index on created_at for message pagination

#### Connection Pool Optimization
```typescript
// Optimal pool configuration
const poolConfig = {
  max: 20, // Maximum connections
  min: 5,  // Minimum connections
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
  maxUses: 7500, // Close connection after X queries
};
```

#### Redis Performance Tips
1. Use pipelining for multiple operations
2. Set appropriate TTLs to prevent memory bloat
3. Use Redis Cluster for horizontal scaling
4. Monitor memory usage and eviction policies

## Docker Configuration

#### docker-compose.yml Addition
```yaml
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: chatuser
      POSTGRES_PASSWORD: chatpass
      POSTGRES_DB: chatdb
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"

volumes:
  postgres_data:
  redis_data:
```

## Success Metrics

- Database migrations execute without errors
- All tables created with proper constraints
- Indexes improve query performance
- Connection pooling handles concurrent requests
- Redis operations complete in <10ms
- Repository methods pass unit tests