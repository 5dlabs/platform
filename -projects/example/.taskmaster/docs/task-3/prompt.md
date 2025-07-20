# Autonomous Prompt for Task 3: Implement Database Schema and ORM

## Context

You are implementing the database layer for a Task Board API, a collaborative task management service built with Rust and gRPC. The system requires a robust PostgreSQL database schema with Diesel ORM integration to handle user management, project boards, tasks, and real-time collaboration features.

### Project Background
- **Application**: Task Board API - A gRPC-based microservice for task management
- **Tech Stack**: Rust, Tonic (gRPC), PostgreSQL, Diesel ORM, Tokio
- **Purpose**: Support multiple users collaborating on project boards with real-time updates

### Current State
- Task 1 (Project Setup) is complete with Rust toolchain and basic project structure
- Task 2 (gRPC Service Contracts) has defined the service interfaces
- You need to implement the database foundation that all other services will depend on

## Task Requirements

### Primary Objectives
1. Design and implement a complete PostgreSQL database schema
2. Set up Diesel ORM with proper configuration and migrations
3. Create Rust model structs for all database entities
4. Implement connection pooling for concurrent operations
5. Integrate async support with Tokio runtime
6. Ensure proper indexes for query performance

### Database Schema Requirements

#### Tables to Implement
1. **users**: User accounts with authentication data
   - UUID primary key
   - Email (unique), password hash, full name, role
   - Timestamps for audit trail

2. **boards**: Project boards for task organization
   - UUID primary key
   - Title, description, owner reference
   - Timestamps

3. **board_members**: Many-to-many relationship for board access
   - Board and user references with cascade delete
   - Member role within board
   - Join timestamp

4. **tasks**: Individual tasks within boards
   - UUID primary key
   - Board reference, title, description
   - Optional assignee reference
   - Status, priority, due date
   - Timestamps

#### Performance Requirements
- Implement indexes on all foreign keys
- Add indexes for common query patterns (status, assignee)
- Ensure efficient board member lookups

## Implementation Instructions

### Step 1: Install and Configure Diesel
```bash
# Install Diesel CLI with PostgreSQL support only
cargo install diesel_cli --no-default-features --features postgres

# Add dependencies to Cargo.toml
diesel = { version = "2.1", features = ["postgres", "uuid", "chrono", "r2d2"] }
diesel_migrations = "2.1"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
```

### Step 2: Create diesel.toml Configuration
```toml
[print_schema]
file = "src/schema.rs"

[migrations_directory]
dir = "migrations"
```

### Step 3: Set Up Database Connection
1. Create `.env` file with DATABASE_URL
2. Ensure PostgreSQL is running and database exists
3. Test connection with `diesel setup`

### Step 4: Create Initial Migration
```bash
diesel migration generate create_initial_schema
```

### Step 5: Write Migration SQL
In `migrations/[timestamp]_create_initial_schema/up.sql`:
- Create all tables with proper constraints
- Add performance indexes
- Use CASCADE for referential actions

In `down.sql`:
- Drop all tables in reverse dependency order

### Step 6: Run Migrations
```bash
diesel migration run
diesel print-schema > src/schema.rs
```

### Step 7: Implement Model Structs
Create organized model files:
- `src/models/mod.rs` - Module exports
- `src/models/user.rs` - User model and NewUser
- `src/models/board.rs` - Board model and NewBoard
- `src/models/task.rs` - Task model and NewTask

### Step 8: Set Up Connection Pool
Implement connection pool management:
- Use r2d2 for connection pooling
- Configure appropriate pool size
- Add retry logic for transient failures

### Step 9: Create Async Wrapper
Implement async query execution using tokio::task::spawn_blocking

### Step 10: Write Comprehensive Tests
- Migration up/down tests
- Model CRUD operations
- Constraint validation
- Connection pool stress tests

## Success Criteria

### Must Complete
- [ ] All tables created with correct schema
- [ ] Diesel migrations run successfully
- [ ] schema.rs generated without errors
- [ ] All model structs compile with proper derives
- [ ] Connection pool initialized correctly
- [ ] Basic CRUD operations work for all entities
- [ ] Foreign key constraints enforced
- [ ] Cascade deletes function properly
- [ ] All indexes created and used by queries

### Quality Checks
- [ ] No compiler warnings in generated code
- [ ] All tests pass including constraint tests
- [ ] Async queries work without blocking runtime
- [ ] Connection pool handles concurrent requests
- [ ] Proper error types for database operations
- [ ] Migration rollback tested and functional

### Performance Validation
- [ ] Indexed queries show performance improvement
- [ ] Connection pool doesn't exhaust under load
- [ ] No N+1 query problems in common operations
- [ ] Efficient pagination support

## Common Pitfalls to Avoid

1. **UUID Handling**: Ensure uuid feature is enabled in Diesel
2. **Timestamp Zones**: Always use TIMESTAMP WITH TIME ZONE
3. **Migration Order**: Create tables in dependency order
4. **Async Integration**: Don't block the Tokio runtime with sync calls
5. **Connection Leaks**: Always properly return connections to pool
6. **Error Handling**: Convert Diesel errors to service-level errors

## Testing Commands

```bash
# Run all migrations
diesel migration run

# Test rollback
diesel migration redo

# Check pending migrations
diesel migration pending

# Run tests
cargo test --test database_tests

# Check schema generation
diesel print-schema
```

## Expected Deliverables

1. Complete migration files in `migrations/` directory
2. Generated `src/schema.rs` file
3. Model implementations in `src/models/`
4. Connection pool setup in `src/db/`
5. Async wrapper utilities
6. Comprehensive test suite
7. Documentation of database design decisions

Remember to follow Rust idioms, use proper error handling with Result types, and ensure all database operations are properly abstracted for use by the service layer.
