# Orchestrator Integration Tests

This directory contains comprehensive integration tests for the orchestrator system. These tests verify the end-to-end functionality of the GitHub webhook → ConfigMap → Job pipeline.

## Test Structure

### Test Modules

- **`webhook_pipeline_tests.rs`** - Tests the complete webhook processing pipeline
- **`error_scenario_tests.rs`** - Tests error handling and edge cases
- **`idempotency_tests.rs`** - Tests that duplicate operations are handled correctly
- **`resource_cleanup_tests.rs`** - Tests Kubernetes resource management and cleanup
- **`concurrent_processing_tests.rs`** - Tests concurrent webhook processing and system behavior under load

### Test Categories

#### 1. Webhook Pipeline Tests
- Full end-to-end webhook processing
- Large payload handling
- Different GitHub issue actions (opened, edited, closed)
- Processing without Kubernetes client (mock mode)

#### 2. Error Scenario Tests
- Invalid JSON payloads
- Missing required headers
- Unsupported event types
- Payload size limits
- Malformed GitHub event structures
- Kubernetes namespace errors

#### 3. Idempotency Tests
- Duplicate webhook processing
- ConfigMap creation idempotency
- Job creation idempotency
- Concurrent processing of same events

#### 4. Resource Cleanup Tests
- ConfigMap size limits and validation
- Job resource specifications
- Resource labels and annotations
- Orphaned resource cleanup
- Resource name collision handling

#### 5. Concurrent Processing Tests
- Multiple different issues processed simultaneously
- Rate limiting behavior
- Kubernetes API stress testing
- Memory usage under load
- Error isolation between concurrent requests

## Prerequisites

### Required Setup

1. **Kubernetes Cluster Access**
   ```bash
   # Ensure kubectl is configured and cluster is accessible
   kubectl cluster-info
   
   # Create test namespace (tests will use 'test-orchestrator')
   kubectl create namespace test-orchestrator --dry-run=client -o yaml | kubectl apply -f -
   ```

2. **Required Kubernetes Secret** (for Job creation tests)
   ```bash
   # Create mock Claude API key secret
   kubectl create secret generic claude-api-key \
     --from-literal=api-key=test-key-for-integration-tests \
     -n test-orchestrator
   ```

3. **Rust Development Environment**
   ```bash
   # Ensure Rust toolchain is installed
   rustc --version
   cargo --version
   ```

### Environment Variables

Set these environment variables to control test behavior:

```bash
# Skip all integration tests
export SKIP_INTEGRATION_TESTS=1

# Skip Kubernetes-dependent tests (run only mock tests)
export SKIP_K8S_TESTS=1

# Enable verbose logging during tests
export RUST_LOG=debug

# Mark as CI environment (affects test output)
export CI=1
```

## Running Tests

### All Integration Tests
```bash
# Run the complete integration test suite
cargo test run_all_integration_tests -- --nocapture

# Run with verbose output
RUST_LOG=info cargo test run_all_integration_tests -- --nocapture
```

### Individual Test Suites
```bash
# Run only webhook pipeline tests
cargo test test_webhook_pipeline_only -- --ignored --nocapture

# Run only error scenario tests
cargo test test_error_scenarios_only -- --ignored --nocapture

# Run only idempotency tests
cargo test test_idempotency_only -- --ignored --nocapture

# Run only resource cleanup tests
cargo test test_resource_cleanup_only -- --ignored --nocapture

# Run only concurrent processing tests
cargo test test_concurrent_processing_only -- --ignored --nocapture
```

### Quick Smoke Test
```bash
# Run basic functionality check
cargo test smoke_test -- --nocapture
```

### Specific Test Functions
```bash
# Run a specific test function
cargo test test_full_webhook_pipeline -- --nocapture

# Run tests matching a pattern
cargo test concurrent -- --nocapture
```

## Test Data

The tests use realistic GitHub webhook payloads:

- **`test-data/github-issue-opened.json`** - Sample GitHub issue webhook payload
- **Generated test events** - Programmatically created events with varying content

### Test Event Generation

Tests use the `create_test_github_event()` utility function to generate consistent test data:

```rust
let test_event = create_test_github_event(
    42,                           // Issue number
    "Test issue title",           // Issue title
    "Test issue body content"     // Issue body
);
```

## Expected Behavior

### Successful Test Run

When all tests pass, you should see:

```
🚀 Starting comprehensive orchestrator integration test suite

📡 Running webhook pipeline tests...
🧪 Starting integration test: full_webhook_pipeline
✅ Integration test 'full_webhook_pipeline' passed

🚨 Running error scenario tests...
🧪 Starting integration test: invalid_json_payload
✅ Integration test 'invalid_json_payload' passed

🔄 Running idempotency tests...
🧪 Starting integration test: duplicate_webhook_idempotency
✅ Integration test 'duplicate_webhook_idempotency' passed

🧹 Running resource cleanup tests...
🧪 Starting integration test: configmap_resource_limits
✅ Integration test 'configmap_resource_limits' passed

⚡ Running concurrent processing tests...
🧪 Starting integration test: concurrent_different_issues
✅ Integration test 'concurrent_different_issues' passed

🎉 All integration tests completed successfully!
```

### Resource Verification

After successful tests, verify no resources are left behind:

```bash
# Check for test resources (should be empty)
kubectl get configmaps,jobs -n test-orchestrator -l source=github

# Clean up test namespace if needed
kubectl delete namespace test-orchestrator
```

## Performance Metrics

The tests collect and report performance metrics:

- **Request Processing Time** - End-to-end webhook processing duration
- **Kubernetes API Latency** - Time for ConfigMap and Job creation
- **Concurrent Throughput** - Number of simultaneous requests handled
- **Memory Usage** - Resource consumption under load

### Typical Performance Expectations

- **Single webhook processing**: < 2 seconds
- **ConfigMap creation**: < 500ms
- **Job creation**: < 1 second
- **Concurrent requests (10)**: All complete within 10 seconds

## Troubleshooting

### Common Issues

#### 1. Kubernetes Connection Errors
```
Error: failed to create Kubernetes client
```

**Solution**: Ensure `kubectl` is configured and cluster is accessible:
```bash
kubectl cluster-info
export KUBECONFIG=~/.kube/config
```

#### 2. Permission Errors
```
Error: configmaps is forbidden
```

**Solution**: Ensure your Kubernetes user has appropriate permissions:
```bash
# Check current context
kubectl config current-context

# Verify permissions
kubectl auth can-i create configmaps -n test-orchestrator
kubectl auth can-i create jobs -n test-orchestrator
```

#### 3. Test Namespace Missing
```
Error: namespace "test-orchestrator" not found
```

**Solution**: Create the test namespace:
```bash
kubectl create namespace test-orchestrator
```

#### 4. Secret Missing Errors
```
Error: secret "claude-api-key" not found
```

**Solution**: Create the required secret:
```bash
kubectl create secret generic claude-api-key \
  --from-literal=api-key=test-key \
  -n test-orchestrator
```

### Debug Mode

Run tests with debug logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo test run_all_integration_tests -- --nocapture
```

### Skipping Tests

If Kubernetes is not available, skip cluster-dependent tests:

```bash
SKIP_K8S_TESTS=1 cargo test run_all_integration_tests -- --nocapture
```

## Test Development

### Adding New Tests

1. **Choose the appropriate module** based on test category
2. **Follow the established patterns** for test structure
3. **Use the TestContext utility** for resource management
4. **Clean up resources** in test teardown

### Test Structure Pattern

```rust
#[tokio::test]
async fn test_new_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = TestContext::new("test-name").await?;
    ctx.cleanup_resources().await?;

    // Test setup
    let test_event = create_test_github_event(/* params */);

    // Test execution
    // ... test logic ...

    // Assertions
    assert_eq!(actual, expected);

    // Cleanup
    ctx.cleanup_resources().await?;
    Ok(())
}
```

### Best Practices

- **Use unique test names** to avoid resource conflicts
- **Clean up resources** before and after tests
- **Test both success and failure paths**
- **Verify Kubernetes resources** are created correctly
- **Use realistic test data** that matches GitHub webhook format

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Kubernetes
        uses: helm/kind-action@v1
        
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Create test namespace
        run: kubectl create namespace test-orchestrator
        
      - name: Create test secret
        run: |
          kubectl create secret generic claude-api-key \
            --from-literal=api-key=test-key \
            -n test-orchestrator
            
      - name: Run integration tests
        run: cargo test run_all_integration_tests -- --nocapture
        env:
          RUST_LOG: info
          CI: 1
```

## Security Considerations

- **Test data is synthetic** - No real GitHub tokens or sensitive data
- **Mock secrets are used** - Test secrets contain dummy values
- **Resource isolation** - Tests use dedicated namespace
- **Cleanup verification** - Tests verify no sensitive data is left behind

The integration tests are designed to validate the orchestrator's security, reliability, and performance in a realistic environment while maintaining safe testing practices.