# Task 2: Database Setup and Schema Design - Acceptance Criteria

## Functional Requirements

### 1. PostgreSQL Database Schema ✓
- [ ] Database created with name `chatapp`
- [ ] All required tables created:
  - [ ] `users` table with all specified columns
  - [ ] `rooms` table with proper structure
  - [ ] `messages` table with content and metadata
  - [ ] `room_users` junction table
  - [ ] `message_read_receipts` table
- [ ] Primary keys configured on all tables
- [ ] Foreign key constraints properly set
- [ ] Indexes created for performance optimization

### 2. Redis Configuration ✓
- [ ] Redis server running and accessible
- [ ] Password authentication configured
- [ ] Connection established from backend
- [ ] Pub/sub channels working
- [ ] Session storage operational

### 3. Migration System ✓
- [ ] Migration tool installed and configured
- [ ] Migration files created for all tables
- [ ] Migrations can be run successfully
- [ ] Rollback functionality tested
- [ ] Seed data scripts created

### 4. Data Models ✓
- [ ] TypeScript interfaces defined for all entities
- [ ] Repository classes implemented:
  - [ ] UserRepository with CRUD operations
  - [ ] RoomRepository with room management
  - [ ] MessageRepository with message handling
- [ ] Proper type safety throughout
- [ ] Error handling in all methods

### 5. Connection Management ✓
- [ ] PostgreSQL connection pool configured
- [ ] Redis client properly initialized
- [ ] Environment variables used for credentials
- [ ] Connection error handling implemented
- [ ] Graceful shutdown handling

## Technical Validation

### Database Tests
```sql
-- Test 1: Verify all tables exist
SELECT table_name FROM information_schema.tables 
WHERE table_schema = 'public';
✓ Returns: users, rooms, messages, room_users, message_read_receipts

-- Test 2: Check indexes
SELECT indexname FROM pg_indexes 
WHERE schemaname = 'public';
✓ All performance indexes present

-- Test 3: Test foreign key constraints
INSERT INTO messages (room_id, user_id, content) 
VALUES ('invalid-uuid', 'invalid-uuid', 'test');
✓ Should fail with foreign key violation

-- Test 4: Test unique constraints
INSERT INTO users (email, username, password_hash) 
VALUES ('test@test.com', 'testuser', 'hash');
-- Insert again with same email
✓ Should fail with unique constraint violation
```

### Redis Tests
```bash
# Test 1: Redis connection
redis-cli -a ${REDIS_PASSWORD} ping
✓ Returns: PONG

# Test 2: Set and get session
redis-cli set session:test "data"
redis-cli get session:test
✓ Returns: "data"

# Test 3: Pub/sub test
# Terminal 1:
redis-cli subscribe test-channel
# Terminal 2:
redis-cli publish test-channel "message"
✓ Message received in Terminal 1
```

### Application Tests
```typescript
// Test 1: Database connection
const pool = await connectToDatabase();
✓ Connection successful, no errors

// Test 2: Create user
const user = await userRepository.create({
  email: 'test@example.com',
  username: 'testuser',
  passwordHash: 'hashed_password'
});
✓ User created with generated UUID

// Test 3: Redis session
const sessionId = await sessionService.createSession(userId, data);
const session = await sessionService.getSession(sessionId);
✓ Session stored and retrieved successfully

// Test 4: Real-time publish
await realtimeService.publishTypingIndicator(roomId, userId, true);
✓ Message published to Redis channel
```

## Performance Criteria

### Query Performance
- [ ] Simple SELECT queries < 10ms
- [ ] JOIN queries < 50ms
- [ ] Bulk inserts handle 1000+ records efficiently
- [ ] Connection pool handles concurrent requests

### Redis Performance
- [ ] Session operations < 5ms
- [ ] Pub/sub latency < 10ms
- [ ] Can handle 10,000+ sessions
- [ ] Memory usage optimized

### Connection Limits
- [ ] PostgreSQL accepts up to 20 concurrent connections
- [ ] Redis handles multiple pub/sub clients
- [ ] Connection failures handled gracefully
- [ ] Automatic reconnection implemented

## Security Validation

### Access Control
- [ ] Database credentials not hardcoded
- [ ] Environment variables properly configured
- [ ] Redis password protection enabled
- [ ] Connection strings secure

### Data Protection
- [ ] SQL injection prevention verified
- [ ] Parameterized queries used throughout
- [ ] Input validation on all user data
- [ ] Password hashes never exposed

## Integration Requirements

### Docker Integration
```yaml
# Verify in docker-compose.yml
services:
  postgres:
    ✓ Image: postgres:14-alpine
    ✓ Environment variables set
    ✓ Volume mounted for persistence
    ✓ Port 5432 exposed

  redis:
    ✓ Image: redis:6-alpine
    ✓ Password authentication
    ✓ Volume for data persistence
    ✓ Port 6379 exposed
```

### Environment Variables
```bash
# Required in .env files
DB_HOST=postgres
DB_PORT=5432
DB_NAME=chatapp
DB_USER=chatuser
DB_PASSWORD=secure_password
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=redis_password
```

## Repository Method Tests

### UserRepository
- [ ] `create()` - Creates user with all fields
- [ ] `findById()` - Returns user or null
- [ ] `findByEmail()` - Case-insensitive search
- [ ] `update()` - Updates only provided fields
- [ ] `updateOnlineStatus()` - Sets online/offline

### RoomRepository
- [ ] `create()` - Creates room with creator
- [ ] `findById()` - Returns room with details
- [ ] `listUserRooms()` - Returns user's rooms
- [ ] `addUser()` - Adds user to room
- [ ] `removeUser()` - Removes user from room

### MessageRepository
- [ ] `create()` - Saves message with timestamp
- [ ] `findByRoom()` - Returns paginated messages
- [ ] `markAsRead()` - Updates read receipt
- [ ] `update()` - Edits message content
- [ ] `getUnreadCount()` - Returns count per room

## Migration Validation

### Migration Commands
```bash
# Run migrations
npm run migrate:latest
✓ All migrations applied successfully

# Rollback test
npm run migrate:rollback
✓ Successfully rolled back to previous version

# Migration status
npm run migrate:status
✓ Shows current migration version
```

## Final Checklist

### Must Complete
- [ ] All tables created with correct schema
- [ ] Indexes optimize query performance
- [ ] Foreign keys maintain referential integrity
- [ ] Redis configured for sessions and real-time
- [ ] Repository pattern implemented
- [ ] Connection pooling configured
- [ ] Environment variables documented
- [ ] Docker services running

### Testing Complete
- [ ] Unit tests for all repositories
- [ ] Integration tests pass
- [ ] Performance benchmarks met
- [ ] Security requirements validated
- [ ] Error handling tested
- [ ] Documentation complete

**Task is complete when all database operations work correctly, performance targets are met, and the system handles failures gracefully.**