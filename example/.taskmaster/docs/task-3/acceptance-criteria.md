# Acceptance Criteria for Task 3: Implement Database Schema and ORM

## Functional Requirements

### Database Schema
1. **Users Table**
   - [ ] Contains id (UUID), email (unique), password_hash, full_name, role, created_at, updated_at
   - [ ] Email field enforces uniqueness constraint
   - [ ] Role field defaults to 'member'
   - [ ] Timestamps automatically set on insert/update

2. **Boards Table**
   - [ ] Contains id (UUID), title, description, owner_id, created_at, updated_at
   - [ ] Owner_id references users table with foreign key
   - [ ] Title is required (NOT NULL)
   - [ ] Supports NULL description

3. **Board Members Table**
   - [ ] Contains id (UUID), board_id, user_id, role, joined_at
   - [ ] Composite unique constraint on (board_id, user_id)
   - [ ] Cascade delete when board or user is deleted
   - [ ] Role defaults to 'member'

4. **Tasks Table**
   - [ ] Contains all required fields per specification
   - [ ] Board_id is required with foreign key to boards
   - [ ] Assignee_id is optional (nullable)
   - [ ] Status defaults to 'todo'
   - [ ] Priority defaults to 'medium'
   - [ ] Due_date is optional

### ORM Integration
1. **Diesel Configuration**
   - [ ] diesel.toml properly configured
   - [ ] Migrations directory structure created
   - [ ] Environment variables loaded from .env

2. **Model Structs**
   - [ ] User, Board, BoardMember, Task structs implement Queryable
   - [ ] NewUser, NewBoard, NewBoardMember, NewTask structs implement Insertable
   - [ ] All structs use appropriate field types (Uuid, DateTime<Utc>, etc.)

## Technical Requirements

### Performance
1. **Indexes**
   - [ ] All foreign key columns have indexes
   - [ ] Task status column indexed for filtering
   - [ ] Board members indexed by both board_id and user_id
   - [ ] Query execution plans show index usage

2. **Connection Pooling**
   - [ ] Pool configured with min 5, max 15 connections
   - [ ] Connection checkout timeout configured
   - [ ] Pool validates connections on checkout
   - [ ] Handles connection failures gracefully

### Async Support
1. **Tokio Integration**
   - [ ] Database operations wrapped in spawn_blocking
   - [ ] No blocking calls on async runtime
   - [ ] Proper error propagation from blocking context
   - [ ] Clean shutdown of connection pool

### Data Integrity
1. **Constraints**
   - [ ] Foreign key constraints prevent orphaned records
   - [ ] Unique constraints prevent duplicate entries
   - [ ] NOT NULL constraints enforce required fields
   - [ ] CASCADE deletes maintain referential integrity

## Test Cases

### Migration Tests
```rust
#[test]
fn test_migrations_up_and_down() {
    // Given: Clean database
    // When: Run migrations up
    // Then: All tables exist with correct schema
    // When: Run migrations down
    // Then: All tables are dropped
}
```

### Model CRUD Tests
```rust
#[tokio::test]
async fn test_user_crud_operations() {
    // Test: Create user
    // Test: Read user by id
    // Test: Update user fields
    // Test: Delete user
}

#[tokio::test]
async fn test_board_with_members() {
    // Test: Create board with owner
    // Test: Add members to board
    // Test: Remove members from board
    // Test: Delete board cascades to memberships
}

#[tokio::test]
async fn test_task_assignment() {
    // Test: Create task without assignee
    // Test: Assign task to user
    // Test: Reassign task
    // Test: Unassign task (set to NULL)
}
```

### Constraint Tests
```rust
#[tokio::test]
async fn test_unique_constraints() {
    // Test: Duplicate email fails
    // Test: Same user can't join board twice
}

#[tokio::test]
async fn test_foreign_key_constraints() {
    // Test: Can't create board with non-existent owner
    // Test: Can't assign task to non-existent user
}

#[tokio::test]
async fn test_cascade_deletes() {
    // Test: Deleting board removes all tasks
    // Test: Deleting user removes board memberships
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_concurrent_connections() {
    // Test: Spawn 20 concurrent queries
    // Assert: All complete without pool exhaustion
}

#[tokio::test]
async fn test_index_performance() {
    // Insert 10,000 tasks
    // Test: Query by status uses index
    // Assert: Query time < 10ms
}
```

## Verification Steps

### Manual Verification
1. **Database Inspection**
   ```sql
   -- Check all tables exist
   \dt
   
   -- Verify indexes
   \di
   
   -- Check constraints
   \d+ users
   \d+ boards
   \d+ board_members
   \d+ tasks
   ```

2. **Migration Verification**
   ```bash
   # Fresh migration
   diesel migration run
   
   # Verify idempotency
   diesel migration run  # Should report no changes
   
   # Test rollback
   diesel migration redo
   
   # Check pending
   diesel migration pending  # Should show none
   ```

3. **Connection Pool Monitoring**
   ```rust
   // Log pool statistics
   println!("Active connections: {}", pool.state().connections);
   println!("Idle connections: {}", pool.state().idle_connections);
   ```

### Automated Verification
1. **CI Pipeline Checks**
   - [ ] All tests pass in clean environment
   - [ ] No compiler warnings
   - [ ] No clippy warnings
   - [ ] Migration rollback succeeds

2. **Load Testing**
   - [ ] 100 concurrent users supported
   - [ ] No connection pool exhaustion
   - [ ] Response times remain consistent

### Documentation Verification
1. **Code Documentation**
   - [ ] All public structs and functions documented
   - [ ] Migration files include comments
   - [ ] README includes database setup instructions

2. **Schema Documentation**
   - [ ] ERD diagram available
   - [ ] Field purposes documented
   - [ ] Index rationale explained

## Success Metrics

- **Functional Completeness**: 100% of schema requirements implemented
- **Test Coverage**: >90% code coverage for database module
- **Performance**: All queries complete in <50ms
- **Reliability**: Zero connection pool failures under normal load
- **Maintainability**: Clean separation between models, schema, and queries
