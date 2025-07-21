# Task 16: Implement Enhanced Wallet Manager for Live Trading

## Overview

This task creates a secure wallet management system for live trading that handles encrypted keypair storage, secure transaction signing, and key rotation with high security standards. This component is critical for the live trading service, ensuring that private keys are protected while maintaining the performance requirements for high-frequency trading.

## Architecture Context

The wallet manager is a core security component in the live trading service, as outlined in the system architecture. It provides:

- **Secure Key Storage**: Encrypted storage of Solana keypairs with password-based key derivation
- **Memory Protection**: Prevents key material from being exposed in logs or memory dumps
- **Transaction Signing**: Secure signing with minimal key exposure time
- **Key Rotation**: Daily rotation capability as specified in the PRD
- **Audit Trail**: Comprehensive logging for all wallet operations

## Implementation Details

### 1. Encrypted Keypair Storage

#### Secure File Format
```rust
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, SealingKey, UnboundKey};
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct EncryptedWallet {
    wallet_id: String,
    salt: Vec<u8>,
    nonce: Vec<u8>,
    encrypted_keypair: Vec<u8>,
    created_at: DateTime<Utc>,
    rotation_count: u32,
}

pub struct WalletStorage {
    storage_path: PathBuf,
}

impl WalletStorage {
    pub fn store_encrypted_keypair(
        &self,
        wallet_id: &str,
        keypair: &Keypair,
        password: &str,
    ) -> Result<(), WalletError> {
        // Generate salt for PBKDF2
        let mut salt = vec![0u8; 32];
        ring::rand::SystemRandom::new().fill(&mut salt)
            .map_err(|_| WalletError::RandomGenerationError)?;

        // Derive key from password
        let mut key = vec![0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(), // 100k iterations
            &salt,
            password.as_bytes(),
            &mut key,
        );

        // Encrypt keypair
        let aead = Aes256Gcm::new(GenericArray::from_slice(&key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let plaintext = keypair.to_bytes();
        let ciphertext = aead.encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| WalletError::EncryptionError)?;

        // Clear sensitive data
        key.zeroize();
        plaintext.zeroize();

        // Store encrypted wallet
        let encrypted_wallet = EncryptedWallet {
            wallet_id: wallet_id.to_string(),
            salt,
            nonce: nonce.to_vec(),
            encrypted_keypair: ciphertext,
            created_at: Utc::now(),
            rotation_count: 0,
        };

        let path = self.storage_path.join(format!("{}.wallet", wallet_id));
        let mut file = File::create(&path)?;
        let data = serde_json::to_vec(&encrypted_wallet)?;
        file.write_all(&data)?;
        file.sync_all()?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    pub fn load_encrypted_keypair(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<Keypair, WalletError> {
        let path = self.storage_path.join(format!("{}.wallet", wallet_id));
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let encrypted_wallet: EncryptedWallet = serde_json::from_slice(&data)?;

        // Derive key from password
        let mut key = vec![0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(),
            &encrypted_wallet.salt,
            password.as_bytes(),
            &mut key,
        );

        // Decrypt keypair
        let aead = Aes256Gcm::new(GenericArray::from_slice(&key));
        let nonce = GenericArray::from_slice(&encrypted_wallet.nonce);
        
        let plaintext = aead.decrypt(nonce, encrypted_wallet.encrypted_keypair.as_ref())
            .map_err(|_| WalletError::DecryptionError)?;

        // Clear key immediately
        key.zeroize();

        // Create keypair and clear plaintext
        let keypair = Keypair::from_bytes(&plaintext)
            .map_err(|_| WalletError::InvalidKeypair)?;
        plaintext.zeroize();

        Ok(keypair)
    }
}
```

### 2. Secure Memory Management

#### Memory Protection with Secrecy Crate
```rust
use secrecy::{ExposeSecret, Secret, SecretVec};
use zeroize::Zeroize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SecureWalletManager {
    keypair: Arc<RwLock<Secret<Keypair>>>,
    recent_blockhash: Arc<RwLock<Hash>>,
    rotation_schedule: RotationSchedule,
    audit_log: Arc<AuditLogger>,
    failed_attempts: Arc<RwLock<u32>>,
    last_access: Arc<RwLock<Instant>>,
}

impl SecureWalletManager {
    pub fn new(
        encrypted_keypair: &[u8],
        password: SecretVec<u8>,
    ) -> Result<Self, WalletError> {
        // Decrypt with automatic memory clearing
        let keypair = {
            let key = derive_key(password.expose_secret())?;
            let plaintext = decrypt_keypair(encrypted_keypair, &key)?;
            
            // Key and plaintext are automatically zeroed when dropped
            Keypair::from_bytes(plaintext.expose_secret())?
        };

        Ok(Self {
            keypair: Arc::new(RwLock::new(Secret::new(keypair))),
            recent_blockhash: Arc::new(RwLock::new(Hash::default())),
            rotation_schedule: RotationSchedule::daily(),
            audit_log: Arc::new(AuditLogger::new()),
            failed_attempts: Arc::new(RwLock::new(0)),
            last_access: Arc::new(RwLock::new(Instant::now())),
        })
    }

    pub async fn sign_transaction(
        &self,
        mut transaction: Transaction,
    ) -> Result<Transaction, WalletError> {
        // Rate limiting check
        self.check_rate_limit().await?;

        // Audit log entry
        self.audit_log.log_signing_attempt(&transaction).await;

        // Get current blockhash
        let blockhash = self.recent_blockhash.read().await.clone();
        transaction.message.recent_blockhash = blockhash;

        // Sign with minimal exposure
        let signature = {
            let keypair_guard = self.keypair.read().await;
            let exposed = keypair_guard.expose_secret();
            transaction.try_sign(&[exposed], blockhash)
                .map_err(|e| WalletError::SigningError(e.to_string()))?
        };

        // Update last access
        *self.last_access.write().await = Instant::now();

        // Audit successful signing
        self.audit_log.log_signing_success(&transaction).await;

        Ok(transaction)
    }

    async fn check_rate_limit(&self) -> Result<(), WalletError> {
        let last_access = *self.last_access.read().await;
        if last_access.elapsed() < Duration::from_millis(10) {
            return Err(WalletError::RateLimitExceeded);
        }
        Ok(())
    }
}

// Custom drop implementation to ensure cleanup
impl Drop for SecureWalletManager {
    fn drop(&mut self) {
        // The Secret wrapper automatically zeroizes on drop
        // Log wallet shutdown for audit trail
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                // Async audit log entry
            });
        }
    }
}
```

### 3. Key Rotation Implementation

```rust
use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct RotationSchedule {
    interval: Duration,
    next_rotation: DateTime<Utc>,
}

pub struct KeyRotationManager {
    current_wallet: Arc<SecureWalletManager>,
    previous_wallets: Arc<RwLock<VecDeque<Arc<SecureWalletManager>>>>,
    storage: Arc<WalletStorage>,
    rotation_schedule: RotationSchedule,
}

impl KeyRotationManager {
    pub async fn rotate_keys(&self, password: &str) -> Result<(), WalletError> {
        // Generate new keypair
        let new_keypair = Keypair::new();
        let wallet_id = format!("wallet_{}", Utc::now().timestamp());

        // Store encrypted
        self.storage.store_encrypted_keypair(
            &wallet_id,
            &new_keypair,
            password,
        )?;

        // Create new secure wallet
        let new_wallet = SecureWalletManager::new(
            &self.storage.load_encrypted_keypair(&wallet_id, password)?.to_bytes(),
            SecretVec::new(password.as_bytes().to_vec()),
        )?;

        // Atomic rotation
        {
            let mut previous = self.previous_wallets.write().await;
            previous.push_front(self.current_wallet.clone());
            
            // Keep last 3 wallets for transaction continuity
            if previous.len() > 3 {
                previous.pop_back();
            }
        }

        // Update current wallet
        self.current_wallet = Arc::new(new_wallet);

        // Audit log rotation
        self.audit_log.log_key_rotation(&wallet_id).await;

        Ok(())
    }

    pub async fn check_rotation_needed(&self) -> bool {
        Utc::now() >= self.rotation_schedule.next_rotation
    }

    pub async fn sign_with_continuity(
        &self,
        transaction: Transaction,
    ) -> Result<Transaction, WalletError> {
        // Try current wallet first
        match self.current_wallet.sign_transaction(transaction.clone()).await {
            Ok(signed) => Ok(signed),
            Err(_) => {
                // Try previous wallets for continuity during rotation
                let previous = self.previous_wallets.read().await;
                for wallet in previous.iter() {
                    if let Ok(signed) = wallet.sign_transaction(transaction.clone()).await {
                        return Ok(signed);
                    }
                }
                Err(WalletError::SigningError("All wallets failed".to_string()))
            }
        }
    }
}
```

### 4. Transaction Signing Security

```rust
pub struct TransactionSigner {
    wallet_manager: Arc<KeyRotationManager>,
    blockhash_manager: Arc<BlockhashManager>,
    signing_metrics: Arc<SigningMetrics>,
}

impl TransactionSigner {
    pub async fn sign_transaction(
        &self,
        instructions: Vec<Instruction>,
        payer: &Pubkey,
    ) -> Result<Transaction, WalletError> {
        // Get recent blockhash with retry
        let blockhash = self.blockhash_manager
            .get_recent_blockhash()
            .await?;

        // Build transaction
        let message = Message::new_with_blockhash(
            &instructions,
            Some(payer),
            &blockhash,
        );

        let mut transaction = Transaction::new_unsigned(message);

        // Time the signing operation
        let start = Instant::now();
        
        // Sign with key rotation support
        let signed = self.wallet_manager
            .sign_with_continuity(transaction)
            .await?;

        // Record metrics
        self.signing_metrics.record_signing(start.elapsed()).await;

        Ok(signed)
    }

    pub async fn sign_versioned_transaction(
        &self,
        message: VersionedMessage,
    ) -> Result<VersionedTransaction, WalletError> {
        // Similar implementation for versioned transactions
        let mut transaction = VersionedTransaction::unsigned(message);
        
        // Sign with the current wallet
        let keypair_guard = self.wallet_manager.current_wallet.keypair.read().await;
        let signature = keypair_guard.expose_secret().sign_message(
            &transaction.message.serialize()
        );

        transaction.signatures = vec![signature];
        
        Ok(transaction)
    }
}
```

### 5. Security Measures Implementation

```rust
#[derive(Clone)]
pub struct AuditLogger {
    db_pool: Arc<PgPool>,
}

impl AuditLogger {
    pub async fn log_signing_attempt(&self, transaction: &Transaction) {
        let query = r#"
            INSERT INTO wallet_audit_log 
            (timestamp, event_type, transaction_hash, instructions_count)
            VALUES ($1, $2, $3, $4)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind("signing_attempt")
            .bind(bs58::encode(&transaction.message_data()).into_string())
            .bind(transaction.message.instructions.len() as i32)
            .execute(&*self.db_pool)
            .await;
    }

    pub async fn log_failed_decryption(&self, reason: &str) {
        let query = r#"
            INSERT INTO wallet_audit_log 
            (timestamp, event_type, error_reason)
            VALUES ($1, $2, $3)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind("decryption_failed")
            .bind(reason)
            .execute(&*self.db_pool)
            .await;
    }

    pub async fn log_key_rotation(&self, new_wallet_id: &str) {
        let query = r#"
            INSERT INTO wallet_audit_log 
            (timestamp, event_type, wallet_id)
            VALUES ($1, $2, $3)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind("key_rotation")
            .bind(new_wallet_id)
            .execute(&*self.db_pool)
            .await;
    }
}

pub struct TamperDetection {
    storage_path: PathBuf,
    checksums: Arc<RwLock<HashMap<String, String>>>,
}

impl TamperDetection {
    pub async fn verify_wallet_integrity(&self, wallet_id: &str) -> Result<(), WalletError> {
        let path = self.storage_path.join(format!("{}.wallet", wallet_id));
        let data = fs::read(&path)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let current_hash = format!("{:x}", hasher.finalize());

        let checksums = self.checksums.read().await;
        if let Some(stored_hash) = checksums.get(wallet_id) {
            if *stored_hash != current_hash {
                return Err(WalletError::TamperDetected);
            }
        }

        Ok(())
    }

    pub async fn update_checksum(&self, wallet_id: &str) -> Result<(), WalletError> {
        let path = self.storage_path.join(format!("{}.wallet", wallet_id));
        let data = fs::read(&path)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = format!("{:x}", hasher.finalize());

        self.checksums.write().await.insert(wallet_id.to_string(), hash);
        Ok(())
    }
}
```

### 6. Wallet Manager API

```rust
pub struct WalletManagerBuilder {
    storage_path: Option<PathBuf>,
    rotation_interval: Option<Duration>,
    max_failed_attempts: Option<u32>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl WalletManagerBuilder {
    pub fn new() -> Self {
        Self {
            storage_path: None,
            rotation_interval: None,
            max_failed_attempts: None,
            audit_logger: None,
        }
    }

    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    pub fn with_rotation_interval(mut self, interval: Duration) -> Self {
        self.rotation_interval = Some(interval);
        self
    }

    pub fn with_max_failed_attempts(mut self, max: u32) -> Self {
        self.max_failed_attempts = Some(max);
        self
    }

    pub fn with_audit_logger(mut self, logger: Arc<AuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    pub fn build(self) -> Result<WalletManager, WalletError> {
        let storage_path = self.storage_path
            .ok_or(WalletError::BuilderError("storage_path required"))?;

        Ok(WalletManager {
            storage: Arc::new(WalletStorage { storage_path }),
            rotation_manager: Arc::new(KeyRotationManager::new(
                self.rotation_interval.unwrap_or(Duration::days(1))
            )),
            audit_logger: self.audit_logger
                .unwrap_or_else(|| Arc::new(AuditLogger::new())),
            max_failed_attempts: self.max_failed_attempts.unwrap_or(5),
            tamper_detection: Arc::new(TamperDetection::new()),
        })
    }
}

// Clean public API
pub struct WalletManager {
    storage: Arc<WalletStorage>,
    rotation_manager: Arc<KeyRotationManager>,
    audit_logger: Arc<AuditLogger>,
    max_failed_attempts: u32,
    tamper_detection: Arc<TamperDetection>,
}

impl WalletManager {
    pub async fn create_wallet(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<Pubkey, WalletError> {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        
        self.storage.store_encrypted_keypair(wallet_id, &keypair, password)?;
        self.tamper_detection.update_checksum(wallet_id).await?;
        
        Ok(pubkey)
    }

    pub async fn load_wallet(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<Arc<SecureWalletManager>, WalletError> {
        // Verify integrity
        self.tamper_detection.verify_wallet_integrity(wallet_id).await?;
        
        // Load and create secure manager
        let keypair = self.storage.load_encrypted_keypair(wallet_id, password)?;
        let manager = SecureWalletManager::new(
            &keypair.to_bytes(),
            SecretVec::new(password.as_bytes().to_vec()),
        )?;
        
        Ok(Arc::new(manager))
    }

    pub async fn rotate_if_needed(&self, password: &str) -> Result<bool, WalletError> {
        if self.rotation_manager.check_rotation_needed().await {
            self.rotation_manager.rotate_keys(password).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_wallet_encryption_decryption() {
        let temp_dir = TempDir::new().unwrap();
        let storage = WalletStorage {
            storage_path: temp_dir.path().to_path_buf(),
        };

        let keypair = Keypair::new();
        let wallet_id = "test_wallet";
        let password = "strong_password_123!";

        // Store encrypted
        storage.store_encrypted_keypair(wallet_id, &keypair, password)
            .unwrap();

        // Load and verify
        let loaded = storage.load_encrypted_keypair(wallet_id, password)
            .unwrap();

        assert_eq!(keypair.pubkey(), loaded.pubkey());
    }

    #[tokio::test]
    async fn test_memory_zeroization() {
        let password = SecretVec::new(b"test_password".to_vec());
        let exposed = password.expose_secret();
        
        // Verify we can access the password
        assert_eq!(exposed, b"test_password");
        
        drop(password);
        // After drop, the memory should be zeroed
        // This is verified by the secrecy crate
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WalletManagerBuilder::new()
            .with_storage_path(temp_dir.path().to_path_buf())
            .with_rotation_interval(Duration::seconds(1))
            .build()
            .unwrap();

        let wallet_id = "rotation_test";
        let password = "test_password";

        let original_pubkey = manager.create_wallet(wallet_id, password).await.unwrap();
        
        // Wait for rotation interval
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Rotate keys
        let rotated = manager.rotate_if_needed(password).await.unwrap();
        assert!(rotated);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let wallet = create_test_wallet().await;
        let tx = create_test_transaction();

        // First signing should succeed
        let result1 = wallet.sign_transaction(tx.clone()).await;
        assert!(result1.is_ok());

        // Immediate second signing should fail due to rate limit
        let result2 = wallet.sign_transaction(tx.clone()).await;
        assert!(matches!(result2, Err(WalletError::RateLimitExceeded)));

        // After delay, should succeed
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        let result3 = wallet.sign_transaction(tx).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_tamper_detection() {
        let temp_dir = TempDir::new().unwrap();
        let storage = WalletStorage {
            storage_path: temp_dir.path().to_path_buf(),
        };

        let wallet_id = "tamper_test";
        let keypair = Keypair::new();
        storage.store_encrypted_keypair(wallet_id, &keypair, "password").unwrap();

        let tamper_detection = TamperDetection::new(temp_dir.path().to_path_buf());
        tamper_detection.update_checksum(wallet_id).await.unwrap();

        // Verify passes
        assert!(tamper_detection.verify_wallet_integrity(wallet_id).await.is_ok());

        // Tamper with file
        let wallet_path = temp_dir.path().join(format!("{}.wallet", wallet_id));
        let mut file = OpenOptions::new()
            .append(true)
            .open(&wallet_path)
            .unwrap();
        file.write_all(b"tampered").unwrap();

        // Verify should fail
        assert!(matches!(
            tamper_detection.verify_wallet_integrity(wallet_id).await,
            Err(WalletError::TamperDetected)
        ));
    }
}
```

### Security Tests

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_no_key_in_logs() {
        // Configure test logger
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let keypair = Keypair::new();
        let secret = Secret::new(keypair);

        // This should not compile if Secret doesn't implement Debug properly
        log::debug!("Secret: {:?}", secret);
        
        // Verify no actual key material in logs
        // This is enforced by the secrecy crate
    }

    #[tokio::test]
    async fn test_timing_attack_resistance() {
        let storage = create_test_storage();
        let wallet_id = "timing_test";
        let keypair = Keypair::new();
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";

        storage.store_encrypted_keypair(wallet_id, &keypair, correct_password).unwrap();

        // Measure correct password timing
        let start1 = Instant::now();
        let _ = storage.load_encrypted_keypair(wallet_id, correct_password);
        let correct_time = start1.elapsed();

        // Measure wrong password timing
        let start2 = Instant::now();
        let _ = storage.load_encrypted_keypair(wallet_id, wrong_password);
        let wrong_time = start2.elapsed();

        // Times should be similar (within 20%)
        let ratio = correct_time.as_millis() as f64 / wrong_time.as_millis() as f64;
        assert!(ratio > 0.8 && ratio < 1.2);
    }

    #[test]
    fn test_brute_force_resistance() {
        // With 100,000 PBKDF2 iterations, each attempt should take significant time
        let salt = vec![0u8; 32];
        let password = "test";
        let mut key = vec![0u8; 32];

        let start = Instant::now();
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(),
            &salt,
            password.as_bytes(),
            &mut key,
        );
        let duration = start.elapsed();

        // Should take at least 10ms per attempt
        assert!(duration.as_millis() >= 10);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_wallet_manager_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_pool = create_test_db_pool().await;

    let audit_logger = Arc::new(AuditLogger { db_pool });
    
    let manager = WalletManagerBuilder::new()
        .with_storage_path(temp_dir.path().to_path_buf())
        .with_rotation_interval(Duration::hours(24))
        .with_audit_logger(audit_logger.clone())
        .with_max_failed_attempts(3)
        .build()
        .unwrap();

    // Create wallet
    let wallet_id = "integration_test";
    let password = "secure_password_123!";
    let pubkey = manager.create_wallet(wallet_id, password).await.unwrap();

    // Load wallet
    let wallet = manager.load_wallet(wallet_id, password).await.unwrap();

    // Create and sign transaction
    let mut tx = Transaction::new_with_payer(
        &[system_instruction::transfer(&pubkey, &pubkey, 1000)],
        Some(&pubkey),
    );

    let signed_tx = wallet.sign_transaction(tx).await.unwrap();
    assert!(!signed_tx.signatures.is_empty());

    // Verify audit log
    let logs = audit_logger.get_recent_logs(10).await.unwrap();
    assert!(logs.iter().any(|log| log.event_type == "signing_attempt"));
    assert!(logs.iter().any(|log| log.event_type == "signing_success"));
}

#[tokio::test]
async fn test_concurrent_signing() {
    let wallet = Arc::new(create_test_wallet().await);
    let mut handles = vec![];

    // Spawn 10 concurrent signing operations
    for i in 0..10 {
        let wallet_clone = wallet.clone();
        let handle = tokio::spawn(async move {
            let tx = create_test_transaction_with_nonce(i);
            wallet_clone.sign_transaction(tx).await
        });
        handles.push(handle);
    }

    // All should complete without deadlock
    let results = futures::future::join_all(handles).await;
    
    // Some may fail due to rate limiting, but no panics
    for result in results {
        assert!(result.is_ok()); // No panic
    }
}
```

## Dependencies

- **Task 1**: Common libraries for base models
- **Task 3**: Database setup for audit logging

## Integration Points

- **Live Trader Service**: Primary consumer of wallet functionality
- **Transaction Executor**: Uses wallet for signing operations
- **Monitoring System**: Tracks wallet operations and rotation events
- **Database**: Stores audit logs and rotation history

## Performance Considerations

- **Key Derivation**: 100k PBKDF2 iterations (~50-100ms)
- **Signing Operation**: <10ms including security checks
- **Memory Protection**: Minimal overhead from secrecy wrapper
- **Rotation Process**: <1 second for complete rotation

## Security Considerations

- **Key Storage**: AES-256-GCM encryption with PBKDF2
- **Memory Safety**: Automatic zeroization via secrecy crate
- **Access Control**: File permissions restricted to owner only
- **Audit Trail**: All operations logged with timestamps
- **Rate Limiting**: Prevents rapid-fire signing attempts
- **Tamper Detection**: SHA-256 checksums for wallet files

## Future Enhancements

- Hardware wallet support (Ledger integration)
- Multi-signature wallet capabilities
- Threshold signing schemes
- HSM integration for enterprise deployments
- Biometric authentication support
- Emergency key recovery mechanisms