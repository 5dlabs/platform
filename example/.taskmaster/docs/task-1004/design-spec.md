# Technical Design Specification: Data Backup and Recovery System

## 1. System Architecture Overview

### 1.1 High-Level Architecture

The data backup and recovery system follows a layered architecture with separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Admin UI      │    │   Backup API    │    │   Recovery API  │
│   (Management)  │◄──►│   (Scheduling   │◄──►│   (Restoration) │
│                 │    │    & Status)    │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                       ┌─────────────────┐             │
                       │  Backup Service │◄────────────┘
                       │   (Orchestration)│
                       └─────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Database      │    │   File System   │    │   Application   │
│   Backup        │    │   Backup        │    │   Backup        │
│   Strategies    │    │   Strategies    │    │   Strategies    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                │
                       ┌─────────────────┐
                       │   Encryption    │
                       │   Service       │
                       └─────────────────┘
                                │
                       ┌─────────────────┐
                       │   Compression   │
                       │   Service       │
                       └─────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   S3 Storage    │    │  Azure Storage  │    │  Local Storage  │
│   Backend       │    │   Backend       │    │   Backend       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                       ┌─────────────────┐
                       │   Monitoring    │
                       │   & Alerting    │
                       └─────────────────┘
```

### 1.2 Component Responsibilities

- **Backup Service**: Orchestrates backup operations, manages schedules, and coordinates components
- **Database Strategies**: Handle database-specific backup and recovery operations
- **File System Strategies**: Manage file system backups and file-level recovery
- **Encryption Service**: Provides data encryption/decryption using AES-256
- **Compression Service**: Handles data compression and decompression
- **Storage Backends**: Abstract storage operations across different providers
- **Monitoring Service**: Tracks backup health, performance, and generates alerts

## 2. Database Design

### 2.1 Core Tables

```sql
-- Backup configurations and schedules
CREATE TABLE backup_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    backup_type backup_type_enum NOT NULL DEFAULT 'full',
    target_type target_type_enum NOT NULL, -- 'database', 'filesystem', 'application'
    target_config JSONB NOT NULL,
    schedule_cron VARCHAR(255) NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',
    enabled BOOLEAN DEFAULT true,
    retention_policy JSONB NOT NULL,
    storage_config JSONB NOT NULL,
    encryption_enabled BOOLEAN DEFAULT true,
    compression_enabled BOOLEAN DEFAULT true,
    verification_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    CONSTRAINT backup_schedules_cron_valid CHECK (schedule_cron ~ '^[0-9*,-/]+ [0-9*,-/]+ [0-9*,-/]+ [0-9*,-/]+ [0-9*,-/]+$')
);

-- Individual backup executions
CREATE TABLE backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID REFERENCES backup_schedules(id) ON DELETE CASCADE,
    backup_type backup_type_enum NOT NULL,
    target_type target_type_enum NOT NULL,
    target_identifier VARCHAR(255) NOT NULL,
    parent_backup_id UUID REFERENCES backups(id),
    status backup_status_enum DEFAULT 'pending',
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    
    -- Size information
    original_size_bytes BIGINT,
    compressed_size_bytes BIGINT,
    final_size_bytes BIGINT,
    
    -- Security
    encryption_enabled BOOLEAN DEFAULT true,
    encryption_key_id VARCHAR(255),
    compression_algorithm VARCHAR(50),
    
    -- Storage
    storage_backend VARCHAR(50) NOT NULL,
    storage_location TEXT,
    storage_metadata JSONB,
    
    -- Integrity
    checksum_algorithm VARCHAR(50) DEFAULT 'sha256',
    checksum_value VARCHAR(255),
    
    -- Timing
    scheduled_at TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    retention_until TIMESTAMP,
    
    -- Additional data
    metadata JSONB,
    error_message TEXT,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_backups_schedule_id (schedule_id),
    INDEX idx_backups_status (status),
    INDEX idx_backups_target (target_type, target_identifier),
    INDEX idx_backups_created_at (created_at),
    INDEX idx_backups_retention_until (retention_until),
    INDEX idx_backups_parent (parent_backup_id)
);

-- Recovery operations
CREATE TABLE recovery_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_id UUID NOT NULL REFERENCES backups(id),
    recovery_type recovery_type_enum DEFAULT 'full',
    target_config JSONB NOT NULL,
    
    -- Point-in-time recovery
    point_in_time TIMESTAMP,
    
    -- Options
    validate_only BOOLEAN DEFAULT false,
    overwrite_existing BOOLEAN DEFAULT false,
    
    -- Status
    status recovery_status_enum DEFAULT 'pending',
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    
    -- Execution details
    steps JSONB,
    current_step VARCHAR(255),
    
    -- Timing
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    estimated_completion TIMESTAMP,
    
    -- Results
    recovered_size_bytes BIGINT,
    error_message TEXT,
    validation_results JSONB,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    -- Indexes
    INDEX idx_recovery_jobs_backup_id (backup_id),
    INDEX idx_recovery_jobs_status (status),
    INDEX idx_recovery_jobs_created_at (created_at)
);

-- Backup verification results
CREATE TABLE backup_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_id UUID NOT NULL REFERENCES backups(id),
    verification_type verification_type_enum NOT NULL,
    status verification_status_enum DEFAULT 'pending',
    
    -- Results
    checksum_valid BOOLEAN,
    restore_test_passed BOOLEAN,
    data_integrity_verified BOOLEAN,
    
    -- Details
    verification_details JSONB,
    error_message TEXT,
    
    -- Timing
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_backup_verifications_backup_id (backup_id),
    INDEX idx_backup_verifications_status (status),
    INDEX idx_backup_verifications_type (verification_type)
);

-- Audit and activity logging
CREATE TABLE backup_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_type VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_backup_audit_log_timestamp (timestamp),
    INDEX idx_backup_audit_log_user_id (user_id),
    INDEX idx_backup_audit_log_operation (operation_type, action)
);
```

### 2.2 Enums and Types

```sql
-- Backup types
CREATE TYPE backup_type_enum AS ENUM ('full', 'incremental', 'differential');

-- Target types
CREATE TYPE target_type_enum AS ENUM ('database', 'filesystem', 'application');

-- Backup status
CREATE TYPE backup_status_enum AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');

-- Recovery types
CREATE TYPE recovery_type_enum AS ENUM ('full', 'partial', 'point_in_time');

-- Recovery status
CREATE TYPE recovery_status_enum AS ENUM ('pending', 'validating', 'running', 'completed', 'failed', 'cancelled');

-- Verification types
CREATE TYPE verification_type_enum AS ENUM ('checksum', 'restore_test', 'integrity_check', 'full_verification');

-- Verification status
CREATE TYPE verification_status_enum AS ENUM ('pending', 'running', 'passed', 'failed');
```

## 3. Core Service Implementation

### 3.1 Backup Service Architecture

```javascript
class BackupService {
  constructor(config) {
    this.config = config;
    this.scheduler = new BackupScheduler(this);
    this.strategyFactory = new BackupStrategyFactory(config);
    this.encryptionService = new EncryptionService(config.encryption);
    this.compressionService = new CompressionService(config.compression);
    this.storageManager = new StorageManager(config.storage);
    this.verificationService = new VerificationService(this);
    this.auditLogger = new AuditLogger(config.audit);
    this.metricsCollector = new MetricsCollector();
  }

  async executeBackup(scheduleId, triggerType = 'scheduled') {
    const schedule = await this.getBackupSchedule(scheduleId);
    const backupJob = await this.createBackupJob(schedule, triggerType);
    
    try {
      await this.updateBackupStatus(backupJob.id, 'running');
      await this.auditLogger.log('backup_started', 'backup', backupJob.id, { schedule: schedule.name });
      
      // Execute backup based on target type
      const strategy = this.strategyFactory.getStrategy(schedule.target_type);
      const backupData = await strategy.executeBackup(schedule, backupJob);
      
      // Process backup data (compress, encrypt)
      const processedData = await this.processBackupData(backupData, schedule);
      
      // Store backup
      const storageResult = await this.storeBackup(processedData, backupJob, schedule);
      
      // Update backup job with results
      await this.updateBackupJob(backupJob.id, {
        status: 'completed',
        completed_at: new Date(),
        storage_location: storageResult.location,
        storage_metadata: storageResult.metadata,
        checksum_value: processedData.checksum,
        original_size_bytes: backupData.size,
        compressed_size_bytes: processedData.compressedSize,
        final_size_bytes: processedData.finalSize
      });
      
      // Schedule verification if enabled
      if (schedule.verification_enabled) {
        await this.scheduleVerification(backupJob.id);
      }
      
      // Apply retention policy
      await this.applyRetentionPolicy(schedule);
      
      await this.auditLogger.log('backup_completed', 'backup', backupJob.id);
      return backupJob;
      
    } catch (error) {
      await this.updateBackupStatus(backupJob.id, 'failed', error.message);
      await this.auditLogger.log('backup_failed', 'backup', backupJob.id, { error: error.message });
      throw error;
    }
  }

  async processBackupData(backupData, schedule) {
    let processedData = backupData;
    
    // Compress if enabled
    if (schedule.compression_enabled) {
      const compressionAlgorithm = schedule.storage_config.compression_algorithm || 'gzip';
      processedData = await this.compressionService.compress(processedData, compressionAlgorithm);
    }
    
    // Encrypt if enabled
    if (schedule.encryption_enabled) {
      processedData = await this.encryptionService.encrypt(processedData);
    }
    
    // Calculate checksum
    processedData.checksum = await this.calculateChecksum(processedData.data);
    
    return processedData;
  }

  async storeBackup(processedData, backupJob, schedule) {
    const storageBackend = this.storageManager.getBackend(schedule.storage_config.backend);
    const fileName = this.generateBackupFileName(backupJob, schedule);
    
    const storageResult = await storageBackend.store(fileName, processedData.data, {
      metadata: {
        backup_id: backupJob.id,
        backup_type: backupJob.backup_type,
        target_identifier: backupJob.target_identifier,
        created_at: backupJob.created_at,
        encryption_enabled: schedule.encryption_enabled,
        compression_enabled: schedule.compression_enabled
      }
    });
    
    return storageResult;
  }

  generateBackupFileName(backupJob, schedule) {
    const date = new Date(backupJob.created_at);
    const dateStr = date.toISOString().split('T')[0];
    const timeStr = date.toISOString().split('T')[1].split('.')[0].replace(/:/g, '-');
    
    return `${schedule.name}_${backupJob.backup_type}_${dateStr}_${timeStr}_${backupJob.id}.backup`;
  }
}
```

### 3.2 Database Backup Strategies

```javascript
class PostgreSQLBackupStrategy extends DatabaseBackupStrategy {
  constructor(config) {
    super(config);
    this.pgDumpPath = config.pgDumpPath || 'pg_dump';
    this.pgRestorePath = config.pgRestorePath || 'pg_restore';
  }

  async executeBackup(schedule, backupJob) {
    const targetConfig = schedule.target_config;
    const backupType = backupJob.backup_type;
    
    switch (backupType) {
      case 'full':
        return this.executeFullBackup(targetConfig, backupJob);
      case 'incremental':
        return this.executeIncrementalBackup(targetConfig, backupJob);
      case 'differential':
        return this.executeDifferentialBackup(targetConfig, backupJob);
      default:
        throw new Error(`Unsupported backup type: ${backupType}`);
    }
  }

  async executeFullBackup(targetConfig, backupJob) {
    const dumpOptions = this.buildPgDumpOptions(targetConfig, 'full');
    
    return new Promise((resolve, reject) => {
      const child = spawn(this.pgDumpPath, dumpOptions);
      const chunks = [];
      let totalSize = 0;
      
      child.stdout.on('data', (chunk) => {
        chunks.push(chunk);
        totalSize += chunk.length;
        
        // Update progress based on estimated total size
        const progress = Math.min(90, Math.round((totalSize / this.estimateBackupSize(targetConfig)) * 100));
        this.updateBackupProgress(backupJob.id, progress);
      });
      
      child.stderr.on('data', (data) => {
        console.error(`pg_dump stderr: ${data}`);
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve({
            data: Buffer.concat(chunks),
            size: totalSize,
            metadata: {
              database: targetConfig.database,
              dump_version: this.getPgDumpVersion(),
              backup_method: 'pg_dump',
              options: dumpOptions
            }
          });
        } else {
          reject(new Error(`pg_dump failed with exit code ${code}`));
        }
      });
      
      child.on('error', (error) => {
        reject(new Error(`Failed to spawn pg_dump: ${error.message}`));
      });
    });
  }

  async executeIncrementalBackup(targetConfig, backupJob) {
    // Find the last backup to determine what has changed
    const lastBackup = await this.findLastBackup(targetConfig.database, backupJob.id);
    
    if (!lastBackup) {
      // No previous backup found, perform full backup
      return this.executeFullBackup(targetConfig, backupJob);
    }
    
    // For PostgreSQL incremental backups, we use WAL files and base backup
    const walFiles = await this.getWALFilesSince(lastBackup.completed_at);
    
    return {
      data: await this.packWALFiles(walFiles),
      size: walFiles.reduce((total, file) => total + file.size, 0),
      metadata: {
        database: targetConfig.database,
        backup_method: 'wal_files',
        parent_backup_id: lastBackup.id,
        wal_files: walFiles.map(f => f.name)
      }
    };
  }

  buildPgDumpOptions(targetConfig, backupType) {
    const options = [
      '--host', targetConfig.host,
      '--port', targetConfig.port.toString(),
      '--username', targetConfig.username,
      '--no-password',
      '--format', 'custom',
      '--compress', '9',
      '--verbose'
    ];
    
    // Add SSL options if configured
    if (targetConfig.ssl) {
      options.push('--ssl');
      if (targetConfig.ssl_cert) {
        options.push('--ssl-cert', targetConfig.ssl_cert);
      }
    }
    
    // Add specific tables or schemas if configured
    if (targetConfig.tables) {
      targetConfig.tables.forEach(table => {
        options.push('--table', table);
      });
    }
    
    if (targetConfig.schemas) {
      targetConfig.schemas.forEach(schema => {
        options.push('--schema', schema);
      });
    }
    
    options.push(targetConfig.database);
    
    return options;
  }

  async restore(backupData, targetConfig, recoveryJob) {
    const restoreOptions = this.buildPgRestoreOptions(targetConfig, recoveryJob);
    
    return new Promise((resolve, reject) => {
      const child = spawn(this.pgRestorePath, restoreOptions);
      
      child.stdin.write(backupData.data);
      child.stdin.end();
      
      child.stdout.on('data', (data) => {
        console.log(`pg_restore stdout: ${data}`);
      });
      
      child.stderr.on('data', (data) => {
        console.error(`pg_restore stderr: ${data}`);
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve({
            success: true,
            restored_objects: this.parseRestoreOutput(),
            restored_size: backupData.size
          });
        } else {
          reject(new Error(`pg_restore failed with exit code ${code}`));
        }
      });
      
      child.on('error', (error) => {
        reject(new Error(`Failed to spawn pg_restore: ${error.message}`));
      });
    });
  }
}

class MySQLBackupStrategy extends DatabaseBackupStrategy {
  constructor(config) {
    super(config);
    this.mysqldumpPath = config.mysqldumpPath || 'mysqldump';
    this.mysqlPath = config.mysqlPath || 'mysql';
  }

  async executeFullBackup(targetConfig, backupJob) {
    const dumpOptions = this.buildMysqldumpOptions(targetConfig);
    
    return new Promise((resolve, reject) => {
      const child = spawn(this.mysqldumpPath, dumpOptions);
      const chunks = [];
      let totalSize = 0;
      
      child.stdout.on('data', (chunk) => {
        chunks.push(chunk);
        totalSize += chunk.length;
        
        const progress = Math.min(90, Math.round((totalSize / this.estimateBackupSize(targetConfig)) * 100));
        this.updateBackupProgress(backupJob.id, progress);
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve({
            data: Buffer.concat(chunks),
            size: totalSize,
            metadata: {
              database: targetConfig.database,
              backup_method: 'mysqldump',
              mysql_version: this.getMySQLVersion(targetConfig)
            }
          });
        } else {
          reject(new Error(`mysqldump failed with exit code ${code}`));
        }
      });
    });
  }

  buildMysqldumpOptions(targetConfig) {
    const options = [
      '--host', targetConfig.host,
      '--port', targetConfig.port.toString(),
      '--user', targetConfig.username,
      `--password=${targetConfig.password}`,
      '--single-transaction',
      '--routines',
      '--triggers',
      '--events',
      '--hex-blob',
      '--complete-insert',
      '--extended-insert'
    ];
    
    if (targetConfig.tables) {
      options.push('--tables');
      options.push(...targetConfig.tables);
    }
    
    options.push(targetConfig.database);
    
    return options;
  }
}

class MongoDBBackupStrategy extends DatabaseBackupStrategy {
  constructor(config) {
    super(config);
    this.mongodumpPath = config.mongodumpPath || 'mongodump';
    this.mongorestorePath = config.mongorestorePath || 'mongorestore';
  }

  async executeFullBackup(targetConfig, backupJob) {
    const dumpOptions = this.buildMongodumpOptions(targetConfig);
    const outputDir = `/tmp/mongodb_backup_${backupJob.id}`;
    
    try {
      // Create temporary directory
      await fs.mkdir(outputDir, { recursive: true });
      
      // Execute mongodump
      await this.executeMongodump([...dumpOptions, '--out', outputDir]);
      
      // Archive the backup directory
      const archiveData = await this.archiveDirectory(outputDir);
      
      return {
        data: archiveData,
        size: archiveData.length,
        metadata: {
          database: targetConfig.database,
          backup_method: 'mongodump',
          collections: await this.getCollectionNames(targetConfig)
        }
      };
    } finally {
      // Cleanup temporary directory
      await fs.rmdir(outputDir, { recursive: true });
    }
  }

  buildMongodumpOptions(targetConfig) {
    const options = [
      '--host', `${targetConfig.host}:${targetConfig.port}`,
      '--db', targetConfig.database
    ];
    
    if (targetConfig.username && targetConfig.password) {
      options.push('--username', targetConfig.username);
      options.push('--password', targetConfig.password);
      options.push('--authenticationDatabase', targetConfig.authDatabase || 'admin');
    }
    
    if (targetConfig.ssl) {
      options.push('--ssl');
    }
    
    if (targetConfig.collections) {
      targetConfig.collections.forEach(collection => {
        options.push('--collection', collection);
      });
    }
    
    return options;
  }
}
```

### 3.3 Encryption Service Implementation

```javascript
class EncryptionService {
  constructor(config) {
    this.algorithm = config.algorithm || 'aes-256-gcm';
    this.keyManager = new KeyManager(config.keyManagement);
  }

  async encrypt(data) {
    const key = await this.keyManager.getCurrentKey();
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipher(this.algorithm, key.value, iv);
    
    let encrypted = cipher.update(data);
    encrypted = Buffer.concat([encrypted, cipher.final()]);
    
    const tag = cipher.getAuthTag();
    
    const encryptedPackage = {
      algorithm: this.algorithm,
      keyId: key.id,
      iv: iv.toString('base64'),
      tag: tag.toString('base64'),
      data: encrypted.toString('base64')
    };
    
    return {
      data: Buffer.from(JSON.stringify(encryptedPackage)),
      encrypted: true,
      keyId: key.id,
      algorithm: this.algorithm
    };
  }

  async decrypt(encryptedData) {
    const encryptedPackage = JSON.parse(encryptedData.toString());
    
    const key = await this.keyManager.getKey(encryptedPackage.keyId);
    const iv = Buffer.from(encryptedPackage.iv, 'base64');
    const tag = Buffer.from(encryptedPackage.tag, 'base64');
    const data = Buffer.from(encryptedPackage.data, 'base64');
    
    const decipher = crypto.createDecipher(encryptedPackage.algorithm, key.value, iv);
    decipher.setAuthTag(tag);
    
    let decrypted = decipher.update(data);
    decrypted = Buffer.concat([decrypted, decipher.final()]);
    
    return decrypted;
  }
}

class KeyManager {
  constructor(config) {
    this.config = config;
    this.keyStorage = this.initializeKeyStorage(config);
    this.keyRotationInterval = config.rotationInterval || 30 * 24 * 60 * 60 * 1000; // 30 days
  }

  async getCurrentKey() {
    let key = await this.keyStorage.getCurrentKey();
    
    if (!key || this.shouldRotateKey(key)) {
      key = await this.rotateKey();
    }
    
    return key;
  }

  async rotateKey() {
    const newKey = {
      id: crypto.randomUUID(),
      value: crypto.randomBytes(32), // 256-bit key
      created_at: new Date(),
      status: 'active'
    };
    
    // Mark current key as deprecated
    const currentKey = await this.keyStorage.getCurrentKey();
    if (currentKey) {
      await this.keyStorage.updateKeyStatus(currentKey.id, 'deprecated');
    }
    
    // Store new key
    await this.keyStorage.storeKey(newKey);
    
    return newKey;
  }

  shouldRotateKey(key) {
    const keyAge = Date.now() - new Date(key.created_at).getTime();
    return keyAge > this.keyRotationInterval;
  }
}
```

### 3.4 Storage Backend Implementation

```javascript
class S3StorageBackend {
  constructor(config) {
    this.s3Client = new AWS.S3({
      accessKeyId: config.accessKeyId,
      secretAccessKey: config.secretAccessKey,
      region: config.region,
      signatureVersion: 'v4'
    });
    this.bucket = config.bucket;
    this.keyPrefix = config.keyPrefix || 'backups/';
  }

  async store(fileName, data, options = {}) {
    const key = `${this.keyPrefix}${this.generateKeyPath(fileName)}`;
    
    const uploadParams = {
      Bucket: this.bucket,
      Key: key,
      Body: data,
      ServerSideEncryption: 'AES256',
      StorageClass: options.storageClass || 'STANDARD_IA',
      Metadata: options.metadata || {}
    };
    
    // Add lifecycle rules
    if (options.retentionDays) {
      uploadParams.Tagging = `retention=${options.retentionDays}`;
    }
    
    const result = await this.s3Client.upload(uploadParams).promise();
    
    return {
      location: `s3://${this.bucket}/${key}`,
      metadata: {
        etag: result.ETag,
        versionId: result.VersionId,
        size: data.length,
        storageClass: uploadParams.StorageClass
      }
    };
  }

  async retrieve(location) {
    const { bucket, key } = this.parseS3Location(location);
    
    const params = {
      Bucket: bucket,
      Key: key
    };
    
    const result = await this.s3Client.getObject(params).promise();
    
    return {
      data: result.Body,
      metadata: {
        lastModified: result.LastModified,
        etag: result.ETag,
        size: result.ContentLength
      }
    };
  }

  async delete(location) {
    const { bucket, key } = this.parseS3Location(location);
    
    const params = {
      Bucket: bucket,
      Key: key
    };
    
    await this.s3Client.deleteObject(params).promise();
  }

  async list(prefix, options = {}) {
    const params = {
      Bucket: this.bucket,
      Prefix: `${this.keyPrefix}${prefix || ''}`,
      MaxKeys: options.limit || 1000
    };
    
    if (options.continuationToken) {
      params.ContinuationToken = options.continuationToken;
    }
    
    const result = await this.s3Client.listObjectsV2(params).promise();
    
    return {
      objects: result.Contents.map(obj => ({
        key: obj.Key,
        lastModified: obj.LastModified,
        size: obj.Size,
        etag: obj.ETag,
        storageClass: obj.StorageClass
      })),
      continuationToken: result.NextContinuationToken,
      isTruncated: result.IsTruncated
    };
  }

  generateKeyPath(fileName) {
    const date = new Date();
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    
    return `${year}/${month}/${day}/${fileName}`;
  }

  parseS3Location(location) {
    const url = new URL(location);
    return {
      bucket: url.hostname.split('.')[0],
      key: url.pathname.substring(1)
    };
  }
}

class AzureBlobStorageBackend {
  constructor(config) {
    this.blobServiceClient = new BlobServiceClient(
      `https://${config.accountName}.blob.core.windows.net`,
      new StorageSharedKeyCredential(config.accountName, config.accountKey)
    );
    this.containerName = config.containerName;
  }

  async store(fileName, data, options = {}) {
    const blobName = this.generateBlobName(fileName);
    const containerClient = this.blobServiceClient.getContainerClient(this.containerName);
    const blockBlobClient = containerClient.getBlockBlobClient(blobName);
    
    const uploadOptions = {
      metadata: options.metadata || {},
      tier: options.accessTier || 'Cool'
    };
    
    const result = await blockBlobClient.upload(data, data.length, uploadOptions);
    
    return {
      location: `https://${this.blobServiceClient.accountName}.blob.core.windows.net/${this.containerName}/${blobName}`,
      metadata: {
        etag: result.etag,
        lastModified: result.lastModified,
        size: data.length
      }
    };
  }

  async retrieve(location) {
    const blobName = this.parseBlobLocation(location);
    const containerClient = this.blobServiceClient.getContainerClient(this.containerName);
    const blockBlobClient = containerClient.getBlockBlobClient(blobName);
    
    const downloadResponse = await blockBlobClient.download();
    const data = await this.streamToBuffer(downloadResponse.readableStreamBody);
    
    return {
      data,
      metadata: {
        lastModified: downloadResponse.lastModified,
        etag: downloadResponse.etag,
        size: downloadResponse.contentLength
      }
    };
  }

  generateBlobName(fileName) {
    const date = new Date();
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    
    return `backups/${year}/${month}/${day}/${fileName}`;
  }
}
```

## 4. Recovery Service Implementation

### 4.1 Recovery Manager

```javascript
class RecoveryManager {
  constructor(config) {
    this.config = config;
    this.backupService = new BackupService(config);
    this.strategyFactory = new BackupStrategyFactory(config);
    this.storageManager = new StorageManager(config.storage);
    this.encryptionService = new EncryptionService(config.encryption);
    this.compressionService = new CompressionService(config.compression);
    this.auditLogger = new AuditLogger(config.audit);
  }

  async initiateRecovery(recoveryRequest) {
    const recoveryJob = await this.createRecoveryJob(recoveryRequest);
    
    try {
      await this.updateRecoveryStatus(recoveryJob.id, 'validating');
      
      // Validate backup and recovery request
      await this.validateRecoveryRequest(recoveryRequest);
      
      // Get backup information
      const backup = await this.getBackup(recoveryRequest.backup_id);
      const backupChain = await this.buildBackupChain(backup);
      
      await this.updateRecoveryStatus(recoveryJob.id, 'running');
      
      // Execute recovery based on type
      const result = await this.executeRecovery(recoveryJob, backupChain, recoveryRequest);
      
      await this.updateRecoveryJob(recoveryJob.id, {
        status: 'completed',
        completed_at: new Date(),
        recovered_size_bytes: result.recoveredSize,
        validation_results: result.validationResults
      });
      
      await this.auditLogger.log('recovery_completed', 'recovery_job', recoveryJob.id);
      return recoveryJob;
      
    } catch (error) {
      await this.updateRecoveryStatus(recoveryJob.id, 'failed', error.message);
      await this.auditLogger.log('recovery_failed', 'recovery_job', recoveryJob.id, { error: error.message });
      throw error;
    }
  }

  async executeRecovery(recoveryJob, backupChain, recoveryRequest) {
    const steps = this.planRecoverySteps(backupChain, recoveryRequest);
    
    await this.updateRecoverySteps(recoveryJob.id, steps);
    
    let recoveredSize = 0;
    const validationResults = {};
    
    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      await this.updateCurrentStep(recoveryJob.id, step.name);
      
      const stepResult = await this.executeRecoveryStep(step, recoveryRequest);
      recoveredSize += stepResult.size || 0;
      
      if (stepResult.validationResult) {
        validationResults[step.name] = stepResult.validationResult;
      }
      
      const progress = Math.round(((i + 1) / steps.length) * 100);
      await this.updateRecoveryProgress(recoveryJob.id, progress);
    }
    
    return { recoveredSize, validationResults };
  }

  planRecoverySteps(backupChain, recoveryRequest) {
    const steps = [];
    
    // Step 1: Prepare target environment
    steps.push({
      name: 'prepare_target',
      description: 'Prepare target environment for recovery',
      backup: null,
      action: 'prepare'
    });
    
    // Step 2: Restore base backup (full backup)
    const baseBackup = backupChain.find(b => b.backup_type === 'full');
    if (!baseBackup) {
      throw new Error('No full backup found in chain');
    }
    
    steps.push({
      name: 'restore_base',
      description: 'Restore base full backup',
      backup: baseBackup,
      action: 'restore'
    });
    
    // Step 3: Apply incremental backups in order
    const incrementalBackups = backupChain
      .filter(b => b.backup_type === 'incremental')
      .sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
    
    incrementalBackups.forEach((backup, index) => {
      steps.push({
        name: `apply_incremental_${index + 1}`,
        description: `Apply incremental backup ${index + 1}`,
        backup: backup,
        action: 'apply_incremental'
      });
    });
    
    // Step 4: Apply point-in-time recovery if requested
    if (recoveryRequest.point_in_time) {
      steps.push({
        name: 'apply_point_in_time',
        description: 'Apply point-in-time recovery',
        backup: null,
        action: 'point_in_time',
        target_time: recoveryRequest.point_in_time
      });
    }
    
    // Step 5: Validate recovery
    if (!recoveryRequest.validate_only) {
      steps.push({
        name: 'validate_recovery',
        description: 'Validate recovered data',
        backup: null,
        action: 'validate'
      });
    }
    
    return steps;
  }

  async executeRecoveryStep(step, recoveryRequest) {
    switch (step.action) {
      case 'prepare':
        return this.prepareTarget(recoveryRequest);
      case 'restore':
        return this.restoreBackup(step.backup, recoveryRequest);
      case 'apply_incremental':
        return this.applyIncrementalBackup(step.backup, recoveryRequest);
      case 'point_in_time':
        return this.applyPointInTimeRecovery(step.target_time, recoveryRequest);
      case 'validate':
        return this.validateRecovery(recoveryRequest);
      default:
        throw new Error(`Unknown recovery step action: ${step.action}`);
    }
  }

  async restoreBackup(backup, recoveryRequest) {
    // Retrieve backup data
    const backupData = await this.retrieveBackupData(backup);
    
    // Decrypt if needed
    let processedData = backupData;
    if (backup.encryption_enabled) {
      processedData = await this.encryptionService.decrypt(processedData);
    }
    
    // Decompress if needed
    if (backup.compressed_size_bytes !== backup.original_size_bytes) {
      processedData = await this.compressionService.decompress(processedData);
    }
    
    // Get appropriate strategy and restore
    const strategy = this.strategyFactory.getStrategy(backup.target_type);
    const result = await strategy.restore(processedData, recoveryRequest.target_config, {
      validate_only: recoveryRequest.validate_only,
      overwrite_existing: recoveryRequest.overwrite_existing
    });
    
    return {
      size: backup.original_size_bytes,
      validationResult: result.validationResult
    };
  }

  async retrieveBackupData(backup) {
    const storageBackend = this.storageManager.getBackend(backup.storage_backend);
    const result = await storageBackend.retrieve(backup.storage_location);
    
    // Verify checksum
    const calculatedChecksum = await this.calculateChecksum(result.data);
    if (calculatedChecksum !== backup.checksum_value) {
      throw new Error(`Backup checksum mismatch. Expected: ${backup.checksum_value}, Got: ${calculatedChecksum}`);
    }
    
    return result.data;
  }

  async pointInTimeRecovery(recoveryRequest) {
    const { backup_id, point_in_time, target_config } = recoveryRequest;
    
    // Find the backup chain up to the point in time
    const backup = await this.getBackup(backup_id);
    const backupChain = await this.buildBackupChainForPointInTime(backup, point_in_time);
    
    // Restore the chain
    const recoveryJob = await this.createRecoveryJob({
      ...recoveryRequest,
      recovery_type: 'point_in_time'
    });
    
    return this.executeRecovery(recoveryJob, backupChain, recoveryRequest);
  }

  async buildBackupChainForPointInTime(targetBackup, pointInTime) {
    const chain = [];
    const targetTime = new Date(pointInTime);
    
    // Find the most recent full backup before the point in time
    const fullBackup = await this.findMostRecentFullBackup(targetBackup.target_identifier, targetTime);
    if (!fullBackup) {
      throw new Error('No full backup found before the specified point in time');
    }
    
    chain.push(fullBackup);
    
    // Find all incremental backups between the full backup and point in time
    const incrementalBackups = await this.findIncrementalBackups(
      targetBackup.target_identifier,
      fullBackup.created_at,
      pointInTime
    );
    
    chain.push(...incrementalBackups);
    
    return chain;
  }
}
```

## 5. Verification and Testing

### 5.1 Backup Verification Service

```javascript
class VerificationService {
  constructor(backupService) {
    this.backupService = backupService;
    this.storageManager = backupService.storageManager;
    this.encryptionService = backupService.encryptionService;
    this.compressionService = backupService.compressionService;
  }

  async verifyBackup(backupId, verificationType = 'full_verification') {
    const verification = await this.createVerificationJob(backupId, verificationType);
    
    try {
      await this.updateVerificationStatus(verification.id, 'running');
      
      const backup = await this.getBackup(backupId);
      const verificationResults = {};
      
      // Checksum verification
      if (['checksum', 'full_verification'].includes(verificationType)) {
        verificationResults.checksum = await this.verifyChecksum(backup);
      }
      
      // Restore test
      if (['restore_test', 'full_verification'].includes(verificationType)) {
        verificationResults.restoreTest = await this.performRestoreTest(backup);
      }
      
      // Data integrity check
      if (['integrity_check', 'full_verification'].includes(verificationType)) {
        verificationResults.integrityCheck = await this.performIntegrityCheck(backup);
      }
      
      const overallResult = Object.values(verificationResults).every(result => result.passed);
      
      await this.updateVerificationResult(verification.id, {
        status: overallResult ? 'passed' : 'failed',
        completed_at: new Date(),
        checksum_valid: verificationResults.checksum?.passed,
        restore_test_passed: verificationResults.restoreTest?.passed,
        data_integrity_verified: verificationResults.integrityCheck?.passed,
        verification_details: verificationResults
      });
      
      return verification;
      
    } catch (error) {
      await this.updateVerificationStatus(verification.id, 'failed', error.message);
      throw error;
    }
  }

  async verifyChecksum(backup) {
    try {
      const backupData = await this.retrieveBackupData(backup);
      const calculatedChecksum = await this.calculateChecksum(backupData);
      
      const passed = calculatedChecksum === backup.checksum_value;
      
      return {
        passed,
        expected: backup.checksum_value,
        calculated: calculatedChecksum,
        algorithm: backup.checksum_algorithm
      };
    } catch (error) {
      return {
        passed: false,
        error: error.message
      };
    }
  }

  async performRestoreTest(backup) {
    try {
      // Create a test database/target for restoration
      const testTarget = await this.createTestTarget(backup);
      
      // Perform restoration to test target
      const recoveryRequest = {
        backup_id: backup.id,
        target_config: testTarget,
        validate_only: false,
        overwrite_existing: true
      };
      
      const result = await this.backupService.recoveryManager.restoreBackup(backup, recoveryRequest);
      
      // Verify the restored data
      const verificationResult = await this.verifyRestoredData(backup, testTarget);
      
      // Cleanup test target
      await this.cleanupTestTarget(testTarget);
      
      return {
        passed: verificationResult.passed,
        restoredSize: result.size,
        verificationDetails: verificationResult.details
      };
    } catch (error) {
      return {
        passed: false,
        error: error.message
      };
    }
  }

  async performIntegrityCheck(backup) {
    try {
      // Retrieve and process backup data
      let backupData = await this.retrieveBackupData(backup);
      
      // Decrypt if needed
      if (backup.encryption_enabled) {
        backupData = await this.encryptionService.decrypt(backupData);
      }
      
      // Decompress if needed
      if (backup.compressed_size_bytes !== backup.original_size_bytes) {
        backupData = await this.compressionService.decompress(backupData);
      }
      
      // Perform format-specific integrity checks
      const strategy = this.backupService.strategyFactory.getStrategy(backup.target_type);
      const integrityResult = await strategy.checkIntegrity(backupData, backup.metadata);
      
      return {
        passed: integrityResult.valid,
        details: integrityResult.details
      };
    } catch (error) {
      return {
        passed: false,
        error: error.message
      };
    }
  }

  async createTestTarget(backup) {
    // Create a temporary test database/target based on backup type
    const testConfig = {
      ...backup.metadata.originalConfig,
      database: `test_restore_${backup.id}`,
      temporary: true
    };
    
    const strategy = this.backupService.strategyFactory.getStrategy(backup.target_type);
    await strategy.createTestTarget(testConfig);
    
    return testConfig;
  }
}
```

This comprehensive technical design specification provides the foundation for implementing a robust, secure, and scalable data backup and recovery system with enterprise-grade features including encryption, multiple storage backends, and comprehensive verification capabilities.