# AI Agent Prompt: Implement Data Backup and Recovery System

## Task Context

You are tasked with implementing a comprehensive data backup and recovery system for critical business data. This system must provide automated backups, secure storage, reliable recovery, and disaster recovery capabilities with encryption and verification throughout the process.

## Primary Objective

Build a robust backup and recovery system that includes:
- Automated backup scheduling with full and incremental backups
- Multi-database support (PostgreSQL, MySQL, MongoDB)
- File system backup capabilities
- Encryption at rest and in transit using AES-256
- Multiple storage backends (S3, Azure, GCS, local storage)
- Point-in-time recovery capabilities
- Backup verification and integrity checking
- Disaster recovery automation and testing

## Technical Requirements

### Core Components to Implement

1. **Backup Service**
   - Automated backup scheduler with configurable timing
   - Support for full, incremental, and differential backups
   - Database-specific backup strategies
   - File system backup capabilities
   - Backup compression and optimization
   - Backup metadata tracking and management

2. **Encryption and Security**
   - AES-256 encryption for all backup data
   - Secure key management with rotation
   - Encrypted transmission protocols
   - Access control and audit logging
   - Secure deletion and data sanitization

3. **Storage Management**
   - Multiple storage backend support
   - Intelligent storage tiering
   - Retention policy enforcement
   - Storage usage monitoring and optimization
   - Cross-region replication capabilities

4. **Recovery Service**
   - Point-in-time recovery functionality
   - Backup validation and integrity checking
   - Recovery testing and verification
   - Disaster recovery automation
   - Recovery progress tracking and reporting

### Performance Requirements

- Backup operations should not impact production performance
- Incremental backups should complete within 30 minutes
- Full backups should complete within 4 hours for databases up to 1TB
- Recovery operations should complete within 2 hours
- Backup verification should complete within 1 hour

### Security Requirements

- All backup data must be encrypted using AES-256
- Encryption keys must be managed securely with rotation
- All access must be logged and audited
- Network transmission must use secure protocols
- Compliance with data protection regulations (GDPR, HIPAA)

## Implementation Approach

### Phase 1: Core Backup Infrastructure
1. Set up backup service architecture and database schema
2. Implement automated backup scheduler
3. Create database-specific backup strategies
4. Add compression and optimization capabilities
5. Set up backup metadata tracking

### Phase 2: Security and Encryption
1. Implement AES-256 encryption service
2. Create secure key management system
3. Add access control and audit logging
4. Set up secure transmission protocols
5. Implement data sanitization procedures

### Phase 3: Storage and Recovery
1. Implement multiple storage backend support
2. Create recovery service with validation
3. Add point-in-time recovery capabilities
4. Implement backup verification system
5. Create disaster recovery automation

### Phase 4: Monitoring and Management
1. Build backup monitoring and alerting
2. Create management dashboard and APIs
3. Implement performance optimization
4. Add compliance and audit reporting
5. Create operational procedures and documentation

## Code Structure Expectations

```
src/
├── backup/
│   ├── services/
│   │   ├── backup-manager.js
│   │   ├── backup-scheduler.js
│   │   └── backup-validator.js
│   ├── strategies/
│   │   ├── postgresql-strategy.js
│   │   ├── mysql-strategy.js
│   │   ├── mongodb-strategy.js
│   │   └── file-system-strategy.js
│   ├── storage/
│   │   ├── s3-backend.js
│   │   ├── azure-backend.js
│   │   ├── gcs-backend.js
│   │   └── local-backend.js
│   └── compression/
│       ├── gzip-compressor.js
│       └── lz4-compressor.js
├── recovery/
│   ├── services/
│   │   ├── recovery-manager.js
│   │   └── recovery-validator.js
│   ├── strategies/
│   │   └── point-in-time-recovery.js
│   └── testing/
│       └── recovery-tester.js
├── security/
│   ├── encryption-service.js
│   ├── key-manager.js
│   └── audit-logger.js
├── monitoring/
│   ├── backup-monitor.js
│   ├── alert-manager.js
│   └── metrics-collector.js
└── tests/
    ├── unit/
    ├── integration/
    └── disaster-recovery/
```

## Database Schema Requirements

### Backups Table
```sql
CREATE TABLE backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_type VARCHAR(20) NOT NULL CHECK (backup_type IN ('full', 'incremental', 'differential')),
    database_name VARCHAR(255) NOT NULL,
    parent_backup_id UUID REFERENCES backups(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    size_bytes BIGINT,
    compressed_size_bytes BIGINT,
    encryption_enabled BOOLEAN DEFAULT true,
    checksum VARCHAR(255),
    storage_location TEXT NOT NULL,
    storage_backend VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    retention_until TIMESTAMP,
    metadata JSONB
);
```

### Recovery Jobs Table
```sql
CREATE TABLE recovery_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_id UUID NOT NULL REFERENCES backups(id),
    target_database VARCHAR(255) NOT NULL,
    point_in_time TIMESTAMP,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    progress INTEGER DEFAULT 0,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    steps JSONB,
    error_message TEXT
);
```

## Backup Implementation Details

### PostgreSQL Backup Strategy
```javascript
class PostgreSQLBackupStrategy {
  constructor(config) {
    this.config = config;
  }

  async backup(dbConfig, type, sinceTimestamp) {
    const pgDumpCommand = this.buildPgDumpCommand(dbConfig, type, sinceTimestamp);
    
    return new Promise((resolve, reject) => {
      const child = spawn('pg_dump', pgDumpCommand);
      const chunks = [];
      
      child.stdout.on('data', (chunk) => {
        chunks.push(chunk);
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve(Buffer.concat(chunks));
        } else {
          reject(new Error(`pg_dump failed with code ${code}`));
        }
      });
    });
  }

  buildPgDumpCommand(dbConfig, type, sinceTimestamp) {
    const cmd = [
      '--host', dbConfig.host,
      '--port', dbConfig.port,
      '--username', dbConfig.username,
      '--no-password',
      '--format', 'custom',
      '--compress', '9'
    ];
    
    if (type === 'incremental' && sinceTimestamp) {
      cmd.push('--where', `updated_at > '${sinceTimestamp}'`);
    }
    
    cmd.push(dbConfig.database);
    return cmd;
  }
}
```

### Encryption Service Implementation
```javascript
class EncryptionService {
  constructor(config) {
    this.algorithm = 'aes-256-gcm';
    this.keyManager = new KeyManager(config.keyManagement);
  }

  async encrypt(data) {
    const key = await this.keyManager.getCurrentKey();
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipher(this.algorithm, key, iv);
    
    let encrypted = cipher.update(data);
    encrypted = Buffer.concat([encrypted, cipher.final()]);
    
    const tag = cipher.getAuthTag();
    
    return {
      data: encrypted,
      iv: iv.toString('hex'),
      tag: tag.toString('hex'),
      algorithm: this.algorithm,
      keyId: key.id
    };
  }

  async decrypt(encryptedData) {
    const key = await this.keyManager.getKey(encryptedData.keyId);
    const iv = Buffer.from(encryptedData.iv, 'hex');
    const tag = Buffer.from(encryptedData.tag, 'hex');
    
    const decipher = crypto.createDecipher(this.algorithm, key, iv);
    decipher.setAuthTag(tag);
    
    let decrypted = decipher.update(encryptedData.data);
    decrypted = Buffer.concat([decrypted, decipher.final()]);
    
    return decrypted;
  }
}
```

## Storage Backend Implementation

### S3 Storage Backend
```javascript
class S3StorageBackend {
  constructor(config) {
    this.s3Client = new AWS.S3({
      accessKeyId: config.accessKeyId,
      secretAccessKey: config.secretAccessKey,
      region: config.region
    });
    this.bucket = config.bucket;
  }

  async store(fileName, data) {
    const key = `backups/${new Date().getFullYear()}/${fileName}`;
    
    const params = {
      Bucket: this.bucket,
      Key: key,
      Body: data,
      ServerSideEncryption: 'AES256',
      StorageClass: 'STANDARD_IA'
    };
    
    await this.s3Client.upload(params).promise();
    return `s3://${this.bucket}/${key}`;
  }

  async retrieve(location) {
    const url = new URL(location);
    const key = url.pathname.substring(1);
    
    const params = {
      Bucket: this.bucket,
      Key: key
    };
    
    const result = await this.s3Client.getObject(params).promise();
    return result.Body;
  }
}
```

## Recovery Implementation

### Recovery Manager
```javascript
class RecoveryManager {
  constructor(config) {
    this.config = config;
    this.backupManager = new BackupManager(config);
    this.encryptionService = new EncryptionService(config);
  }

  async restoreFromBackup(options) {
    const recoveryJob = await this.createRecoveryJob(options);
    
    try {
      await this.updateRecoveryStatus(recoveryJob.id, 'in_progress');
      
      // Validate backup
      await this.validateBackup(options.backupId);
      
      // Retrieve and decrypt backup data
      const backupData = await this.retrieveBackupData(options.backupId);
      
      // Perform restoration
      await this.performRestoration(backupData, options);
      
      await this.updateRecoveryStatus(recoveryJob.id, 'completed');
      
      return recoveryJob;
    } catch (error) {
      await this.updateRecoveryStatus(recoveryJob.id, 'failed', error.message);
      throw error;
    }
  }

  async pointInTimeRecovery(options) {
    const { database, pointInTime } = options;
    
    // Find base backup
    const baseBackup = await this.findBaseBackup(database, pointInTime);
    
    // Find incremental backups
    const incrementalBackups = await this.findIncrementalBackups(
      database, 
      baseBackup.createdAt, 
      pointInTime
    );
    
    // Restore base backup
    await this.restoreFromBackup({
      backupId: baseBackup.id,
      targetDatabase: options.targetDatabase
    });
    
    // Apply incremental backups
    for (const backup of incrementalBackups) {
      await this.applyIncrementalBackup(backup, options.targetDatabase);
    }
  }
}
```

## Monitoring and Alerting

### Backup Monitor
```javascript
class BackupMonitor {
  constructor(config) {
    this.config = config;
    this.alertManager = new AlertManager(config.alerts);
  }

  async monitorBackups() {
    const checks = [
      this.checkScheduledBackups(),
      this.checkBackupHealth(),
      this.checkStorageUsage(),
      this.checkRetentionCompliance()
    ];
    
    const results = await Promise.allSettled(checks);
    
    for (const result of results) {
      if (result.status === 'rejected') {
        await this.alertManager.sendAlert({
          type: 'backup_monitor_error',
          message: result.reason.message,
          severity: 'high'
        });
      }
    }
  }

  async checkScheduledBackups() {
    const overdue = await this.getOverdueBackups();
    
    if (overdue.length > 0) {
      await this.alertManager.sendAlert({
        type: 'backup_overdue',
        message: `${overdue.length} backup(s) are overdue`,
        severity: 'high'
      });
    }
  }
}
```

## Testing Requirements

### Unit Tests (Minimum Coverage: 90%)
- Backup creation and validation
- Encryption and decryption operations
- Storage backend operations
- Recovery procedures and validation
- Compression and decompression
- Key management operations

### Integration Tests
- End-to-end backup and recovery workflows
- Multi-database backup scenarios
- Storage backend integration
- Disaster recovery procedures
- Performance under load
- Security and access control

### Disaster Recovery Tests
- Complete system failure scenarios
- Data corruption recovery
- Storage backend failures
- Network partition scenarios
- Key management failures
- Point-in-time recovery accuracy

## Environment Configuration

Required environment variables:
```env
# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=main_db
DB_USER=backup_user
DB_PASSWORD=secure_password

# Storage Configuration
BACKUP_STORAGE_BACKEND=s3
S3_ACCESS_KEY_ID=your_access_key
S3_SECRET_ACCESS_KEY=your_secret_key
S3_BUCKET=backup-bucket
S3_REGION=us-east-1

# Encryption Configuration
ENCRYPTION_ALGORITHM=aes-256-gcm
KEY_MANAGEMENT_SERVICE=aws-kms
KMS_KEY_ID=your-kms-key-id

# Backup Configuration
BACKUP_SCHEDULE=0 2 * * *
BACKUP_RETENTION_DAYS=90
BACKUP_COMPRESSION_ENABLED=true
BACKUP_ENCRYPTION_ENABLED=true

# Monitoring Configuration
ALERT_WEBHOOK_URL=https://alerts.example.com/webhook
MONITORING_INTERVAL=300000
STORAGE_USAGE_THRESHOLD=80
```

## Quality Assurance Checklist

Before marking this task complete, ensure:

- [ ] Automated backup scheduling works correctly
- [ ] Full and incremental backups are created successfully
- [ ] All backup data is encrypted using AES-256
- [ ] Multiple storage backends are supported and functional
- [ ] Recovery operations complete within time requirements
- [ ] Point-in-time recovery works accurately
- [ ] Backup verification and integrity checking are implemented
- [ ] Retention policies are enforced automatically
- [ ] Monitoring and alerting systems are functional
- [ ] Disaster recovery procedures are documented and tested
- [ ] Security requirements are met and validated
- [ ] Performance requirements are achieved
- [ ] Tests achieve minimum 90% coverage
- [ ] Documentation is complete and accurate
- [ ] Compliance requirements are satisfied

## Success Metrics

- Backup success rate > 99.9%
- Recovery time objective (RTO) < 2 hours
- Recovery point objective (RPO) < 1 hour
- Backup verification success rate > 99%
- Zero data loss incidents
- Test coverage > 90%
- All integration tests passing
- Security audit findings resolved
- Performance requirements met under load

## Important Notes

1. **Security First**: All data must be encrypted and access must be controlled
2. **Reliability**: Backup system must be more reliable than the systems it protects
3. **Performance**: Backup operations must not impact production systems
4. **Testing**: Regular disaster recovery testing is mandatory
5. **Compliance**: Ensure compliance with all relevant data protection regulations
6. **Documentation**: Comprehensive procedures for operations team
7. **Monitoring**: Continuous monitoring and alerting for backup health

Begin implementation with the core backup service and encryption capabilities. Focus on security and reliability throughout the development process.