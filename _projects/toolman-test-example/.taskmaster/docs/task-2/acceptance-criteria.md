# Acceptance Criteria: Database Setup and Schema Design

## Overview
This document defines the acceptance criteria for the database setup and schema design task.

## PostgreSQL Setup Criteria

### ✅ Database Configuration
- [ ] PostgreSQL database 'chatdb' created successfully
- [ ] UUID extension enabled: `CREATE EXTENSION "uuid-ossp"`
- [ ] Database user with proper permissions configured
- [ ] Connection string environment variables set

### ✅ Connection Pool
- [ ] Connection pool configured with min 5, max 20 connections
- [ ] Idle timeout set to 30 seconds
- [ ] Connection timeout set to 2 seconds
- [ ] Pool handles concurrent requests without exhaustion

## Schema Implementation Criteria

### ✅ Users Table
- [ ] Table created with all required columns
- [ ] UUID primary key with auto-generation
- [ ] Email column unique and not null
- [ ] Username column unique and not null
- [ ] Password hash column not null
- [ ] Timestamps auto-update on modification
- [ ] Indexes on email, username, and is_online

### ✅ Rooms Table
- [ ] Table created with proper structure
- [ ] Foreign key to users table (created_by)
- [ ] Boolean is_private with default false
- [ ] Proper cascading rules on deletion
- [ ] Index on created_by and is_private

### ✅ Messages Table
- [ ] Table created with all columns
- [ ] Foreign keys to rooms and users with CASCADE
- [ ] Message type enum includes 'text', 'image', 'file'
- [ ] Composite index on (room_id, created_at)
- [ ] Index on user_id for user message queries

### ✅ Room Users Junction Table
- [ ] Composite primary key (room_id, user_id)
- [ ] Foreign keys with proper constraints
- [ ] Role column with default 'member'
- [ ] Indexes on both foreign key columns

### ✅ Message Read Receipts
- [ ] Composite primary key (message_id, user_id)
- [ ] Foreign keys with cascade delete
- [ ] Read timestamp tracking
- [ ] Index on user_id for receipt queries

## Redis Configuration Criteria

### ✅ Redis Connection
- [ ] Redis client connects successfully
- [ ] Retry strategy implemented
- [ ] Connection errors handled gracefully
- [ ] Environment variables for host/port/password

### ✅ Session Management
- [ ] Sessions stored with pattern `session:{sessionId}`
- [ ] TTL set appropriately (e.g., 1 hour)
- [ ] Session data includes userId and permissions
- [ ] Sessions retrievable and deletable

### ✅ Real-time Features
- [ ] Typing indicators use `typing:{roomId}` pattern
- [ ] Typing data auto-expires after 5 seconds
- [ ] Presence tracking with `presence:{userId}`
- [ ] Socket mapping with `socket:{userId}`

## Migration System Criteria

### ✅ Migration Setup
- [ ] Migration tool installed and configured
- [ ] Migration scripts in package.json
- [ ] Initial migration creates all tables
- [ ] Migrations are reversible

### ✅ Migration Files
- [ ] Each migration has up and down methods
- [ ] Migrations run in correct order
- [ ] No errors during migration execution
- [ ] Rollback tested and functional

## Data Access Layer Criteria

### ✅ TypeScript Models
- [ ] User interface matches database schema
- [ ] Room interface properly typed
- [ ] Message interface includes all fields
- [ ] All optional fields marked correctly

### ✅ Repository Implementation
- [ ] UserRepository with CRUD operations
- [ ] RoomRepository with user management
- [ ] MessageRepository with pagination support
- [ ] All methods return proper TypeScript types
- [ ] Error handling in all repository methods

## Performance Criteria

### ✅ Query Performance
- [ ] User lookup by email < 10ms
- [ ] Message pagination < 50ms
- [ ] Room user list < 20ms
- [ ] All indexes utilized in queries

### ✅ Connection Management
- [ ] No connection leaks detected
- [ ] Pool reuses connections efficiently
- [ ] Failed connections retry appropriately
- [ ] Health checks report accurate status

## Docker Integration Criteria

### ✅ Service Configuration
- [ ] PostgreSQL service in docker-compose
- [ ] Redis service in docker-compose
- [ ] Persistent volumes configured
- [ ] Services accessible from app containers

### ✅ Environment Setup
- [ ] All database credentials in .env.example
- [ ] No hardcoded passwords in code
- [ ] Services start with docker-compose up
- [ ] Data persists between container restarts

## Testing Checklist

### Unit Tests
```typescript
describe('UserRepository', () => {
  it('should create a new user');
  it('should find user by email');
  it('should update online status');
  it('should handle duplicate emails');
});
```

### Integration Tests
1. **Database Connection**
   ```bash
   npm test -- --testNamePattern="database connection"
   ```

2. **Migration Verification**
   ```bash
   npm run migrate:up
   npm run migrate:down
   npm run migrate:up
   ```

3. **Redis Operations**
   ```bash
   npm test -- --testNamePattern="redis operations"
   ```

### Performance Tests
```javascript
// Test concurrent connections
for (let i = 0; i < 100; i++) {
  await userRepository.findById(userId);
}
// Should complete without connection pool exhaustion
```

## Definition of Done

The task is complete when:
1. All database tables created with proper constraints
2. All indexes improve query performance measurably
3. Redis operations complete successfully
4. Migration system works bidirectionally
5. Repository methods pass all tests
6. Docker services run without errors
7. No hardcoded credentials in codebase
8. Performance benchmarks met

## Common Issues to Avoid

- ❌ Missing foreign key constraints
- ❌ No indexes on frequently queried columns
- ❌ Synchronous database operations
- ❌ Connection pool exhaustion
- ❌ Redis keys without TTL
- ❌ Non-reversible migrations
- ❌ Hardcoded database credentials
- ❌ Missing error handling in repositories

## Verification Commands

```bash
# Check PostgreSQL schema
docker exec -it postgres_container psql -U chatuser -d chatdb -c "\dt"

# Test Redis connection
docker exec -it redis_container redis-cli ping

# Run migrations
npm run migrate:up

# Run repository tests
npm test src/repositories

# Check connection pool
npm run test:integration -- connection-pool
```