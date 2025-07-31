# Task 2: Database Setup and Schema Design

## Overview

This task establishes the data persistence layer for the chat application using PostgreSQL as the primary database and Redis for session management and real-time features. The implementation includes comprehensive schema design, migration scripts, connection pooling, and TypeScript data models with repository patterns.

## Prerequisites

- Task 1: Project Setup and Core Architecture (completed)
- PostgreSQL 15+ installed and running
- Redis 7+ installed and running
- Database administration credentials
- TypeScript and Node.js environment configured

## Technical Implementation

### 1. PostgreSQL Database Setup

#### 1.1 Database Creation

```bash
# Create database and user
sudo -u postgres psql
CREATE DATABASE chatapp_db;
CREATE USER chatapp_user WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE chatapp_db TO chatapp_user;
\q
```

#### 1.2 Connection Configuration

```typescript
// src/config/database.ts
import { Pool } from 'pg';
import { config } from './config';

export const dbPool = new Pool({
  host: config.db.host,
  port: config.db.port,
  database: config.db.database,
  user: config.db.user,
  password: config.db.password,
  max: 20, // maximum pool size
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
  ssl: config.environment === 'production' ? { rejectUnauthorized: false } : false
});

// Connection health check
export async function checkDatabaseConnection(): Promise<boolean> {
  try {
    const client = await dbPool.connect();
    await client.query('SELECT 1');
    client.release();
    return true;
  } catch (error) {
    console.error('Database connection failed:', error);
    return false;
  }
}
```

### 2. Complete Schema Design

#### 2.1 Users Table

```sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  username VARCHAR(50) UNIQUE NOT NULL,
  avatar_url TEXT,
  is_active BOOLEAN DEFAULT true,
  last_seen TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_is_active ON users(is_active);
```

#### 2.2 Rooms Table

```sql
CREATE TABLE rooms (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(100) NOT NULL,
  description TEXT,
  type VARCHAR(20) NOT NULL DEFAULT 'public', -- 'public', 'private', 'direct'
  created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rooms_created_by ON rooms(created_by);
CREATE INDEX idx_rooms_type ON rooms(type);
CREATE INDEX idx_rooms_is_active ON rooms(is_active);
```

#### 2.3 Messages Table

```sql
CREATE TABLE messages (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  content TEXT NOT NULL,
  type VARCHAR(20) DEFAULT 'text', -- 'text', 'image', 'file', 'system'
  is_edited BOOLEAN DEFAULT false,
  edited_at TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_messages_room_id ON messages(room_id);
CREATE INDEX idx_messages_user_id ON messages(user_id);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC);
```

#### 2.4 Room Users Table

```sql
CREATE TABLE room_users (
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role VARCHAR(20) DEFAULT 'member', -- 'owner', 'admin', 'member'
  joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  last_read_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_room_users_user_id ON room_users(user_id);
CREATE INDEX idx_room_users_role ON room_users(role);
```

#### 2.5 Message Read Status Table

```sql
CREATE TABLE message_read_status (
  message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  read_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (message_id, user_id)
);

CREATE INDEX idx_message_read_user ON message_read_status(user_id);
```

#### 2.6 Database Functions and Triggers

```sql
-- Update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rooms_updated_at BEFORE UPDATE ON rooms
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 3. Redis Setup and Configuration

#### 3.1 Redis Connection

```typescript
// src/config/redis.ts
import Redis from 'ioredis';
import { config } from './config';

export const redisClient = new Redis({
  host: config.redis.host,
  port: config.redis.port,
  password: config.redis.password,
  db: 0,
  retryStrategy: (times) => {
    const delay = Math.min(times * 50, 2000);
    return delay;
  },
  reconnectOnError: (err) => {
    const targetError = 'READONLY';
    if (err.message.includes(targetError)) {
      return true;
    }
    return false;
  }
});

// Redis pub/sub clients
export const redisPub = redisClient.duplicate();
export const redisSub = redisClient.duplicate();

// Redis key patterns
export const RedisKeys = {
  userSession: (userId: string) => `session:${userId}`,
  refreshToken: (token: string) => `refresh:${token}`,
  userPresence: (userId: string) => `presence:${userId}`,
  roomPresence: (roomId: string) => `room:${roomId}:presence`,
  typingIndicator: (roomId: string) => `typing:${roomId}`,
  socketMapping: (socketId: string) => `socket:${socketId}`
};
```

#### 3.2 Redis Data Structures

```typescript
// Session Management
interface SessionData {
  userId: string;
  username: string;
  socketId?: string;
  lastActivity: Date;
}

// Typing Indicators
interface TypingData {
  userId: string;
  username: string;
  timestamp: number;
}

// Presence Status
interface PresenceData {
  status: 'online' | 'away' | 'offline';
  lastSeen: Date;
  currentRoom?: string;
}
```

### 4. Migration Scripts

#### 4.1 Migration Runner

```typescript
// src/db/migrate.ts
import { readdir, readFile } from 'fs/promises';
import { join } from 'path';
import { dbPool } from '../config/database';

interface Migration {
  id: number;
  name: string;
  executed_at: Date;
}

export class MigrationRunner {
  private migrationsPath = join(__dirname, 'migrations');

  async initialize(): Promise<void> {
    await dbPool.query(`
      CREATE TABLE IF NOT EXISTS migrations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) UNIQUE NOT NULL,
        executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      )
    `);
  }

  async run(): Promise<void> {
    await this.initialize();
    
    const executedMigrations = await this.getExecutedMigrations();
    const migrationFiles = await this.getMigrationFiles();
    
    for (const file of migrationFiles) {
      if (!executedMigrations.includes(file)) {
        await this.executeMigration(file);
      }
    }
  }

  private async executeMigration(filename: string): Promise<void> {
    const filepath = join(this.migrationsPath, filename);
    const sql = await readFile(filepath, 'utf-8');
    
    const client = await dbPool.connect();
    try {
      await client.query('BEGIN');
      await client.query(sql);
      await client.query(
        'INSERT INTO migrations (name) VALUES ($1)',
        [filename]
      );
      await client.query('COMMIT');
      console.log(`Migration executed: ${filename}`);
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  private async getExecutedMigrations(): Promise<string[]> {
    const result = await dbPool.query<Migration>(
      'SELECT name FROM migrations ORDER BY id'
    );
    return result.rows.map(row => row.name);
  }

  private async getMigrationFiles(): Promise<string[]> {
    const files = await readdir(this.migrationsPath);
    return files
      .filter(f => f.endsWith('.sql'))
      .sort();
  }
}
```

### 5. TypeScript Data Models

#### 5.1 User Model

```typescript
// src/models/User.ts
export interface User {
  id: string;
  email: string;
  passwordHash: string;
  username: string;
  avatarUrl?: string;
  isActive: boolean;
  lastSeen?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface CreateUserDto {
  email: string;
  password: string;
  username: string;
  avatarUrl?: string;
}

export interface UpdateUserDto {
  username?: string;
  avatarUrl?: string;
  lastSeen?: Date;
}
```

#### 5.2 Room Model

```typescript
// src/models/Room.ts
export interface Room {
  id: string;
  name: string;
  description?: string;
  type: 'public' | 'private' | 'direct';
  createdBy: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface RoomWithUsers extends Room {
  users: RoomUser[];
  userCount: number;
}

export interface CreateRoomDto {
  name: string;
  description?: string;
  type: 'public' | 'private' | 'direct';
  createdBy: string;
}
```

#### 5.3 Message Model

```typescript
// src/models/Message.ts
export interface Message {
  id: string;
  roomId: string;
  userId: string;
  content: string;
  type: 'text' | 'image' | 'file' | 'system';
  isEdited: boolean;
  editedAt?: Date;
  createdAt: Date;
}

export interface MessageWithUser extends Message {
  user: {
    id: string;
    username: string;
    avatarUrl?: string;
  };
  readBy: string[];
}

export interface CreateMessageDto {
  roomId: string;
  userId: string;
  content: string;
  type?: 'text' | 'image' | 'file' | 'system';
}
```

### 6. Repository Pattern Implementation

#### 6.1 Base Repository

```typescript
// src/repositories/BaseRepository.ts
import { Pool } from 'pg';

export abstract class BaseRepository<T> {
  constructor(
    protected pool: Pool,
    protected tableName: string
  ) {}

  protected async query<R = T>(
    sql: string,
    params?: any[]
  ): Promise<R[]> {
    const result = await this.pool.query<R>(sql, params);
    return result.rows;
  }

  protected async queryOne<R = T>(
    sql: string,
    params?: any[]
  ): Promise<R | null> {
    const rows = await this.query<R>(sql, params);
    return rows[0] || null;
  }

  async findById(id: string): Promise<T | null> {
    return this.queryOne(
      `SELECT * FROM ${this.tableName} WHERE id = $1`,
      [id]
    );
  }

  async deleteById(id: string): Promise<boolean> {
    const result = await this.pool.query(
      `DELETE FROM ${this.tableName} WHERE id = $1`,
      [id]
    );
    return result.rowCount > 0;
  }
}
```

#### 6.2 User Repository

```typescript
// src/repositories/UserRepository.ts
import { BaseRepository } from './BaseRepository';
import { User, CreateUserDto, UpdateUserDto } from '../models/User';
import bcrypt from 'bcrypt';

export class UserRepository extends BaseRepository<User> {
  constructor(pool: Pool) {
    super(pool, 'users');
  }

  async create(dto: CreateUserDto): Promise<User> {
    const passwordHash = await bcrypt.hash(dto.password, 10);
    
    return this.queryOne(
      `INSERT INTO users (email, password_hash, username, avatar_url)
       VALUES ($1, $2, $3, $4)
       RETURNING *`,
      [dto.email, passwordHash, dto.username, dto.avatarUrl]
    );
  }

  async findByEmail(email: string): Promise<User | null> {
    return this.queryOne(
      'SELECT * FROM users WHERE email = $1',
      [email]
    );
  }

  async findByUsername(username: string): Promise<User | null> {
    return this.queryOne(
      'SELECT * FROM users WHERE username = $1',
      [username]
    );
  }

  async update(id: string, dto: UpdateUserDto): Promise<User | null> {
    const fields = [];
    const values = [];
    let paramCount = 1;

    Object.entries(dto).forEach(([key, value]) => {
      if (value !== undefined) {
        fields.push(`${this.camelToSnake(key)} = $${paramCount}`);
        values.push(value);
        paramCount++;
      }
    });

    if (fields.length === 0) return this.findById(id);

    values.push(id);
    return this.queryOne(
      `UPDATE users SET ${fields.join(', ')}
       WHERE id = $${paramCount}
       RETURNING *`,
      values
    );
  }

  private camelToSnake(str: string): string {
    return str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);
  }
}
```

#### 6.3 Message Repository

```typescript
// src/repositories/MessageRepository.ts
import { BaseRepository } from './BaseRepository';
import { Message, MessageWithUser, CreateMessageDto } from '../models/Message';

export class MessageRepository extends BaseRepository<Message> {
  constructor(pool: Pool) {
    super(pool, 'messages');
  }

  async create(dto: CreateMessageDto): Promise<Message> {
    return this.queryOne(
      `INSERT INTO messages (room_id, user_id, content, type)
       VALUES ($1, $2, $3, $4)
       RETURNING *`,
      [dto.roomId, dto.userId, dto.content, dto.type || 'text']
    );
  }

  async findByRoom(
    roomId: string,
    limit: number = 50,
    before?: Date
  ): Promise<MessageWithUser[]> {
    const params = [roomId, limit];
    let whereClause = 'm.room_id = $1';
    
    if (before) {
      params.push(before);
      whereClause += ` AND m.created_at < $3`;
    }

    return this.query<MessageWithUser>(
      `SELECT 
        m.*,
        json_build_object(
          'id', u.id,
          'username', u.username,
          'avatarUrl', u.avatar_url
        ) as user,
        COALESCE(
          array_agg(mrs.user_id) FILTER (WHERE mrs.user_id IS NOT NULL),
          '{}'::uuid[]
        ) as read_by
       FROM messages m
       JOIN users u ON m.user_id = u.id
       LEFT JOIN message_read_status mrs ON m.id = mrs.message_id
       WHERE ${whereClause}
       GROUP BY m.id, u.id
       ORDER BY m.created_at DESC
       LIMIT $2`,
      params
    );
  }

  async markAsRead(messageId: string, userId: string): Promise<void> {
    await this.pool.query(
      `INSERT INTO message_read_status (message_id, user_id)
       VALUES ($1, $2)
       ON CONFLICT (message_id, user_id) DO NOTHING`,
      [messageId, userId]
    );
  }

  async markRoomMessagesAsRead(roomId: string, userId: string): Promise<void> {
    await this.pool.query(
      `INSERT INTO message_read_status (message_id, user_id)
       SELECT m.id, $2
       FROM messages m
       WHERE m.room_id = $1
       ON CONFLICT (message_id, user_id) DO NOTHING`,
      [roomId, userId]
    );
  }
}
```

### 7. Connection Pooling Configuration

```typescript
// src/config/database-pool.ts
import { Pool, PoolConfig } from 'pg';
import { config } from './config';

const poolConfig: PoolConfig = {
  host: config.db.host,
  port: config.db.port,
  database: config.db.database,
  user: config.db.user,
  password: config.db.password,
  
  // Pool configuration
  max: 20,                      // Maximum number of clients in the pool
  min: 5,                       // Minimum number of clients in the pool
  idleTimeoutMillis: 30000,     // Close idle clients after 30 seconds
  connectionTimeoutMillis: 2000, // Return an error after 2 seconds if connection could not be established
  maxUses: 7500,                // Close and replace a connection after it has been used 7500 times
  
  // Statement timeout
  statement_timeout: 30000,      // 30 seconds
  query_timeout: 30000,         // 30 seconds
  
  // SSL configuration for production
  ssl: config.environment === 'production' ? {
    rejectUnauthorized: false,
    ca: config.db.sslCert
  } : false
};

// Create pool with error handling
export const createDatabasePool = (): Pool => {
  const pool = new Pool(poolConfig);
  
  // Handle pool errors
  pool.on('error', (err, client) => {
    console.error('Unexpected error on idle client', err);
  });
  
  // Log when new client is connected
  pool.on('connect', (client) => {
    console.log('New client connected to database pool');
  });
  
  // Log when client is removed
  pool.on('remove', (client) => {
    console.log('Client removed from database pool');
  });
  
  return pool;
};

// Health check function
export const checkPoolHealth = async (pool: Pool): Promise<{
  totalCount: number;
  idleCount: number;
  waitingCount: number;
}> => {
  return {
    totalCount: pool.totalCount,
    idleCount: pool.idleCount,
    waitingCount: pool.waitingCount
  };
};
```

### 8. Environment Configuration

```typescript
// src/config/config.ts
import dotenv from 'dotenv';

dotenv.config();

export const config = {
  environment: process.env.NODE_ENV || 'development',
  
  db: {
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT || '5432'),
    database: process.env.DB_NAME || 'chatapp_db',
    user: process.env.DB_USER || 'chatapp_user',
    password: process.env.DB_PASSWORD || '',
    sslCert: process.env.DB_SSL_CERT
  },
  
  redis: {
    host: process.env.REDIS_HOST || 'localhost',
    port: parseInt(process.env.REDIS_PORT || '6379'),
    password: process.env.REDIS_PASSWORD || ''
  }
};
```

## Error Handling

```typescript
// src/utils/database-errors.ts
export class DatabaseError extends Error {
  constructor(
    message: string,
    public code?: string,
    public detail?: string
  ) {
    super(message);
    this.name = 'DatabaseError';
  }
}

export function handleDatabaseError(error: any): never {
  if (error.code === '23505') {
    throw new DatabaseError(
      'Duplicate entry',
      error.code,
      error.detail
    );
  }
  
  if (error.code === '23503') {
    throw new DatabaseError(
      'Foreign key violation',
      error.code,
      error.detail
    );
  }
  
  throw new DatabaseError(
    error.message || 'Database operation failed',
    error.code
  );
}
```

## Security Considerations

1. **Connection Security**
   - Use SSL/TLS for database connections in production
   - Implement connection rate limiting
   - Use prepared statements to prevent SQL injection

2. **Access Control**
   - Use role-based database permissions
   - Implement row-level security where appropriate
   - Audit sensitive operations

3. **Data Protection**
   - Encrypt sensitive data at rest
   - Hash passwords with bcrypt (minimum 10 rounds)
   - Implement data retention policies

## Performance Optimization

1. **Indexing Strategy**
   - Primary keys on all tables
   - Foreign key indexes for joins
   - Composite indexes for common query patterns
   - Partial indexes for filtered queries

2. **Query Optimization**
   - Use EXPLAIN ANALYZE for query planning
   - Implement query result caching in Redis
   - Batch operations where possible
   - Use database views for complex queries

3. **Connection Management**
   - Proper pool sizing based on load
   - Connection recycling
   - Health checks and automatic reconnection
   - Statement timeouts to prevent long-running queries

## Next Steps

After completing this task:
1. Proceed to Task 3: Backend API Development
2. Implement authentication using the user model
3. Create API endpoints for data access
4. Set up real-time features with Redis pub/sub