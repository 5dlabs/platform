# Task 3: Implement Database Schema and ORM

## Overview

This task involves designing and implementing the PostgreSQL database schema for the Task Board API and setting up Diesel ORM for type-safe database interactions. The schema must support user management, project boards, task tracking, and real-time collaboration features while ensuring data integrity and performance.

## Architecture Context

Based on the system architecture:

### Database Technology Stack
- **Database**: PostgreSQL (latest stable version)
- **ORM**: Diesel 2.1+ for type-safe queries and migrations
- **Connection Management**: Built-in connection pooling with Diesel
- **Async Support**: Integration with Tokio runtime for async operations

### Schema Design Principles
1. **UUID Primary Keys**: Use UUIDs for all primary keys to support distributed systems
2. **Timestamps**: Include created_at and updated_at for audit trails
3. **Referential Integrity**: Enforce foreign key constraints at the database level
4. **Performance Indexes**: Create indexes for common query patterns
5. **Soft Deletes**: Consider implementing soft deletes for data recovery

## Implementation Details

### 1. Database Schema Structure

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Boards table
CREATE TABLE boards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Board members table (many-to-many)
CREATE TABLE board_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(board_id, user_id)
);

-- Tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    assignee_id UUID REFERENCES users(id),
    status VARCHAR(50) NOT NULL DEFAULT 'todo',
    priority VARCHAR(50) NOT NULL DEFAULT 'medium',
    due_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 2. Performance Indexes

```sql
-- Critical indexes for query performance
CREATE INDEX idx_board_members_board_id ON board_members(board_id);
CREATE INDEX idx_board_members_user_id ON board_members(user_id);
CREATE INDEX idx_tasks_board_id ON tasks(board_id);
CREATE INDEX idx_tasks_assignee_id ON tasks(assignee_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_boards_owner_id ON boards(owner_id);
```

### 3. Diesel Setup and Configuration

#### Install Diesel CLI
```bash
cargo install diesel_cli --no-default-features --features postgres
```

#### Configure diesel.toml
```toml
[print_schema]
file = "src/schema.rs"

[migrations_directory]
dir = "migrations"
```

#### Environment Configuration
```bash
# .env file
DATABASE_URL=postgres://username:password@localhost/task_board_api
```

### 4. Migration Management

#### Create Initial Migration
```bash
diesel migration generate create_initial_schema
```

#### Migration Files Structure
```
migrations/
├── 2024_01_01_000001_create_initial_schema/
│   ├── up.sql    # Schema creation
│   └── down.sql  # Schema rollback
```

#### Run Migrations
```bash
diesel migration run
diesel migration redo  # For testing rollback
```

### 5. Diesel Models Implementation

```rust
// src/models/user.rs
use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::schema::users;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub full_name: &'a str,
    pub role: &'a str,
}
```

### 6. Connection Pool Setup

```rust
// src/db/mod.rs
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create connection pool")
}
```

### 7. Async Integration with Tokio

```rust
// src/db/async_wrapper.rs
use tokio::task;
use diesel::prelude::*;
use crate::db::DbPool;

pub async fn run_async_query<F, R>(
    pool: &DbPool,
    query_fn: F,
) -> Result<R, diesel::result::Error>
where
    F: FnOnce(&mut PgConnection) -> Result<R, diesel::result::Error> + Send + 'static,
    R: Send + 'static,
{
    let pool = pool.clone();
    task::spawn_blocking(move || {
        let mut conn = pool.get().unwrap();
        query_fn(&mut conn)
    })
    .await
    .unwrap()
}
```

## Dependencies

### Task Dependencies
- **Task 1**: Setup Project Repository and Toolchain (must be complete)
  - Rust toolchain installed
  - Project structure initialized
  - Development environment configured

### Cargo Dependencies
```toml
[dependencies]
diesel = { version = "2.1", features = ["postgres", "uuid", "chrono", "r2d2"] }
diesel_migrations = "2.1"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
diesel_cli = { version = "2.1", default-features = false, features = ["postgres"] }
```

## Testing Strategy

### 1. Migration Testing
```bash
# Test migration up and down
diesel migration run
diesel migration redo
diesel migration pending  # Should show no pending migrations
```

### 2. Schema Validation Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use diesel::prelude::*;
    
    #[test]
    fn test_schema_compiles() {
        // This test ensures schema.rs is valid
        use crate::schema::*;
        let _ = users::table;
        let _ = boards::table;
        let _ = board_members::table;
        let _ = tasks::table;
    }
}
```

### 3. Model CRUD Tests
```rust
#[tokio::test]
async fn test_user_creation() {
    let pool = establish_test_pool();
    
    let new_user = NewUser {
        email: "test@example.com",
        password_hash: "hashed_password",
        full_name: "Test User",
        role: "member",
    };
    
    let result = run_async_query(&pool, move |conn| {
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)
    }).await;
    
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

### 4. Constraint Testing
```rust
#[tokio::test]
async fn test_unique_email_constraint() {
    // Test that duplicate emails are rejected
    // Test foreign key constraints work
    // Test cascade deletes function properly
}
```

### 5. Performance Testing
```rust
#[tokio::test]
async fn test_query_performance() {
    // Test that indexed queries perform within acceptable limits
    // Verify connection pool handles concurrent requests
}
```

## Subtask Breakdown

1. **Design Database Schema** (Complete schema design with all tables and relationships)
2. **Write and Organize Migrations** (Create migration files for schema creation)
3. **Configure Diesel in Project** (Setup diesel.toml and dependencies)
4. **Generate schema.rs File** (Run diesel print-schema)
5. **Implement Rust Models** (Create model structs for all tables)
6. **Verify Setup with Test Queries** (Write and run basic CRUD operations)

## Success Metrics

- All migrations run successfully without errors
- Schema.rs is generated and compiles without warnings
- All model structs implement necessary Diesel traits
- Basic CRUD operations work for all entities
- Foreign key constraints are properly enforced
- Indexes improve query performance as expected
- Connection pool handles concurrent operations efficiently
