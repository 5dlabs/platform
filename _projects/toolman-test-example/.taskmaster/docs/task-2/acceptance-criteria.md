# Task 2: Database Setup and Schema Design - Acceptance Criteria

## Overview

This document defines the acceptance criteria and test cases for verifying the successful implementation of the database setup and schema design for the chat application. All criteria must be met for the task to be considered complete.

## 1. Database Connectivity Tests

### 1.1 PostgreSQL Connection
**Test:** Verify PostgreSQL database connection
```typescript
// Test Case
it('should connect to PostgreSQL database', async () => {
  const isConnected = await checkDatabaseConnection();
  expect(isConnected).toBe(true);
});

it('should handle connection pool properly', async () => {
  const pool = createDatabasePool();
  const client = await pool.connect();
  expect(client).toBeDefined();
  client.release();
  
  const health = await checkPoolHealth(pool);
  expect(health.totalCount).toBeGreaterThan(0);
  expect(health.idleCount).toBeGreaterThanOrEqual(0);
});
```

**Expected Result:**
- Connection established successfully
- Pool maintains minimum connections
- Connection errors are properly handled

### 1.2 Redis Connection
**Test:** Verify Redis connection and operations
```typescript
// Test Case
it('should connect to Redis', async () => {
  const pong = await redisClient.ping();
  expect(pong).toBe('PONG');
});

it('should handle pub/sub operations', async () => {
  const testChannel = 'test:channel';
  const testMessage = 'Hello Redis';
  
  redisSub.subscribe(testChannel);
  
  const messagePromise = new Promise((resolve) => {
    redisSub.on('message', (channel, message) => {
      if (channel === testChannel) {
        resolve(message);
      }
    });
  });
  
  await redisPub.publish(testChannel, testMessage);
  const received = await messagePromise;
  expect(received).toBe(testMessage);
});
```

**Expected Result:**
- Redis responds to ping
- Pub/sub functionality works
- Separate clients for pub/sub operations

## 2. Schema Validation Tests

### 2.1 Table Structure Verification
**Test:** Verify all tables exist with correct columns
```sql
-- Test Query
SELECT 
    table_name,
    column_name,
    data_type,
    is_nullable,
    column_default
FROM information_schema.columns
WHERE table_schema = 'public'
AND table_name IN ('users', 'rooms', 'messages', 'room_users', 'message_read_status')
ORDER BY table_name, ordinal_position;
```

**Expected Result:**
- All 5 tables exist
- Each table has all required columns
- Data types match specifications
- Constraints are properly set

### 2.2 Foreign Key Constraints
**Test:** Verify foreign key relationships
```sql
-- Test Query
SELECT
    tc.table_name,
    tc.constraint_name,
    tc.constraint_type,
    kcu.column_name,
    ccu.table_name AS foreign_table_name,
    ccu.column_name AS foreign_column_name,
    rc.delete_rule
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.constraint_column_usage AS ccu
    ON ccu.constraint_name = tc.constraint_name
JOIN information_schema.referential_constraints AS rc
    ON rc.constraint_name = tc.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY';
```

**Expected Result:**
- All foreign keys properly defined
- CASCADE delete rules where appropriate
- Referential integrity maintained

### 2.3 Index Verification
**Test:** Verify all indexes are created
```sql
-- Test Query
SELECT
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE schemaname = 'public'
ORDER BY tablename, indexname;
```

**Expected Result:**
- Primary key indexes on all tables
- Foreign key indexes created
- Composite indexes for performance
- Custom indexes as specified

## 3. Migration System Tests

### 3.1 Migration Runner Functionality
**Test:** Verify migration system works correctly
```typescript
// Test Case
it('should run migrations successfully', async () => {
  const runner = new MigrationRunner();
  await runner.initialize();
  await runner.run();
  
  // Verify migrations table exists
  const result = await dbPool.query(
    "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'migrations')"
  );
  expect(result.rows[0].exists).toBe(true);
});

it('should not run migrations twice', async () => {
  const runner = new MigrationRunner();
  await runner.run();
  const firstCount = await getMigrationCount();
  
  await runner.run();
  const secondCount = await getMigrationCount();
  
  expect(firstCount).toBe(secondCount);
});
```

**Expected Result:**
- Migrations run in order
- Migrations tracked properly
- Idempotent execution
- Rollback on failure

### 3.2 Schema Integrity After Migration
**Test:** Verify schema is complete after migrations
```typescript
// Test Case
it('should create all required database objects', async () => {
  // Check tables
  const tables = await dbPool.query(`
    SELECT table_name 
    FROM information_schema.tables 
    WHERE table_schema = 'public'
  `);
  
  const tableNames = tables.rows.map(r => r.table_name);
  expect(tableNames).toContain('users');
  expect(tableNames).toContain('rooms');
  expect(tableNames).toContain('messages');
  expect(tableNames).toContain('room_users');
  expect(tableNames).toContain('message_read_status');
  
  // Check triggers
  const triggers = await dbPool.query(`
    SELECT trigger_name, event_object_table
    FROM information_schema.triggers
    WHERE trigger_schema = 'public'
  `);
  
  expect(triggers.rows.length).toBeGreaterThan(0);
});
```

**Expected Result:**
- All tables created
- All triggers functional
- All functions created
- Schema matches design

## 4. Repository Pattern Tests

### 4.1 User Repository Tests
**Test:** Verify UserRepository operations
```typescript
// Test Cases
describe('UserRepository', () => {
  let userRepo: UserRepository;
  
  beforeEach(() => {
    userRepo = new UserRepository(dbPool);
  });
  
  it('should create a new user', async () => {
    const dto: CreateUserDto = {
      email: 'test@example.com',
      password: 'SecurePass123!',
      username: 'testuser',
      avatarUrl: 'https://example.com/avatar.jpg'
    };
    
    const user = await userRepo.create(dto);
    expect(user.id).toBeDefined();
    expect(user.email).toBe(dto.email);
    expect(user.passwordHash).not.toBe(dto.password);
  });
  
  it('should find user by email', async () => {
    const user = await userRepo.findByEmail('test@example.com');
    expect(user).toBeDefined();
    expect(user?.email).toBe('test@example.com');
  });
  
  it('should update user fields', async () => {
    const user = await userRepo.findByEmail('test@example.com');
    const updated = await userRepo.update(user!.id, {
      username: 'updateduser',
      lastSeen: new Date()
    });
    
    expect(updated?.username).toBe('updateduser');
    expect(updated?.updatedAt).not.toBe(user?.updatedAt);
  });
  
  it('should handle duplicate email constraint', async () => {
    const dto: CreateUserDto = {
      email: 'test@example.com',
      password: 'AnotherPass123!',
      username: 'anotheruser'
    };
    
    await expect(userRepo.create(dto)).rejects.toThrow();
  });
});
```

**Expected Result:**
- CRUD operations work correctly
- Password hashing implemented
- Unique constraints enforced
- Error handling works

### 4.2 Message Repository Tests
**Test:** Verify MessageRepository operations
```typescript
// Test Cases
describe('MessageRepository', () => {
  let messageRepo: MessageRepository;
  let testRoom: Room;
  let testUser: User;
  
  it('should create messages', async () => {
    const dto: CreateMessageDto = {
      roomId: testRoom.id,
      userId: testUser.id,
      content: 'Hello, world!',
      type: 'text'
    };
    
    const message = await messageRepo.create(dto);
    expect(message.id).toBeDefined();
    expect(message.content).toBe(dto.content);
  });
  
  it('should fetch messages with pagination', async () => {
    // Create 100 messages
    for (let i = 0; i < 100; i++) {
      await messageRepo.create({
        roomId: testRoom.id,
        userId: testUser.id,
        content: `Message ${i}`
      });
    }
    
    const messages = await messageRepo.findByRoom(testRoom.id, 50);
    expect(messages.length).toBe(50);
    expect(messages[0].createdAt).toBeAfter(messages[49].createdAt);
  });
  
  it('should mark messages as read', async () => {
    const message = await messageRepo.create({
      roomId: testRoom.id,
      userId: testUser.id,
      content: 'Unread message'
    });
    
    await messageRepo.markAsRead(message.id, testUser.id);
    
    const messages = await messageRepo.findByRoom(testRoom.id);
    const readMessage = messages.find(m => m.id === message.id);
    expect(readMessage?.readBy).toContain(testUser.id);
  });
});
```

**Expected Result:**
- Message creation works
- Pagination implemented correctly
- Read status tracking works
- Joins return user data

## 5. Redis Functionality Tests

### 5.1 Session Management
**Test:** Verify Redis session operations
```typescript
// Test Cases
describe('Redis Session Management', () => {
  it('should store and retrieve session data', async () => {
    const sessionData: SessionData = {
      userId: 'test-user-id',
      username: 'testuser',
      socketId: 'socket-123',
      lastActivity: new Date()
    };
    
    const key = RedisKeys.userSession(sessionData.userId);
    await redisClient.setex(key, 3600, JSON.stringify(sessionData));
    
    const retrieved = await redisClient.get(key);
    const parsed = JSON.parse(retrieved!);
    expect(parsed.userId).toBe(sessionData.userId);
  });
  
  it('should expire sessions', async () => {
    const key = 'test:expire';
    await redisClient.setex(key, 1, 'data');
    
    await new Promise(resolve => setTimeout(resolve, 1100));
    
    const value = await redisClient.get(key);
    expect(value).toBeNull();
  });
});
```

**Expected Result:**
- Session storage works
- TTL functionality works
- Data serialization correct

### 5.2 Real-time Features
**Test:** Verify typing indicators and presence
```typescript
// Test Cases
describe('Redis Real-time Features', () => {
  it('should handle typing indicators', async () => {
    const roomId = 'test-room';
    const typingData: TypingData = {
      userId: 'user-123',
      username: 'testuser',
      timestamp: Date.now()
    };
    
    const key = RedisKeys.typingIndicator(roomId);
    await redisClient.hset(key, typingData.userId, JSON.stringify(typingData));
    await redisClient.expire(key, 5);
    
    const typing = await redisClient.hgetall(key);
    expect(Object.keys(typing).length).toBe(1);
  });
  
  it('should track user presence', async () => {
    const userId = 'user-123';
    const presence: PresenceData = {
      status: 'online',
      lastSeen: new Date(),
      currentRoom: 'room-456'
    };
    
    const key = RedisKeys.userPresence(userId);
    await redisClient.set(key, JSON.stringify(presence));
    
    const retrieved = await redisClient.get(key);
    const parsed = JSON.parse(retrieved!);
    expect(parsed.status).toBe('online');
  });
});
```

**Expected Result:**
- Typing indicators work
- Presence tracking works
- Hash operations functional
- Expiration works correctly

## 6. Performance Benchmarks

### 6.1 Query Performance
**Test:** Verify query performance meets requirements
```typescript
// Test Cases
describe('Performance Benchmarks', () => {
  it('should fetch messages within 100ms', async () => {
    const start = Date.now();
    await messageRepo.findByRoom('room-id', 50);
    const duration = Date.now() - start;
    
    expect(duration).toBeLessThan(100);
  });
  
  it('should handle concurrent connections', async () => {
    const promises = [];
    for (let i = 0; i < 50; i++) {
      promises.push(dbPool.query('SELECT 1'));
    }
    
    const start = Date.now();
    await Promise.all(promises);
    const duration = Date.now() - start;
    
    expect(duration).toBeLessThan(1000);
  });
});
```

**Expected Result:**
- Message queries < 100ms
- User queries < 50ms
- Concurrent operations handled
- No connection pool exhaustion

### 6.2 Redis Performance
**Test:** Verify Redis operation latency
```typescript
// Test Cases
it('should have low Redis latency', async () => {
  const iterations = 1000;
  const start = Date.now();
  
  for (let i = 0; i < iterations; i++) {
    await redisClient.set(`perf:test:${i}`, 'value');
  }
  
  const duration = Date.now() - start;
  const avgLatency = duration / iterations;
  
  expect(avgLatency).toBeLessThan(5); // < 5ms average
});
```

**Expected Result:**
- Average latency < 5ms
- Pub/sub latency < 10ms
- No connection drops

## 7. Security Validations

### 7.1 SQL Injection Prevention
**Test:** Verify parameterized queries prevent injection
```typescript
// Test Cases
it('should prevent SQL injection', async () => {
  const maliciousInput = "'; DROP TABLE users; --";
  
  try {
    await userRepo.findByEmail(maliciousInput);
    // Should not throw error, just return null
  } catch (error) {
    fail('Query should be safe from injection');
  }
  
  // Verify users table still exists
  const tables = await dbPool.query(
    "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users')"
  );
  expect(tables.rows[0].exists).toBe(true);
});
```

**Expected Result:**
- Malicious input handled safely
- No SQL execution from user input
- Tables remain intact

### 7.2 Password Security
**Test:** Verify password hashing
```typescript
// Test Cases
it('should hash passwords securely', async () => {
  const password = 'TestPassword123!';
  const user = await userRepo.create({
    email: 'secure@example.com',
    password,
    username: 'secureuser'
  });
  
  expect(user.passwordHash).not.toBe(password);
  expect(user.passwordHash.length).toBeGreaterThan(50);
  expect(user.passwordHash).toMatch(/^\$2[aby]\$/); // bcrypt format
});
```

**Expected Result:**
- Passwords never stored in plain text
- Bcrypt hashing used
- Appropriate cost factor (10+)

## 8. Error Handling Tests

### 8.1 Database Error Handling
**Test:** Verify proper error handling
```typescript
// Test Cases
it('should handle connection failures gracefully', async () => {
  const badPool = new Pool({
    host: 'nonexistent-host',
    port: 5432,
    connectionTimeoutMillis: 1000
  });
  
  try {
    await badPool.query('SELECT 1');
    fail('Should have thrown error');
  } catch (error) {
    expect(error).toBeDefined();
    expect(error.message).toContain('connect');
  }
});

it('should handle constraint violations', async () => {
  try {
    await messageRepo.create({
      roomId: 'nonexistent-room',
      userId: 'nonexistent-user',
      content: 'Test'
    });
    fail('Should have thrown foreign key error');
  } catch (error) {
    expect(error).toBeInstanceOf(DatabaseError);
    expect(error.code).toBe('23503');
  }
});
```

**Expected Result:**
- Connection errors caught
- Constraint violations handled
- Useful error messages provided
- No system crashes

## Summary Checklist

- [ ] PostgreSQL database created and accessible
- [ ] Redis server running and accessible
- [ ] All 5 tables created with correct schema
- [ ] All indexes and constraints in place
- [ ] Migration system functional
- [ ] Repository pattern implemented for all entities
- [ ] Connection pooling configured
- [ ] Redis data structures implemented
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Performance benchmarks met
- [ ] Security measures validated
- [ ] Error handling comprehensive
- [ ] Documentation complete