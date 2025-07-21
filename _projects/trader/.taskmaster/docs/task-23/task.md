# Task 23: Implement Advanced Configuration Management System

## Overview

This task implements a sophisticated configuration management system that provides centralized, versioned, and auditable configuration for the trading platform. The system supports both paper and live trading modes, handles sensitive data securely, and enables hot-reloading of non-critical settings. Configuration is persisted in PostgreSQL with full audit trails and supports hierarchical inheritance with environment-specific overrides.

## Dependencies

This task depends on:
- Task 1: Common Rust Libraries for Trading Models - Provides core configuration structures
- Task 2: PostgreSQL Database Schema and Migration System - Database infrastructure for persistence
- Task 13: Risk Management System with Position Tracking - Defines risk parameter structures
- Task 18: Monitoring and Logging Infrastructure - Integration for configuration change events

## Architecture Context

According to the architecture.md, the configuration management system is crucial for:
- Managing trader-specific settings and risk parameters
- Supporting Token-2022 extension configurations
- Storing MEV protection parameters (priority fees, timeouts)
- Enabling circuit breaker thresholds (200ms P99 latency)
- Facilitating seamless switching between paper and live modes

## Implementation Details

### 1. Core Configuration Manager

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Transaction};
use chrono::{DateTime, Utc};
use notify::{Watcher, RecursiveMode, watcher};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ConfigManager {
    pool: PgPool,
    cache: Arc<RwLock<ConfigCache>>,
    schema_validator: Arc<ConfigSchemaValidator>,
    encryption_key: Vec<u8>,
    notifier: Arc<ConfigChangeNotifier>,
    environment: Environment,
}

#[derive(Default)]
struct ConfigCache {
    items: HashMap<String, CachedConfigItem>,
    version_id: i32,
}

struct CachedConfigItem {
    value: JsonValue,
    cached_at: DateTime<Utc>,
    ttl_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl ConfigManager {
    pub async fn new(
        pool: PgPool,
        encryption_key: Vec<u8>,
        environment: Environment,
    ) -> Result<Self> {
        let schema_validator = Arc::new(ConfigSchemaValidator::new());
        let notifier = Arc::new(ConfigChangeNotifier::new());
        
        let manager = Self {
            pool,
            cache: Arc::new(RwLock::new(ConfigCache::default())),
            schema_validator,
            encryption_key,
            notifier,
            environment,
        };
        
        // Load active configuration
        manager.reload_configuration().await?;
        
        // Start configuration watcher
        manager.start_config_watcher().await?;
        
        Ok(manager)
    }
    
    /// Get configuration value by path with type safety
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        // Check cache first
        if let Some(cached) = self.get_from_cache(path).await {
            return Ok(serde_json::from_value(cached)?);
        }
        
        // Load from database
        let value = self.load_from_database(path).await?;
        
        // Update cache
        self.update_cache(path, value.clone()).await;
        
        Ok(serde_json::from_value(value)?)
    }
    
    /// Set configuration value with validation
    pub async fn set<T: Serialize>(
        &self,
        path: &str,
        value: T,
        changed_by: &str,
        description: Option<&str>,
    ) -> Result<()> {
        let json_value = serde_json::to_value(&value)?;
        
        // Validate against schema
        self.schema_validator.validate(path, &json_value)?;
        
        // Check if this is a sensitive value
        let is_sensitive = self.is_sensitive_path(path);
        let stored_value = if is_sensitive {
            self.encrypt_value(&json_value)?
        } else {
            json_value.clone()
        };
        
        // Start transaction
        let mut tx = self.pool.begin().await?;
        
        // Get or create new version
        let version_id = self.get_or_create_version(&mut tx, description).await?;
        
        // Store old value for audit
        let old_value = self.get_current_value(&mut tx, path).await.ok();
        
        // Update configuration
        self.store_config_item(&mut tx, version_id, path, &stored_value, is_sensitive).await?;
        
        // Create audit entry
        self.create_audit_entry(
            &mut tx,
            version_id,
            path,
            old_value,
            Some(json_value.clone()),
            changed_by,
        ).await?;
        
        // Commit transaction
        tx.commit().await?;
        
        // Clear cache
        self.invalidate_cache(path).await;
        
        // Notify listeners
        self.notifier.notify_change(path, &json_value).await;
        
        Ok(())
    }
    
    async fn get_from_cache(&self, path: &str) -> Option<JsonValue> {
        let cache = self.cache.read().await;
        
        if let Some(item) = cache.items.get(path) {
            let age = Utc::now() - item.cached_at;
            if age.num_seconds() < item.ttl_seconds as i64 {
                return Some(item.value.clone());
            }
        }
        
        None
    }
    
    async fn update_cache(&self, path: &str, value: JsonValue) {
        let mut cache = self.cache.write().await;
        
        let ttl_seconds = match path {
            p if p.starts_with("risk.") => 300,  // 5 minutes for risk params
            p if p.starts_with("token.") => 3600, // 1 hour for token config
            _ => 60, // 1 minute default
        };
        
        cache.items.insert(path.to_string(), CachedConfigItem {
            value,
            cached_at: Utc::now(),
            ttl_seconds,
        });
    }
    
    fn is_sensitive_path(&self, path: &str) -> bool {
        path.contains("api_key") ||
        path.contains("private_key") ||
        path.contains("secret") ||
        path.contains("password")
    }
    
    fn encrypt_value(&self, value: &JsonValue) -> Result<JsonValue> {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
        
        let json_str = serde_json::to_string(value)?;
        let plaintext = json_str.as_bytes();
        
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.encryption_key)?;
        let key = LessSafeKey::new(unbound_key);
        
        let nonce = Nonce::assume_unique_for_key([0u8; 12]);
        let mut ciphertext = plaintext.to_vec();
        
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)?;
        
        Ok(json!({
            "encrypted": true,
            "data": base64::encode(&ciphertext),
            "algorithm": "AES-256-GCM"
        }))
    }
}
```

### 2. Database Schema Implementation

```rust
// migrations/postgres/config_management.sql
pub const CONFIG_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS configuration_versions (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT FALSE,
    description TEXT,
    environment VARCHAR(20) NOT NULL
);

CREATE INDEX idx_config_versions_active ON configuration_versions(is_active);
CREATE INDEX idx_config_versions_env ON configuration_versions(environment);

CREATE TABLE IF NOT EXISTS configuration_items (
    id SERIAL PRIMARY KEY,
    version_id INTEGER REFERENCES configuration_versions(id) ON DELETE CASCADE,
    path VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    sensitive BOOLEAN DEFAULT FALSE,
    inherited BOOLEAN DEFAULT FALSE,
    parent_path VARCHAR(255),
    UNIQUE(version_id, path)
);

CREATE INDEX idx_config_items_path ON configuration_items(path);
CREATE INDEX idx_config_items_version ON configuration_items(version_id);

CREATE TABLE IF NOT EXISTS configuration_audit (
    id SERIAL PRIMARY KEY,
    version_id INTEGER REFERENCES configuration_versions(id),
    path VARCHAR(255) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    changed_by VARCHAR(100) NOT NULL,
    change_type VARCHAR(20) NOT NULL -- 'create', 'update', 'delete'
);

CREATE INDEX idx_config_audit_path ON configuration_audit(path);
CREATE INDEX idx_config_audit_time ON configuration_audit(changed_at);

-- Function to ensure only one active version per environment
CREATE OR REPLACE FUNCTION ensure_single_active_version()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_active = TRUE THEN
        UPDATE configuration_versions 
        SET is_active = FALSE 
        WHERE environment = NEW.environment 
        AND id != NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_single_active
AFTER INSERT OR UPDATE ON configuration_versions
FOR EACH ROW
EXECUTE FUNCTION ensure_single_active_version();
"#;
```

### 3. Configuration Schema Validator

```rust
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

pub struct ConfigSchemaValidator {
    schemas: HashMap<String, JSONSchema>,
}

impl ConfigSchemaValidator {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();
        
        // Risk parameters schema
        let risk_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "maxPositionSize": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Maximum position size in base currency"
                },
                "maxPositions": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100
                },
                "dailyLossLimit": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Maximum daily loss allowed"
                },
                "maxExposurePerToken": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "description": "Maximum exposure per token as percentage"
                }
            },
            "required": ["maxPositionSize", "dailyLossLimit"],
            "additionalProperties": false
        });
        
        schemas.insert(
            "risk".to_string(),
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&risk_schema)
                .unwrap()
        );
        
        // MEV protection schema
        let mev_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "minPriorityFee": {
                    "type": "integer",
                    "minimum": 1000,
                    "maximum": 100000
                },
                "maxPriorityFee": {
                    "type": "integer",
                    "minimum": 1000,
                    "maximum": 1000000
                },
                "dynamicFeeEnabled": {
                    "type": "boolean"
                },
                "sandwichProtectionEnabled": {
                    "type": "boolean"
                },
                "timeoutMs": {
                    "type": "integer",
                    "minimum": 100,
                    "maximum": 5000
                }
            },
            "required": ["minPriorityFee", "maxPriorityFee"],
            "additionalProperties": false
        });
        
        schemas.insert(
            "mev".to_string(),
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&mev_schema)
                .unwrap()
        );
        
        // Token configuration schema
        let token_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "pattern": "^[1-9A-HJ-NP-Za-km-z]{32,44}$"
                },
                "symbol": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 10
                },
                "decimals": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 18
                },
                "customSlippageBps": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 1000
                },
                "transferFeeEnabled": {
                    "type": "boolean"
                },
                "transferFeeBps": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 10000
                },
                "extensions": {
                    "type": "object",
                    "description": "Token-2022 extension data"
                }
            },
            "required": ["address", "symbol", "decimals"],
            "additionalProperties": false
        });
        
        schemas.insert(
            "token".to_string(),
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&token_schema)
                .unwrap()
        );
        
        // Circuit breaker schema
        let circuit_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "latencyThresholdMs": {
                    "type": "integer",
                    "minimum": 50,
                    "maximum": 1000,
                    "default": 200
                },
                "errorRateThreshold": {
                    "type": "number",
                    "minimum": 0.01,
                    "maximum": 1.0,
                    "default": 0.05
                },
                "failureThreshold": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 5
                },
                "recoveryTimeoutSecs": {
                    "type": "integer",
                    "minimum": 10,
                    "maximum": 600,
                    "default": 60
                },
                "halfOpenMaxRequests": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 3
                }
            },
            "required": ["latencyThresholdMs", "errorRateThreshold"],
            "additionalProperties": false
        });
        
        schemas.insert(
            "circuit_breaker".to_string(),
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&circuit_schema)
                .unwrap()
        );
        
        Self { schemas }
    }
    
    pub fn validate(&self, path: &str, value: &JsonValue) -> Result<()> {
        let category = path.split('.').next()
            .ok_or_else(|| anyhow!("Invalid configuration path"))?;
        
        let schema = self.schemas.get(category)
            .ok_or_else(|| anyhow!("No schema found for category: {}", category))?;
        
        let result = schema.validate(value);
        
        if let Err(errors) = result {
            let error_messages: Vec<String> = errors
                .map(|e| format!("{}: {}", e.instance_path, e.to_string()))
                .collect();
            
            return Err(anyhow!(
                "Configuration validation failed:\n{}",
                error_messages.join("\n")
            ));
        }
        
        Ok(())
    }
}
```

### 4. Configuration Access API

```rust
// Strongly-typed configuration access
impl ConfigManager {
    pub async fn get_risk_parameters(&self) -> Result<RiskParameters> {
        self.get("risk.parameters").await
    }
    
    pub async fn get_token_config(&self, token_address: &str) -> Result<TokenConfig> {
        let path = format!("token.{}", token_address);
        self.get(&path).await
    }
    
    pub async fn get_mev_protection(&self) -> Result<MevProtectionConfig> {
        self.get("mev.protection").await
    }
    
    pub async fn get_circuit_breaker_config(&self) -> Result<CircuitBreakerConfig> {
        self.get("circuit_breaker.config").await
    }
    
    pub async fn get_monitoring_config(&self) -> Result<MonitoringConfig> {
        self.get("monitoring.config").await
    }
    
    pub async fn get_trading_mode(&self) -> Result<TradingMode> {
        self.get("trader.mode").await
    }
    
    pub async fn update_risk_limit(
        &self,
        limit_type: &str,
        value: f64,
        changed_by: &str,
    ) -> Result<()> {
        let path = format!("risk.limits.{}", limit_type);
        self.set(&path, value, changed_by, Some("Risk limit update")).await
    }
    
    pub async fn add_token_config(
        &self,
        token: TokenConfig,
        changed_by: &str,
    ) -> Result<()> {
        let path = format!("token.{}", token.address);
        self.set(&path, token, changed_by, Some("Add new token configuration")).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParameters {
    pub max_position_size: f64,
    pub max_positions: u32,
    pub daily_loss_limit: f64,
    pub max_exposure_per_token: f64,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    pub custom_slippage_bps: Option<u16>,
    pub transfer_fee_enabled: bool,
    pub transfer_fee_bps: Option<u16>,
    pub extensions: Option<JsonValue>,
    pub min_order_size: Option<f64>,
    pub max_order_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtectionConfig {
    pub min_priority_fee: u64,
    pub max_priority_fee: u64,
    pub dynamic_fee_enabled: bool,
    pub sandwich_protection_enabled: bool,
    pub timeout_ms: u64,
    pub wrap_sol_enabled: bool,
    pub use_shared_accounts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub latency_threshold_ms: u64,
    pub error_rate_threshold: f64,
    pub failure_threshold: u32,
    pub recovery_timeout_secs: u64,
    pub half_open_max_requests: u32,
}
```

### 5. Hot Reload Implementation

```rust
use notify::{DebouncedEvent, RecommendedWatcher};
use std::sync::mpsc::channel;
use std::time::Duration;

impl ConfigManager {
    async fn start_config_watcher(&self) -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
        
        // Watch configuration change triggers (could be file, database trigger, etc.)
        watcher.watch("./config/reload_trigger", RecursiveMode::NonRecursive)?;
        
        let manager = self.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(DebouncedEvent::Write(_)) | Ok(DebouncedEvent::Create(_)) => {
                        info!("Configuration change detected, reloading...");
                        
                        if let Err(e) = manager.reload_configuration().await {
                            error!("Failed to reload configuration: {}", e);
                        } else {
                            info!("Configuration reloaded successfully");
                        }
                    }
                    Err(e) => {
                        error!("Configuration watcher error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    async fn reload_configuration(&self) -> Result<()> {
        // Get active version for current environment
        let version_id = sqlx::query_scalar!(
            r#"
            SELECT id FROM configuration_versions
            WHERE is_active = true AND environment = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            self.environment.to_string()
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Load all configuration items for this version
        let items = sqlx::query!(
            r#"
            SELECT path, value, sensitive
            FROM configuration_items
            WHERE version_id = $1
            "#,
            version_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Clear and rebuild cache
        let mut cache = self.cache.write().await;
        cache.items.clear();
        cache.version_id = version_id;
        
        for item in items {
            let value = if item.sensitive {
                self.decrypt_value(&item.value)?
            } else {
                item.value
            };
            
            cache.items.insert(item.path.clone(), CachedConfigItem {
                value,
                cached_at: Utc::now(),
                ttl_seconds: 60, // Default TTL
            });
        }
        
        // Notify all listeners about reload
        self.notifier.notify_reload().await;
        
        Ok(())
    }
}
```

### 6. Configuration Change Notifier

```rust
use tokio::sync::broadcast;

pub struct ConfigChangeNotifier {
    sender: broadcast::Sender<ConfigChangeEvent>,
}

#[derive(Clone, Debug)]
pub enum ConfigChangeEvent {
    ValueChanged { path: String, new_value: JsonValue },
    ConfigReloaded,
}

impl ConfigChangeNotifier {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.sender.subscribe()
    }
    
    pub async fn notify_change(&self, path: &str, new_value: &JsonValue) {
        let _ = self.sender.send(ConfigChangeEvent::ValueChanged {
            path: path.to_string(),
            new_value: new_value.clone(),
        });
    }
    
    pub async fn notify_reload(&self) {
        let _ = self.sender.send(ConfigChangeEvent::ConfigReloaded);
    }
}

// Example usage in a component
pub struct RiskManagerWithConfigWatch {
    risk_params: Arc<RwLock<RiskParameters>>,
    config_manager: Arc<ConfigManager>,
}

impl RiskManagerWithConfigWatch {
    pub async fn new(config_manager: Arc<ConfigManager>) -> Result<Self> {
        let risk_params = config_manager.get_risk_parameters().await?;
        
        let manager = Self {
            risk_params: Arc::new(RwLock::new(risk_params)),
            config_manager,
        };
        
        // Start listening for configuration changes
        manager.start_config_listener();
        
        Ok(manager)
    }
    
    fn start_config_listener(&self) {
        let mut receiver = self.config_manager.notifier.subscribe();
        let risk_params = self.risk_params.clone();
        let config_manager = self.config_manager.clone();
        
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    ConfigChangeEvent::ValueChanged { path, .. } if path.starts_with("risk.") => {
                        // Reload risk parameters
                        if let Ok(new_params) = config_manager.get_risk_parameters().await {
                            *risk_params.write().await = new_params;
                            info!("Risk parameters updated from configuration change");
                        }
                    }
                    ConfigChangeEvent::ConfigReloaded => {
                        // Full reload
                        if let Ok(new_params) = config_manager.get_risk_parameters().await {
                            *risk_params.write().await = new_params;
                            info!("Risk parameters reloaded");
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
```

### 7. Configuration CLI Tools

```rust
use clap::{App, Arg, SubCommand};

pub fn build_config_cli() -> App<'static, 'static> {
    App::new("config-manager")
        .about("Configuration management CLI")
        .subcommand(
            SubCommand::with_name("get")
                .about("Get configuration value")
                .arg(Arg::with_name("path")
                    .required(true)
                    .help("Configuration path (e.g., risk.maxPositionSize)"))
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set configuration value")
                .arg(Arg::with_name("path")
                    .required(true)
                    .help("Configuration path"))
                .arg(Arg::with_name("value")
                    .required(true)
                    .help("New value (JSON format)"))
                .arg(Arg::with_name("user")
                    .long("user")
                    .takes_value(true)
                    .default_value("admin")
                    .help("User making the change"))
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List all configuration values")
                .arg(Arg::with_name("prefix")
                    .long("prefix")
                    .takes_value(true)
                    .help("Filter by path prefix"))
        )
        .subcommand(
            SubCommand::with_name("history")
                .about("Show configuration change history")
                .arg(Arg::with_name("path")
                    .required(true)
                    .help("Configuration path"))
                .arg(Arg::with_name("limit")
                    .long("limit")
                    .takes_value(true)
                    .default_value("10")
                    .help("Number of changes to show"))
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("Export configuration")
                .arg(Arg::with_name("output")
                    .long("output")
                    .takes_value(true)
                    .default_value("config_export.json")
                    .help("Output file"))
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Import configuration")
                .arg(Arg::with_name("input")
                    .required(true)
                    .help("Input file"))
                .arg(Arg::with_name("user")
                    .long("user")
                    .takes_value(true)
                    .default_value("admin")
                    .help("User making the import"))
        )
        .subcommand(
            SubCommand::with_name("validate")
                .about("Validate configuration file")
                .arg(Arg::with_name("file")
                    .required(true)
                    .help("Configuration file to validate"))
        )
}

pub async fn handle_config_command(
    matches: &ArgMatches<'_>,
    config_manager: Arc<ConfigManager>,
) -> Result<()> {
    match matches.subcommand() {
        ("get", Some(sub_matches)) => {
            let path = sub_matches.value_of("path").unwrap();
            let value: JsonValue = config_manager.get(path).await?;
            println!("{} = {}", path, serde_json::to_string_pretty(&value)?);
        }
        ("set", Some(sub_matches)) => {
            let path = sub_matches.value_of("path").unwrap();
            let value_str = sub_matches.value_of("value").unwrap();
            let user = sub_matches.value_of("user").unwrap();
            
            let value: JsonValue = serde_json::from_str(value_str)?;
            config_manager.set(path, value, user, None).await?;
            println!("Configuration updated: {}", path);
        }
        ("list", Some(sub_matches)) => {
            let prefix = sub_matches.value_of("prefix");
            let items = config_manager.list_all(prefix).await?;
            
            for (path, value) in items {
                println!("{} = {}", path, serde_json::to_string(&value)?);
            }
        }
        ("history", Some(sub_matches)) => {
            let path = sub_matches.value_of("path").unwrap();
            let limit: i64 = sub_matches.value_of("limit").unwrap().parse()?;
            
            let history = config_manager.get_history(path, limit).await?;
            
            for entry in history {
                println!("{} - {} changed by {}", 
                    entry.changed_at.format("%Y-%m-%d %H:%M:%S"),
                    entry.change_type,
                    entry.changed_by
                );
                if let Some(old) = entry.old_value {
                    println!("  Old: {}", serde_json::to_string(&old)?);
                }
                if let Some(new) = entry.new_value {
                    println!("  New: {}", serde_json::to_string(&new)?);
                }
                println!();
            }
        }
        _ => unreachable!(),
    }
    
    Ok(())
}
```

## Integration Example

```rust
// Example: Using ConfigManager in the trading system
pub struct TradingSystem {
    config_manager: Arc<ConfigManager>,
    trade_executor: Arc<TradeExecutor>,
    risk_manager: Arc<RiskManagerWithConfigWatch>,
}

impl TradingSystem {
    pub async fn new(database_url: &str, encryption_key: Vec<u8>) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        let config_manager = Arc::new(
            ConfigManager::new(pool, encryption_key, Environment::Production).await?
        );
        
        let risk_manager = Arc::new(
            RiskManagerWithConfigWatch::new(config_manager.clone()).await?
        );
        
        let trade_executor = Arc::new(
            TradeExecutor::new(config_manager.clone()).await?
        );
        
        Ok(Self {
            config_manager,
            trade_executor,
            risk_manager,
        })
    }
    
    pub async fn execute_trade(&self, trade_params: TradeParams) -> Result<TradeResult> {
        // Get current configuration
        let mev_config = self.config_manager.get_mev_protection().await?;
        let token_config = self.config_manager
            .get_token_config(&trade_params.token_address)
            .await?;
        
        // Apply configuration to trade
        let adjusted_params = trade_params
            .with_priority_fee(mev_config.calculate_priority_fee())
            .with_slippage(token_config.custom_slippage_bps.unwrap_or(100));
        
        // Execute with current configuration
        self.trade_executor.execute(adjusted_params).await
    }
}
```

## Security Considerations

1. **Encryption**: All sensitive values are encrypted using AES-256-GCM before storage
2. **Access Control**: Configuration changes require user identification
3. **Audit Trail**: Complete history of all changes is maintained
4. **Validation**: Schema validation prevents invalid configurations
5. **Environment Isolation**: Separate configurations per environment

## Performance Optimizations

1. **Caching**: Frequently accessed values cached with appropriate TTLs
2. **Batch Loading**: Configuration loaded in bulk on startup
3. **Selective Invalidation**: Only affected cache entries cleared on updates
4. **Async Operations**: All database operations are non-blocking
5. **Connection Pooling**: Efficient database connection management