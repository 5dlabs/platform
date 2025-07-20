# Task 9: Write Comprehensive Unit and Integration Tests

## Overview

This task focuses on developing a complete test suite for the Task Board API, covering unit tests for individual components and integration tests for the full system. The testing strategy ensures code quality, reliability, and maintainability while providing confidence in the system's behavior under various conditions.

## Task Context

### Description
Develop unit and integration tests for all services and endpoints.

### Priority
Medium - Essential for ensuring system reliability and facilitating future development.

### Dependencies
- Tasks 4-8: All service implementations must be complete
- Task 3: Database layer for test fixtures
- Task 2: gRPC contracts for endpoint testing

### Subtasks
1. Develop Unit Tests for Each Service
2. Create Integration Tests for gRPC Endpoints
3. Implement Database Operation Tests
4. Design Error Case and Fault Injection Tests
5. Set Up Test Container Infrastructure
6. Verify and Report Test Coverage

## Architecture Context

Based on the testing strategy outlined in architecture.md:

### Test Pyramid
1. **Unit Tests** (60%): Fast, isolated tests for individual functions and methods
2. **Integration Tests** (30%): Tests for service interactions and database operations
3. **End-to-End Tests** (10%): Full workflow tests with all components

### Testing Infrastructure
- **Test Framework**: Built-in Rust testing with `tokio::test` for async
- **Mocking**: `mockall` for creating test doubles
- **Test Containers**: `testcontainers` for PostgreSQL test instances
- **Coverage**: `tarpaulin` for code coverage reports

## Implementation Details

### 1. Project Test Structure

```
tests/
├── unit/
│   ├── auth/
│   │   ├── jwt_test.rs
│   │   └── password_test.rs
│   ├── services/
│   │   ├── user_service_test.rs
│   │   ├── board_service_test.rs
│   │   └── task_service_test.rs
│   └── models/
│       └── validation_test.rs
├── integration/
│   ├── grpc/
│   │   ├── user_endpoints_test.rs
│   │   ├── board_endpoints_test.rs
│   │   └── task_endpoints_test.rs
│   ├── database/
│   │   ├── migrations_test.rs
│   │   └── queries_test.rs
│   └── streaming/
│       └── realtime_updates_test.rs
├── common/
│   ├── fixtures.rs
│   ├── helpers.rs
│   └── test_server.rs
└── e2e/
    └── workflows_test.rs
```

### 2. Test Utilities and Helpers

```rust
// tests/common/test_server.rs
use testcontainers::{clients, images::postgres::Postgres, Container};
use diesel::{Connection, PgConnection};
use diesel_migrations::MigrationHarness;

pub struct TestContext {
    pub db_container: Container<'static, Postgres>,
    pub db_url: String,
    pub db_pool: DbPool,
    pub grpc_server: TestServer,
}

impl TestContext {
    pub async fn new() -> Self {
        // Start PostgreSQL container
        let docker = clients::Cli::default();
        let db_container = docker.run(Postgres::default());
        let db_port = db_container.get_host_port_ipv4(5432);
        let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", db_port);
        
        // Run migrations
        let mut conn = PgConnection::establish(&db_url).unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        
        // Create connection pool
        let db_pool = create_test_pool(&db_url);
        
        // Start gRPC server
        let grpc_server = TestServer::start(db_pool.clone()).await;
        
        Self {
            db_container,
            db_url,
            db_pool,
            grpc_server,
        }
    }
    
    pub async fn cleanup(self) {
        self.grpc_server.shutdown().await;
        // Container is automatically cleaned up when dropped
    }
}

// tests/common/fixtures.rs
use uuid::Uuid;
use crate::models::{User, Board, Task};

pub struct TestFixtures;

impl TestFixtures {
    pub fn create_test_user() -> NewUser {
        NewUser {
            email: format!("test-{}@example.com", Uuid::new_v4()),
            password_hash: "$argon2id$v=19$m=4096,t=3,p=1$...",
            full_name: "Test User",
            role: "member",
        }
    }
    
    pub fn create_test_board(owner_id: Uuid) -> NewBoard {
        NewBoard {
            title: format!("Test Board {}", Uuid::new_v4()),
            description: Some("Test board description"),
            owner_id,
        }
    }
    
    pub fn create_test_task(board_id: Uuid) -> NewTask {
        NewTask {
            board_id,
            title: "Test Task",
            description: Some("Test task description"),
            status: "todo",
            priority: "medium",
        }
    }
}
```

### 3. Unit Tests Implementation

```rust
// tests/unit/auth/jwt_test.rs
use crate::auth::jwt::{JwtManager, Claims};
use uuid::Uuid;
use chrono::{Utc, Duration};

#[test]
fn test_jwt_generation_and_validation() {
    let manager = JwtManager::new("test-secret".to_string(), 24);
    let user_id = Uuid::new_v4();
    
    let token = manager.generate_token(user_id, "test@example.com", "member").unwrap();
    assert!(!token.is_empty());
    
    let claims = manager.validate_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.role, "member");
}

#[test]
fn test_expired_token_validation() {
    let manager = JwtManager::new("test-secret".to_string(), -1); // Negative hours = expired
    let user_id = Uuid::new_v4();
    
    let token = manager.generate_token(user_id, "test@example.com", "member").unwrap();
    let result = manager.validate_token(&token);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::TokenExpired => (),
        _ => panic!("Expected TokenExpired error"),
    }
}

// tests/unit/services/user_service_test.rs
use mockall::predicate::*;
use crate::services::UserServiceImpl;

#[tokio::test]
async fn test_register_user_validation() {
    let mut mock_db = MockDatabase::new();
    mock_db.expect_check_user_exists()
        .with(eq("invalid-email"))
        .times(0)  // Should not reach DB due to validation
        .return_const(Ok(false));
    
    let service = UserServiceImpl::new(mock_db, "secret".to_string());
    
    let request = Request::new(RegisterUserRequest {
        email: "invalid-email".to_string(),
        password: "pass".to_string(),
        full_name: "Test".to_string(),
    });
    
    let result = service.register_user(request).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), Code::InvalidArgument);
}
```

### 4. Integration Tests

```rust
// tests/integration/grpc/user_endpoints_test.rs
use tonic::transport::Channel;
use crate::proto::user_service_client::UserServiceClient;
use crate::common::{TestContext, TestFixtures};

#[tokio::test]
async fn test_user_registration_flow() {
    let ctx = TestContext::new().await;
    let mut client = UserServiceClient::connect(ctx.grpc_server.url()).await.unwrap();
    
    let request = RegisterUserRequest {
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
        full_name: "New User".to_string(),
    };
    
    let response = client.register_user(request).await.unwrap();
    let res = response.into_inner();
    
    assert!(!res.user.unwrap().id.is_empty());
    assert!(!res.access_token.is_empty());
    
    // Verify user can login with credentials
    let login_request = LoginUserRequest {
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
    };
    
    let login_response = client.login_user(login_request).await.unwrap();
    assert!(!login_response.into_inner().access_token.is_empty());
    
    ctx.cleanup().await;
}

// tests/integration/database/queries_test.rs
#[tokio::test]
async fn test_cascade_delete_boards() {
    let ctx = TestContext::new().await;
    
    // Create user, board, and tasks
    let user = create_user(&ctx.db_pool).await;
    let board = create_board(&ctx.db_pool, user.id).await;
    let task1 = create_task(&ctx.db_pool, board.id).await;
    let task2 = create_task(&ctx.db_pool, board.id).await;
    
    // Delete board
    delete_board(&ctx.db_pool, board.id).await;
    
    // Verify tasks are also deleted
    let tasks = get_board_tasks(&ctx.db_pool, board.id).await;
    assert_eq!(tasks.len(), 0);
    
    ctx.cleanup().await;
}
```

### 5. Streaming Tests

```rust
// tests/integration/streaming/realtime_updates_test.rs
use tokio_stream::StreamExt;
use crate::proto::task_service_client::TaskServiceClient;

#[tokio::test]
async fn test_task_update_streaming() {
    let ctx = TestContext::new().await;
    let mut client = TaskServiceClient::connect(ctx.grpc_server.url()).await.unwrap();
    
    // Create test data
    let user = create_test_user(&ctx.db_pool).await;
    let board = create_test_board(&ctx.db_pool, user.id).await;
    
    // Subscribe to updates
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let subscribe_request = StreamTaskUpdatesRequest {
        request: Some(stream_task_updates_request::Request::Subscribe(
            SubscribeRequest {
                board_ids: vec![board.id.to_string()],
            }
        )),
    };
    
    let stream = client.stream_task_updates(
        tokio_stream::once(subscribe_request)
    ).await.unwrap();
    
    // Spawn task to collect updates
    let handle = tokio::spawn(async move {
        let mut stream = stream.into_inner();
        while let Some(update) = stream.next().await {
            if let Ok(update) = update {
                tx.send(update).await.unwrap();
            }
        }
    });
    
    // Create task and verify update received
    let task = create_task_via_api(&mut client, board.id).await;
    
    let update = rx.recv().await.unwrap();
    match update.update {
        Some(Update::TaskCreated(created)) => {
            assert_eq!(created.task.unwrap().id, task.id);
        }
        _ => panic!("Expected TaskCreated update"),
    }
    
    ctx.cleanup().await;
}
```

### 6. Error and Edge Case Tests

```rust
// tests/integration/error_cases_test.rs
#[tokio::test]
async fn test_concurrent_board_member_additions() {
    let ctx = TestContext::new().await;
    
    let board = create_test_board(&ctx.db_pool).await;
    let user = create_test_user(&ctx.db_pool).await;
    
    // Attempt to add same user as member concurrently
    let pool1 = ctx.db_pool.clone();
    let pool2 = ctx.db_pool.clone();
    
    let handle1 = tokio::spawn(async move {
        add_board_member(&pool1, board.id, user.id, "member").await
    });
    
    let handle2 = tokio::spawn(async move {
        add_board_member(&pool2, board.id, user.id, "admin").await
    });
    
    let (result1, result2) = tokio::join!(handle1, handle2);
    
    // One should succeed, one should fail with unique constraint
    assert!(result1.is_ok() != result2.is_ok());
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_database_connection_pool_exhaustion() {
    let ctx = TestContext::new().await;
    
    // Spawn many concurrent operations
    let mut handles = vec![];
    for _ in 0..100 {
        let pool = ctx.db_pool.clone();
        handles.push(tokio::spawn(async move {
            // Perform database operation
            get_all_users(&pool).await
        }));
    }
    
    // All should complete without pool exhaustion
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    ctx.cleanup().await;
}
```

### 7. Test Coverage Configuration

```toml
# tarpaulin.toml
[default]
exclude-files = ["*/proto.rs", "*/schema.rs", "tests/*"]
ignored = ["proto", "tests"]
timeout = "300"
all-features = true

[report]
out = ["Html", "Xml"]
output-dir = "target/coverage"
```

### 8. Test Execution Scripts

```bash
#!/bin/bash
# scripts/run-tests.sh

echo "Running unit tests..."
cargo test --lib --bins

echo "Running integration tests..."
cargo test --test '*' -- --test-threads=1

echo "Running with coverage..."
cargo tarpaulin --config tarpaulin.toml

echo "Checking test performance..."
cargo test -- --nocapture --test-threads=1 | grep "test result"
```

## Dependencies

- Tasks 2-8: All implementations must be complete for comprehensive testing
- Test containers require Docker to be running
- Database migrations must be available for test setup

## Testing Strategy

### Test Categories
1. **Unit Tests**: Test individual functions with mocked dependencies
2. **Integration Tests**: Test service interactions with real database
3. **End-to-End Tests**: Test complete user workflows
4. **Performance Tests**: Verify response times and throughput
5. **Security Tests**: Test authentication and authorization
6. **Error Tests**: Verify error handling and recovery

### Coverage Goals
- Overall coverage: > 80%
- Critical paths: 100%
- Error handling: > 90%
- Business logic: > 95%

### Test Execution
- Run tests in CI/CD pipeline
- Parallel execution where possible
- Isolated test databases
- Automatic cleanup after tests