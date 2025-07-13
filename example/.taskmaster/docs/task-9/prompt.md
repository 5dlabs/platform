# Autonomous Prompt for Task 9: Write Comprehensive Unit and Integration Tests

## Context

You are implementing a comprehensive test suite for the Task Board API, a gRPC-based collaborative task management service. The system has been fully implemented with user authentication, board management, task CRUD operations, and real-time streaming. Your goal is to ensure the system is reliable, maintainable, and performs correctly under various conditions.

### Project State
- All service implementations (Tasks 4-8) are complete
- Database schema and models are implemented with Diesel ORM
- gRPC endpoints are fully functional
- Real-time streaming is operational
- Logging and error handling are in place

### Testing Philosophy
- Follow the test pyramid: many unit tests, fewer integration tests, minimal E2E tests
- Test behavior, not implementation details
- Ensure tests are deterministic and isolated
- Mock external dependencies in unit tests
- Use real components in integration tests

## Task Requirements

### Primary Objectives
1. Implement unit tests for all service methods and utility functions
2. Create integration tests for gRPC endpoints
3. Test database operations and constraints
4. Verify real-time streaming functionality
5. Test error handling and edge cases
6. Set up test infrastructure with containers
7. Configure code coverage reporting

### Test Coverage Requirements
- **Unit Tests**: Cover all business logic, authentication, and validation
- **Integration Tests**: Test all gRPC endpoints with real database
- **Database Tests**: Verify migrations, constraints, and transactions
- **Streaming Tests**: Test real-time updates and connection management
- **Error Tests**: Verify error handling and recovery mechanisms
- **Performance Tests**: Basic load testing and concurrent operations

## Implementation Instructions

### Step 1: Set Up Test Infrastructure
```bash
# Add test dependencies to Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
testcontainers = "0.15"
serial_test = "2.0"
```

Create test directory structure:
```
tests/
├── unit/
├── integration/
├── common/
└── e2e/
```

### Step 2: Create Test Utilities
Implement shared test utilities in `tests/common/`:
- `test_server.rs`: gRPC test server setup
- `fixtures.rs`: Test data generators
- `helpers.rs`: Database and authentication helpers
- `containers.rs`: PostgreSQL container management

### Step 3: Implement Unit Tests
For each service module, create corresponding test files:
- Mock dependencies using `mockall`
- Test success and failure paths
- Verify input validation
- Test business logic edge cases

Example structure for user service tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        DbPool {
            fn get_user(&self, id: Uuid) -> Result<User, Error>;
        }
    }
    
    #[tokio::test]
    async fn test_get_user_success() {
        // Arrange
        let mut mock_db = MockDbPool::new();
        mock_db.expect_get_user()
            .returning(|_| Ok(test_user()));
            
        // Act & Assert
    }
}
```

### Step 4: Create Integration Tests
Test each gRPC service with real components:
1. Start PostgreSQL container
2. Run migrations
3. Start gRPC server
4. Execute test scenarios
5. Clean up resources

Use `TestContext` pattern for setup/teardown:
```rust
struct TestContext {
    container: Container<Postgres>,
    db_pool: DbPool,
    server: TestServer,
}

impl TestContext {
    async fn new() -> Self { /* setup */ }
    async fn cleanup(self) { /* teardown */ }
}
```

### Step 5: Test Database Operations
Create database-specific tests:
- Migration up/down testing
- Constraint validation (unique, foreign key)
- Transaction rollback scenarios
- Cascade delete verification
- Index performance testing

### Step 6: Implement Streaming Tests
Test real-time functionality:
- Client subscription/unsubscription
- Update broadcasting
- Connection lifecycle
- Error propagation in streams
- Multiple concurrent clients

### Step 7: Error and Edge Case Testing
Cover failure scenarios:
- Invalid authentication tokens
- Expired tokens
- Database connection failures
- Concurrent modifications
- Resource exhaustion
- Network interruptions

### Step 8: Configure Test Execution
Create test scripts and configuration:
- Sequential execution for integration tests
- Parallel execution for unit tests
- Coverage collection with tarpaulin
- CI/CD integration

## Success Criteria

### Test Implementation
- [ ] All service methods have unit tests
- [ ] All gRPC endpoints have integration tests
- [ ] Database operations are thoroughly tested
- [ ] Streaming functionality is verified
- [ ] Error cases are covered
- [ ] Test utilities are reusable and well-organized

### Coverage Metrics
- [ ] Overall code coverage > 80%
- [ ] Critical path coverage = 100%
- [ ] Error handling coverage > 90%
- [ ] No untested public APIs

### Test Quality
- [ ] Tests are deterministic (no flaky tests)
- [ ] Tests run in isolation
- [ ] Tests are fast (unit tests < 10ms each)
- [ ] Tests have clear assertions
- [ ] Test names describe behavior

### Infrastructure
- [ ] Test containers work reliably
- [ ] Database is properly isolated
- [ ] Resources are cleaned up
- [ ] Tests can run in CI/CD
- [ ] Coverage reports are generated

## Common Pitfalls to Avoid

1. **Test Isolation**: Each test must be independent
2. **Database State**: Always use fresh database for each test
3. **Async Testing**: Use `tokio::test` for async tests
4. **Resource Cleanup**: Ensure containers and connections are closed
5. **Time-based Tests**: Mock time functions for deterministic results
6. **Coverage Gaming**: Don't write meaningless tests for coverage

## Testing Commands

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests (sequential)
cargo test --test '*' -- --test-threads=1

# Run with coverage
cargo tarpaulin --out Html

# Run specific test
cargo test test_user_registration

# Run tests with output
cargo test -- --nocapture
```

## Expected Deliverables

1. Complete test suite in `tests/` directory
2. Test utilities in `tests/common/`
3. Unit tests achieving >80% coverage
4. Integration tests for all endpoints
5. Database constraint tests
6. Streaming functionality tests
7. Coverage report configuration
8. CI/CD test execution setup

Remember: Good tests are an investment in the project's future. They enable confident refactoring, catch regressions early, and serve as living documentation of the system's behavior.