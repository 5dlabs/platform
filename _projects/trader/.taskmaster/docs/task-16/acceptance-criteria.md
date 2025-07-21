# Task 16: Acceptance Criteria

## Functional Requirements

### 1. Encrypted Keypair Storage
- [ ] **Encryption Implementation**:
  - Uses AES-256-GCM from `ring` crate
  - PBKDF2 with exactly 100,000 iterations
  - Generates cryptographically secure 32-byte salt
  - Creates unique nonce for each encryption operation
- [ ] **File Storage**:
  - Stores encrypted wallets in configurable directory
  - File format includes wallet ID, salt, nonce, ciphertext
  - Sets file permissions to 0600 (owner read/write only) on Unix
  - Implements file locking during read/write operations
- [ ] **Password Security**:
  - Derives 256-bit key from password using PBKDF2
  - Clears password from memory after use
  - Validates password strength (minimum requirements)

### 2. Secure Memory Management
- [ ] **Secrecy Integration**:
  - All `Keypair` instances wrapped in `Secret<Keypair>`
  - Password data wrapped in `SecretVec<u8>`
  - Automatic zeroization on drop verified
  - No sensitive data exposed via Debug trait
- [ ] **Memory Lifecycle**:
  - Private keys exist in memory only during signing
  - Immediate cleanup after cryptographic operations
  - No key material in stack traces or core dumps
- [ ] **Logging Safety**:
  - Custom Debug implementations prevent key exposure
  - Audit logs contain only non-sensitive metadata
  - Error messages sanitized of sensitive content

### 3. Key Rotation Functionality
- [ ] **Rotation Schedule**:
  - Default daily rotation (configurable interval)
  - Automatic trigger based on schedule
  - Manual rotation on demand
  - Tracks rotation count and history
- [ ] **Transaction Continuity**:
  - Maintains last 3 keypairs for overlap period
  - Signs with previous keys if current fails
  - No transaction interruption during rotation
  - Graceful handling of in-flight transactions
- [ ] **Rotation Process**:
  - Atomic operation with rollback on failure
  - Generates new keypair securely
  - Updates all references consistently
  - Audit log entry for each rotation

### 4. Transaction Signing
- [ ] **Signing Operations**:
  - Signs both legacy and versioned transactions
  - Manages recent blockhash automatically
  - Supports custom blockhash when provided
  - Returns fully signed transaction ready for submission
- [ ] **Rate Limiting**:
  - Minimum 10ms between signing operations
  - Configurable rate limit thresholds
  - Returns specific error on limit exceeded
  - Per-wallet rate limit tracking
- [ ] **Performance Tracking**:
  - Records signing latency metrics
  - Tracks success/failure rates
  - Monitors rate limit violations
  - Exports metrics for monitoring

### 5. Security Measures
- [ ] **Audit Logging**:
  - Logs all wallet creation events
  - Records every signing attempt (success/failure)
  - Tracks key rotation events
  - Captures failed decryption attempts
  - Stores in PostgreSQL with indexes
- [ ] **Tamper Detection**:
  - Computes SHA-256 checksum for each wallet file
  - Verifies integrity before loading
  - Updates checksum after modifications
  - Raises specific error on tampering
- [ ] **Access Control**:
  - Tracks failed decryption attempts
  - Locks wallet after threshold (default: 5)
  - Implements exponential backoff
  - Requires manual unlock after lockout

### 6. Wallet Manager API
- [ ] **Builder Pattern**:
  - Configurable storage path
  - Adjustable rotation interval
  - Custom audit logger integration
  - Validation of all parameters
- [ ] **Public Interface**:
  - `create_wallet()` returns public key
  - `load_wallet()` returns secure manager
  - `rotate_if_needed()` checks and rotates
  - Clean error types for all failures
- [ ] **Error Handling**:
  - Specific error types for each failure mode
  - No sensitive data in error messages
  - Proper error propagation
  - Recovery suggestions in errors

## Non-Functional Requirements

### Performance
- [ ] Key derivation completes in 50-100ms
- [ ] Transaction signing <10ms (excluding derivation)
- [ ] Rotation process <1 second total
- [ ] Concurrent signing operations supported
- [ ] No memory leaks under sustained load

### Security
- [ ] Passes static analysis with `cargo-audit`
- [ ] No compiler warnings with strict lints
- [ ] Timing attack resistant operations
- [ ] Constant-time password comparison
- [ ] Side-channel attack considerations

### Code Quality
- [ ] 90%+ test coverage for security-critical paths
- [ ] All public APIs documented
- [ ] Examples for common operations
- [ ] Security considerations documented
- [ ] Clean separation of concerns

## Test Cases

### Encryption Tests
```rust
// Test 1: Successful encryption/decryption
Input: Keypair, password="Test123!@#"
Expected: Encrypted file created, successful roundtrip

// Test 2: Wrong password decryption
Input: Encrypted wallet, wrong_password="Wrong123"
Expected: DecryptionError after constant time

// Test 3: Corrupted ciphertext
Input: Modified encrypted data
Expected: DecryptionError with integrity failure
```

### Memory Security Tests
```rust
// Test 1: Zeroization verification
Process: Create Secret<Keypair>, drop it
Expected: Memory location zeroed (0x00 bytes)

// Test 2: No logging of secrets
Process: Debug print Secret wrapper
Expected: Output shows "[REDACTED]" or similar

// Test 3: Stack trace safety
Process: Trigger panic during signing
Expected: No key material in stack trace
```

### Rotation Tests
```rust
// Test 1: Scheduled rotation
Setup: Set 1-hour rotation interval
Wait: 61 minutes
Expected: Automatic rotation triggered

// Test 2: Continuity during rotation
Process: Start signing, trigger rotation mid-operation
Expected: Signing completes with old or new key

// Test 3: Multiple rotation handling
Process: Rotate 5 times rapidly
Expected: Last 3 keys maintained, oldest dropped
```

### Rate Limiting Tests
```rust
// Test 1: Normal operation spacing
Input: Sign with 15ms intervals
Expected: All operations succeed

// Test 2: Rapid signing attempts
Input: 5 signs with 5ms intervals
Expected: First succeeds, others rate limited

// Test 3: Rate limit recovery
Input: Hit limit, wait 15ms, retry
Expected: Operation succeeds after wait
```

### Security Scenario Tests
```rust
// Test 1: Brute force resistance
Attack: 1000 decryption attempts/second
Expected: <1% success chance in 24 hours

// Test 2: Tamper detection
Attack: Modify 1 byte in wallet file
Expected: TamperDetected error on load

// Test 3: Timing attack resistance
Attack: Measure decryption time variance
Expected: <5% timing difference for success/fail
```

## Definition of Done

- [ ] All functional requirements implemented and tested
- [ ] Security test suite passes 100%
- [ ] Performance benchmarks meet targets
- [ ] No security vulnerabilities in dependencies
- [ ] Code review by security-focused developer
- [ ] Integration test with live trader service
- [ ] Audit log entries verified in database
- [ ] Documentation includes security best practices
- [ ] Stress test under production-like load
- [ ] Manual penetration testing attempted