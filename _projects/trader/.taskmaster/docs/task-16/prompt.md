# Task 16: Implement Enhanced Wallet Manager for Live Trading - Autonomous Prompt

You are implementing a secure wallet management system for a high-frequency Solana trading platform. Your goal is to create a robust wallet manager that handles encrypted keypair storage, secure transaction signing, and automated key rotation while maintaining the highest security standards.

## Context

The live trading service requires secure management of Solana keypairs with minimal latency for transaction signing. The system must protect private keys from exposure while supporting thousands of transactions per day with automatic daily key rotation as specified in the PRD.

## Your Objectives

1. **Implement Encrypted Keypair Storage**
   - Use the `ring` crate for AES-256-GCM encryption
   - Implement PBKDF2 with 100,000 iterations for key derivation
   - Create secure file format with wallet metadata
   - Set restrictive file permissions (0600 on Unix)
   - Support multiple wallet storage with unique identifiers

2. **Develop Secure Memory Management**
   - Use the `secrecy` crate to wrap sensitive data
   - Implement automatic memory zeroization on drop
   - Minimize private key lifetime in memory
   - Prevent key material from appearing in logs or dumps
   - Create secure wrapper types for all sensitive operations

3. **Build Key Rotation System**
   - Implement daily automatic rotation schedule
   - Maintain transaction continuity during rotation
   - Keep last 3 wallets for graceful transition
   - Create atomic rotation with rollback capability
   - Log all rotation events for audit trail

4. **Create Transaction Signing Infrastructure**
   - Implement secure signing with recent blockhash management
   - Support both legacy and versioned transactions
   - Add rate limiting to prevent abuse
   - Minimize key exposure during signing operations
   - Track signing metrics and performance

5. **Implement Security Measures**
   - Create comprehensive audit logging system
   - Add tamper detection using SHA-256 checksums
   - Implement rate limiting for failed attempts
   - Create secure backup and recovery mechanisms
   - Add monitoring for suspicious activities

## Implementation Requirements

### Code Structure
```
live_trader/wallet/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── storage.rs      # Encrypted storage implementation
│   ├── manager.rs      # Secure wallet manager
│   ├── rotation.rs     # Key rotation logic
│   ├── signing.rs      # Transaction signing
│   ├── security.rs     # Security measures
│   └── errors.rs       # Error types
```

### Key Security Requirements

1. **Encryption Standards**:
   - AES-256-GCM for keypair encryption
   - PBKDF2-HMAC-SHA256 with 100k iterations
   - Cryptographically secure random nonces
   - Secure key derivation from passwords

2. **Memory Protection**:
   - All sensitive data wrapped in `Secret<T>`
   - Automatic zeroization on drop
   - No logging of sensitive material
   - Minimal exposure time for keys

3. **Access Control**:
   - File permissions restricted to owner
   - Rate limiting on operations
   - Audit logging for all actions
   - Tamper detection for stored files

### API Design

```rust
// Builder pattern for configuration
let wallet_manager = WalletManagerBuilder::new()
    .with_storage_path("/secure/wallets")
    .with_rotation_interval(Duration::days(1))
    .with_audit_logger(audit_logger)
    .with_max_failed_attempts(5)
    .build()?;

// Clean public API
let pubkey = wallet_manager.create_wallet("main_wallet", password).await?;
let wallet = wallet_manager.load_wallet("main_wallet", password).await?;
let signed_tx = wallet.sign_transaction(transaction).await?;
```

### Testing Requirements

1. **Security Tests**:
   - Verify no key material in logs
   - Test timing attack resistance
   - Validate brute force protection
   - Check memory zeroization

2. **Functional Tests**:
   - Encryption/decryption roundtrip
   - Key rotation scenarios
   - Concurrent signing operations
   - Rate limiting behavior

3. **Integration Tests**:
   - Full wallet lifecycle
   - Audit logging verification
   - Tamper detection validation
   - Performance under load

### Performance Targets

- Key derivation: 50-100ms (acceptable for security)
- Transaction signing: <10ms including checks
- Rotation process: <1 second complete
- Concurrent operations: Thread-safe with minimal contention

## Deliverables

1. Complete wallet manager implementation with all security features
2. Comprehensive test suite covering security scenarios
3. Integration with audit logging system
4. Documentation for security properties and usage
5. Performance benchmarks for critical operations

## Success Criteria

- Zero key exposure in logs or memory dumps
- All security tests pass including timing attacks
- Automatic daily rotation works reliably
- Transaction signing maintains <10ms latency
- Audit trail captures all wallet operations
- Code passes security-focused review
- Integration with live trader service successful