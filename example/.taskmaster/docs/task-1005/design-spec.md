# Technical Design Specification: Monitoring and Alerting System

## 1. System Architecture Overview

### 1.1 High-Level Architecture

The monitoring and alerting system follows a distributed, scalable architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Dashboard │    │   Mobile App    │    │   Grafana       │
│   (React/Vue)   │◄──►│   (Native)      │◄──►│   Integration   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │                       │                       │
          └───────────────────────┼───────────────────────┘
                                  │
                       ┌─────────────────┐
                       │   API Gateway   │
                       │  (Rate Limiting)│
                       └─────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Dashboard API  │    │   Metrics API   │    │  Alerting API   │
│   (Express.js)  │    │   (Express.js)  │    │  (Express.js)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                         │                         │
        └─────────────────────────┼─────────────────────────┘
                                  │
                       ┌─────────────────┐
                       │   Message Bus   │
                       │  (Redis/RabbitMQ)│
                       └─────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Metrics Collector│    │  Alert Manager  │    │ Notification    │
│    (Worker)      │    │    (Worker)     │    │    Service      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                         │                         │
        └─────────────────────────┼─────────────────────────┘
                                  │
                       ┌─────────────────┐
                       │   Time Series   │
                       │    Database     │
                       │ (InfluxDB/TimescaleDB)│
                       └─────────────────┘
                                  │
                       ┌─────────────────┐
                       │   Metadata DB   │
                       │  (PostgreSQL)   │
                       └─────────────────┘
```

### 1.2 Component Responsibilities

- **API Gateway**: Request routing, authentication, rate limiting, load balancing
- **Metrics API**: Metrics ingestion, validation, and storage coordination
- **Alerting API**: Alert rule management, alert status, and acknowledgment
- **Dashboard API**: Dashboard configuration, widget management, and data serving
- **Metrics Collector**: System and application metrics collection
- **Alert Manager**: Rule evaluation, alert firing, and correlation
- **Notification Service**: Multi-channel notification delivery
- **Time Series Database**: High-performance metrics storage and querying
- **Metadata Database**: Configuration, rules, dashboards, and user data

## 2. Data Models and Storage

### 2.1 Time Series Schema (InfluxDB/TimescaleDB)

```sql
-- For TimescaleDB (PostgreSQL extension)
CREATE TABLE metrics (
    time TIMESTAMPTZ NOT NULL,
    metric_name TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB,
    
    -- Create hypertable for time-series partitioning
    PRIMARY KEY (time, metric_name, labels)
);

-- Convert to hypertable for automatic partitioning
SELECT create_hypertable('metrics', 'time', chunk_time_interval => INTERVAL '1 hour');

-- Create indexes for efficient querying
CREATE INDEX idx_metrics_name_time ON metrics (metric_name, time DESC);
CREATE INDEX idx_metrics_labels_gin ON metrics USING GIN (labels);
CREATE INDEX idx_metrics_name_labels ON metrics (metric_name, labels);

-- Retention policy for automatic data cleanup
SELECT add_retention_policy('metrics', INTERVAL '90 days');

-- Continuous aggregates for downsampling
CREATE MATERIALIZED VIEW metrics_1h
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', time) AS bucket,
    metric_name,
    labels,
    avg(value) as avg_value,
    min(value) as min_value,
    max(value) as max_value,
    count(*) as sample_count
FROM metrics
GROUP BY bucket, metric_name, labels;

-- Refresh policy for continuous aggregates
SELECT add_continuous_aggregate_policy('metrics_1h',
    start_offset => INTERVAL '2 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '15 minutes');
```

### 2.2 Metadata Schema (PostgreSQL)

```sql
-- Metric definitions and metadata
CREATE TABLE metric_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    type metric_type_enum NOT NULL,
    description TEXT,
    unit VARCHAR(50),
    default_labels JSONB DEFAULT '{}',
    retention_policy JSONB DEFAULT '{"default": "90d", "downsampled": "2y"}',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT metric_name_valid CHECK (name ~ '^[a-zA-Z_:][a-zA-Z0-9_:]*$')
);

-- Alert rules configuration
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query TEXT NOT NULL,
    condition JSONB NOT NULL,
    severity alert_severity_enum NOT NULL DEFAULT 'warning',
    duration INTERVAL DEFAULT '1 minute',
    enabled BOOLEAN DEFAULT true,
    
    -- Grouping and routing
    labels JSONB DEFAULT '{}',
    annotations JSONB DEFAULT '{}',
    
    -- Evaluation settings
    evaluation_interval INTERVAL DEFAULT '30 seconds',
    evaluation_timeout INTERVAL DEFAULT '10 seconds',
    
    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    CONSTRAINT alert_rule_name_unique UNIQUE (name),
    INDEX idx_alert_rules_enabled (enabled),
    INDEX idx_alert_rules_severity (severity)
);

-- Alert instances (firing alerts)
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    fingerprint VARCHAR(64) NOT NULL UNIQUE,
    
    -- Alert state
    status alert_status_enum DEFAULT 'firing',
    value DOUBLE PRECISION,
    labels JSONB NOT NULL DEFAULT '{}',
    annotations JSONB NOT NULL DEFAULT '{}',
    
    -- Timing
    starts_at TIMESTAMP NOT NULL,
    ends_at TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Management
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMP,
    silence_id UUID,
    
    INDEX idx_alerts_rule_id (rule_id),
    INDEX idx_alerts_status (status),
    INDEX idx_alerts_starts_at (starts_at DESC),
    INDEX idx_alerts_fingerprint (fingerprint)
);

-- Alert notification history
CREATE TABLE alert_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    
    -- Notification details
    channel_type VARCHAR(50) NOT NULL,
    channel_config JSONB NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    
    -- Content
    subject TEXT,
    body TEXT,
    
    -- Delivery tracking
    status notification_status_enum DEFAULT 'pending',
    sent_at TIMESTAMP,
    delivered_at TIMESTAMP,
    failed_at TIMESTAMP,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_alert_notifications_alert_id (alert_id),
    INDEX idx_alert_notifications_status (status),
    INDEX idx_alert_notifications_sent_at (sent_at DESC)
);

-- Dashboard configurations
CREATE TABLE dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    
    -- Layout and settings
    layout JSONB NOT NULL DEFAULT '{"grid": {"columns": 12, "rows": "auto"}}',
    settings JSONB DEFAULT '{}',
    
    -- Access control
    visibility dashboard_visibility_enum DEFAULT 'private',
    permissions JSONB DEFAULT '{}',
    
    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    INDEX idx_dashboards_name (name),
    INDEX idx_dashboards_tags (tags),
    INDEX idx_dashboards_visibility (visibility),
    INDEX idx_dashboards_created_by (created_by)
);

-- Dashboard widgets
CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    
    -- Widget configuration
    title VARCHAR(255) NOT NULL,
    type widget_type_enum NOT NULL,
    query TEXT,
    
    -- Visualization settings
    visualization JSONB NOT NULL,
    options JSONB DEFAULT '{}',
    
    -- Layout
    position JSONB NOT NULL, -- {x, y, width, height}
    
    -- Data refresh
    refresh_interval INTERVAL DEFAULT '30 seconds',
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_dashboard_widgets_dashboard_id (dashboard_id)
);

-- Notification channels
CREATE TABLE notification_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    type channel_type_enum NOT NULL,
    configuration JSONB NOT NULL,
    
    -- Routing
    labels JSONB DEFAULT '{}',
    
    -- Settings
    enabled BOOLEAN DEFAULT true,
    rate_limit JSONB DEFAULT '{"per_minute": 10, "per_hour": 100}',
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    CONSTRAINT notification_channel_name_unique UNIQUE (name),
    INDEX idx_notification_channels_type (type),
    INDEX idx_notification_channels_enabled (enabled)
);
```

### 2.3 Enums and Types

```sql
CREATE TYPE metric_type_enum AS ENUM ('counter', 'gauge', 'histogram', 'summary');
CREATE TYPE alert_severity_enum AS ENUM ('info', 'warning', 'critical');
CREATE TYPE alert_status_enum AS ENUM ('firing', 'resolved', 'suppressed');
CREATE TYPE notification_status_enum AS ENUM ('pending', 'sent', 'delivered', 'failed');
CREATE TYPE dashboard_visibility_enum AS ENUM ('private', 'public', 'organization');
CREATE TYPE widget_type_enum AS ENUM ('metric', 'log', 'alert', 'text', 'iframe', 'heatmap');
CREATE TYPE channel_type_enum AS ENUM ('email', 'slack', 'webhook', 'sms', 'pagerduty');
```

## 3. Metrics Collection Architecture

### 3.1 Metrics Collection Service

```javascript
class MetricsCollectionService {
  constructor(config) {
    this.config = config;
    this.storage = new TimeSeriesStorage(config.storage);
    this.registry = new MetricsRegistry();
    this.collectors = this.initializeCollectors(config.collectors);
    this.batchProcessor = new BatchProcessor(config.batching);
    this.retentionManager = new RetentionManager(config.retention);
  }

  async initialize() {
    // Start system metrics collection
    for (const collector of this.collectors) {
      await collector.start();
    }
    
    // Start batch processing
    await this.batchProcessor.start();
    
    // Start retention management
    await this.retentionManager.start();
    
    console.log('Metrics collection service started');
  }

  // Collect single metric sample
  async collectMetric(name, value, labels = {}, timestamp = new Date()) {
    const sample = {
      name,
      value: parseFloat(value),
      labels: this.normalizeLabels(labels),
      timestamp: new Date(timestamp)
    };
    
    // Validate metric
    await this.validateMetric(sample);
    
    // Add to batch for processing
    await this.batchProcessor.addSample(sample);
    
    return sample;
  }

  // Collect multiple metrics in batch
  async collectMetrics(samples) {
    const validatedSamples = [];
    
    for (const sample of samples) {
      try {
        await this.validateMetric(sample);
        validatedSamples.push(sample);
      } catch (error) {
        console.warn(`Invalid metric sample: ${error.message}`, sample);
      }
    }
    
    await this.batchProcessor.addSamples(validatedSamples);
    
    return {
      accepted: validatedSamples.length,
      rejected: samples.length - validatedSamples.length
    };
  }

  async validateMetric(sample) {
    // Validate metric name
    if (!sample.name || typeof sample.name !== 'string') {
      throw new Error('Metric name is required and must be a string');
    }
    
    if (!/^[a-zA-Z_:][a-zA-Z0-9_:]*$/.test(sample.name)) {
      throw new Error('Invalid metric name format');
    }
    
    // Validate value
    if (typeof sample.value !== 'number' || !isFinite(sample.value)) {
      throw new Error('Metric value must be a finite number');
    }
    
    // Validate labels
    if (sample.labels && typeof sample.labels !== 'object') {
      throw new Error('Labels must be an object');
    }
    
    // Check label cardinality
    const labelCount = Object.keys(sample.labels || {}).length;
    if (labelCount > this.config.maxLabelCardinality) {
      throw new Error(`Too many labels: ${labelCount} > ${this.config.maxLabelCardinality}`);
    }
    
    return true;
  }

  normalizeLabels(labels) {
    const normalized = {};
    
    for (const [key, value] of Object.entries(labels || {})) {
      // Normalize label key
      const normalizedKey = key.replace(/[^a-zA-Z0-9_]/g, '_');
      
      // Convert value to string
      normalized[normalizedKey] = String(value);
    }
    
    return normalized;
  }
}

class BatchProcessor {
  constructor(config) {
    this.config = config;
    this.batchSize = config.batchSize || 1000;
    this.flushInterval = config.flushInterval || 10000; // 10 seconds
    this.currentBatch = [];
    this.storage = new TimeSeriesStorage(config.storage);
  }

  async start() {
    // Start periodic flush
    this.flushTimer = setInterval(async () => {
      await this.flush();
    }, this.flushInterval);
    
    console.log('Batch processor started');
  }

  async addSample(sample) {
    this.currentBatch.push(sample);
    
    if (this.currentBatch.length >= this.batchSize) {
      await this.flush();
    }
  }

  async addSamples(samples) {
    this.currentBatch.push(...samples);
    
    if (this.currentBatch.length >= this.batchSize) {
      await this.flush();
    }
  }

  async flush() {
    if (this.currentBatch.length === 0) {
      return;
    }
    
    const batch = this.currentBatch.splice(0);
    
    try {
      await this.storage.writeBatch(batch);
      console.log(`Flushed batch of ${batch.length} metrics`);
    } catch (error) {
      console.error('Failed to write metrics batch:', error);
      
      // Re-add failed samples to retry
      this.currentBatch.unshift(...batch);
    }
  }

  async stop() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    
    // Flush remaining samples
    await this.flush();
    
    console.log('Batch processor stopped');
  }
}
```

### 3.2 System Metrics Collector

```javascript
class SystemMetricsCollector {
  constructor(config) {
    this.config = config;
    this.collectInterval = config.collectInterval || 30000; // 30 seconds
    this.metricsService = config.metricsService;
  }

  async start() {
    this.timer = setInterval(async () => {
      try {
        await this.collectSystemMetrics();
      } catch (error) {
        console.error('System metrics collection failed:', error);
      }
    }, this.collectInterval);
    
    console.log('System metrics collector started');
  }

  async collectSystemMetrics() {
    const metrics = await this.gatherSystemMetrics();
    const timestamp = new Date();
    
    // CPU metrics
    await this.metricsService.collectMetric(
      'system_cpu_usage_percent',
      metrics.cpu.usage,
      { core: 'total' },
      timestamp
    );
    
    for (const [core, usage] of Object.entries(metrics.cpu.cores)) {
      await this.metricsService.collectMetric(
        'system_cpu_usage_percent',
        usage,
        { core },
        timestamp
      );
    }
    
    // Memory metrics
    await this.metricsService.collectMetric(
      'system_memory_total_bytes',
      metrics.memory.total,
      {},
      timestamp
    );
    
    await this.metricsService.collectMetric(
      'system_memory_used_bytes',
      metrics.memory.used,
      {},
      timestamp
    );
    
    await this.metricsService.collectMetric(
      'system_memory_usage_percent',
      (metrics.memory.used / metrics.memory.total) * 100,
      {},
      timestamp
    );
    
    // Disk metrics
    for (const [device, stats] of Object.entries(metrics.disk)) {
      await this.metricsService.collectMetric(
        'system_disk_total_bytes',
        stats.total,
        { device },
        timestamp
      );
      
      await this.metricsService.collectMetric(
        'system_disk_used_bytes',
        stats.used,
        { device },
        timestamp
      );
      
      await this.metricsService.collectMetric(
        'system_disk_usage_percent',
        (stats.used / stats.total) * 100,
        { device },
        timestamp
      );
    }
    
    // Network metrics
    for (const [interface, stats] of Object.entries(metrics.network)) {
      await this.metricsService.collectMetric(
        'system_network_bytes_received_total',
        stats.rx_bytes,
        { interface },
        timestamp
      );
      
      await this.metricsService.collectMetric(
        'system_network_bytes_transmitted_total',
        stats.tx_bytes,
        { interface },
        timestamp
      );
      
      await this.metricsService.collectMetric(
        'system_network_packets_received_total',
        stats.rx_packets,
        { interface },
        timestamp
      );
      
      await this.metricsService.collectMetric(
        'system_network_packets_transmitted_total',
        stats.tx_packets,
        { interface },
        timestamp
      );
    }
  }

  async gatherSystemMetrics() {
    const os = require('os');
    const fs = require('fs').promises;
    
    // CPU metrics
    const cpus = os.cpus();
    const cpu = {
      usage: await this.getCPUUsage(),
      cores: {}
    };
    
    for (let i = 0; i < cpus.length; i++) {
      cpu.cores[`cpu${i}`] = await this.getCoreUsage(i);
    }
    
    // Memory metrics
    const memory = {
      total: os.totalmem(),
      used: os.totalmem() - os.freemem()
    };
    
    // Disk metrics
    const disk = await this.getDiskMetrics();
    
    // Network metrics
    const network = await this.getNetworkMetrics();
    
    return { cpu, memory, disk, network };
  }

  async getCPUUsage() {
    return new Promise((resolve) => {
      const start = process.cpuUsage();
      const startTime = process.hrtime.bigint();
      
      setTimeout(() => {
        const end = process.cpuUsage(start);
        const endTime = process.hrtime.bigint();
        
        const elapsedTime = Number(endTime - startTime) / 1000000; // Convert to milliseconds
        const cpuTime = (end.user + end.system) / 1000; // Convert to milliseconds
        
        const usage = (cpuTime / elapsedTime) * 100;
        resolve(Math.min(100, Math.max(0, usage)));
      }, 1000);
    });
  }

  async getDiskMetrics() {
    const disk = {};
    
    try {
      // Read disk usage from /proc/mounts and statvfs
      const mounts = await fs.readFile('/proc/mounts', 'utf8');
      const lines = mounts.split('\n').filter(line => line.includes('/dev/'));
      
      for (const line of lines) {
        const parts = line.split(' ');
        const device = parts[0].split('/').pop();
        const mountPoint = parts[1];
        
        try {
          const stats = await fs.statvfs(mountPoint);
          disk[device] = {
            total: stats.f_blocks * stats.f_frsize,
            used: (stats.f_blocks - stats.f_bavail) * stats.f_frsize
          };
        } catch (error) {
          // Skip if can't read disk stats
        }
      }
    } catch (error) {
      console.warn('Failed to read disk metrics:', error.message);
    }
    
    return disk;
  }

  async getNetworkMetrics() {
    const network = {};
    
    try {
      const stats = await fs.readFile('/proc/net/dev', 'utf8');
      const lines = stats.split('\n').slice(2); // Skip header lines
      
      for (const line of lines) {
        if (!line.trim()) continue;
        
        const parts = line.trim().split(/\s+/);
        const interface = parts[0].replace(':', '');
        
        network[interface] = {
          rx_bytes: parseInt(parts[1]) || 0,
          rx_packets: parseInt(parts[2]) || 0,
          tx_bytes: parseInt(parts[9]) || 0,
          tx_packets: parseInt(parts[10]) || 0
        };
      }
    } catch (error) {
      console.warn('Failed to read network metrics:', error.message);
    }
    
    return network;
  }
}
```

## 4. Alerting Engine Implementation

### 4.1 Alert Manager Core

```javascript
class AlertManager {
  constructor(config) {
    this.config = config;
    this.ruleEngine = new AlertRuleEngine(config);
    this.evaluator = new AlertEvaluator(config);
    this.correlator = new AlertCorrelator(config);
    this.notificationManager = new NotificationManager(config);
    this.storage = new AlertStorage(config.storage);
    
    this.evaluationInterval = config.evaluationInterval || 30000; // 30 seconds
    this.activeAlerts = new Map();
    this.evaluationTimer = null;
  }

  async start() {
    console.log('Starting Alert Manager...');
    
    // Load alert rules
    await this.ruleEngine.loadRules();
    
    // Load active alerts from storage
    await this.loadActiveAlerts();
    
    // Start rule evaluation loop
    this.startEvaluationLoop();
    
    // Start notification manager
    await this.notificationManager.start();
    
    console.log('Alert Manager started');
  }

  startEvaluationLoop() {
    this.evaluationTimer = setInterval(async () => {
      try {
        await this.evaluateAllRules();
      } catch (error) {
        console.error('Alert evaluation failed:', error);
      }
    }, this.evaluationInterval);
    
    console.log(`Alert evaluation loop started (interval: ${this.evaluationInterval}ms)`);
  }

  async evaluateAllRules() {
    const rules = await this.ruleEngine.getEnabledRules();
    const evaluationPromises = rules.map(rule => this.evaluateRule(rule));
    
    const results = await Promise.allSettled(evaluationPromises);
    
    // Log any evaluation failures
    results.forEach((result, index) => {
      if (result.status === 'rejected') {
        console.error(`Rule evaluation failed for ${rules[index].name}:`, result.reason);
      }
    });
  }

  async evaluateRule(rule) {
    try {
      const queryResult = await this.evaluator.executeQuery(rule.query);
      
      for (const series of queryResult) {
        await this.evaluateSeries(rule, series);
      }
    } catch (error) {
      console.error(`Failed to evaluate rule ${rule.name}:`, error);
    }
  }

  async evaluateSeries(rule, series) {
    const fingerprint = this.generateFingerprint(rule, series.labels);
    const shouldAlert = this.evaluator.shouldAlert(series.value, rule.condition);
    
    const existingAlert = this.activeAlerts.get(fingerprint);
    
    if (shouldAlert) {
      if (!existingAlert) {
        // New alert firing
        await this.handleNewAlert(rule, series, fingerprint);
      } else {
        // Update existing alert
        await this.updateAlert(existingAlert, series);
      }
    } else {
      if (existingAlert && existingAlert.status === 'firing') {
        // Alert resolved
        await this.resolveAlert(existingAlert);
      }
    }
  }

  async handleNewAlert(rule, series, fingerprint) {
    const alert = {
      id: crypto.randomUUID(),
      ruleId: rule.id,
      fingerprint,
      status: 'firing',
      value: series.value,
      labels: { ...rule.labels, ...series.labels },
      annotations: this.renderAnnotations(rule.annotations, series),
      startsAt: new Date(),
      endsAt: null,
      updatedAt: new Date()
    };
    
    // Store alert
    await this.storage.saveAlert(alert);
    this.activeAlerts.set(fingerprint, alert);
    
    console.log(`New alert firing: ${rule.name} (${fingerprint})`);
    
    // Check for correlation
    const correlatedAlerts = await this.correlator.findCorrelatedAlerts(alert);
    
    if (correlatedAlerts.length === 0) {
      // Send notifications for new alert
      await this.notificationManager.sendAlertNotifications(alert, 'firing');
    } else {
      console.log(`Alert correlated with ${correlatedAlerts.length} existing alerts`);
    }
  }

  async updateAlert(alert, series) {
    alert.value = series.value;
    alert.updatedAt = new Date();
    
    await this.storage.updateAlert(alert);
  }

  async resolveAlert(alert) {
    alert.status = 'resolved';
    alert.endsAt = new Date();
    alert.updatedAt = new Date();
    
    await this.storage.updateAlert(alert);
    this.activeAlerts.delete(alert.fingerprint);
    
    console.log(`Alert resolved: ${alert.labels.alertname || 'unknown'} (${alert.fingerprint})`);
    
    // Send resolution notification
    await this.notificationManager.sendAlertNotifications(alert, 'resolved');
  }

  generateFingerprint(rule, labels) {
    const crypto = require('crypto');
    const sortedLabels = Object.keys(labels)
      .sort()
      .map(key => `${key}="${labels[key]}"`)
      .join(',');
    
    const content = `${rule.name}:${sortedLabels}`;
    return crypto.createHash('sha256').update(content).digest('hex').substring(0, 16);
  }

  renderAnnotations(templates, series) {
    const annotations = {};
    
    for (const [key, template] of Object.entries(templates || {})) {
      annotations[key] = this.renderTemplate(template, {
        value: series.value,
        labels: series.labels
      });
    }
    
    return annotations;
  }

  renderTemplate(template, context) {
    return template.replace(/\{\{\s*([^}]+)\s*\}\}/g, (match, expression) => {
      try {
        // Handle $value expressions
        if (expression.includes('$value')) {
          const formatter = expression.split('|')[1]?.trim();
          return this.formatValue(context.value, formatter);
        }
        
        // Handle $labels expressions
        if (expression.startsWith('$labels.')) {
          const labelKey = expression.replace('$labels.', '');
          return context.labels[labelKey] || '';
        }
        
        return match;
      } catch (error) {
        console.warn(`Template rendering error: ${error.message}`);
        return match;
      }
    });
  }

  formatValue(value, formatter) {
    switch (formatter) {
      case 'humanizePercentage':
        return `${(value * 100).toFixed(2)}%`;
      case 'humanizeBytes':
        return this.humanizeBytes(value);
      case 'humanizeDuration':
        return this.humanizeDuration(value);
      case 'humanize':
        return this.humanizeNumber(value);
      default:
        return value.toString();
    }
  }

  humanizeBytes(bytes) {
    const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(2)} ${units[unitIndex]}`;
  }

  humanizeDuration(seconds) {
    if (seconds < 60) return `${seconds.toFixed(1)}s`;
    if (seconds < 3600) return `${(seconds / 60).toFixed(1)}m`;
    if (seconds < 86400) return `${(seconds / 3600).toFixed(1)}h`;
    return `${(seconds / 86400).toFixed(1)}d`;
  }

  humanizeNumber(num) {
    if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`;
    if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`;
    if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`;
    return num.toFixed(2);
  }
}
```

### 4.2 Alert Evaluator

```javascript
class AlertEvaluator {
  constructor(config) {
    this.config = config;
    this.queryEngine = new QueryEngine(config.timeSeriesDB);
    this.anomalyDetector = new AnomalyDetector(config.anomalyDetection);
  }

  async executeQuery(query) {
    try {
      const result = await this.queryEngine.execute(query);
      return this.parseQueryResult(result);
    } catch (error) {
      console.error(`Query execution failed: ${query}`, error);
      throw error;
    }
  }

  parseQueryResult(result) {
    // Convert time series result to series array
    if (Array.isArray(result)) {
      return result.map(series => ({
        labels: series.metric || {},
        value: this.getLatestValue(series.values || [])
      }));
    }
    
    return [];
  }

  getLatestValue(values) {
    if (values.length === 0) return 0;
    
    // Values are [timestamp, value] pairs
    const latestValue = values[values.length - 1];
    return parseFloat(latestValue[1]) || 0;
  }

  shouldAlert(value, condition) {
    switch (condition.operator) {
      case 'gt':
      case '>':
        return value > condition.threshold;
      
      case 'gte':
      case '>=':
        return value >= condition.threshold;
      
      case 'lt':
      case '<':
        return value < condition.threshold;
      
      case 'lte':
      case '<=':
        return value <= condition.threshold;
      
      case 'eq':
      case '==':
        return Math.abs(value - condition.threshold) < 0.0001;
      
      case 'ne':
      case '!=':
        return Math.abs(value - condition.threshold) >= 0.0001;
      
      case 'anomaly':
        return this.evaluateAnomalyCondition(value, condition);
      
      default:
        console.warn(`Unknown condition operator: ${condition.operator}`);
        return false;
    }
  }

  async evaluateAnomalyCondition(value, condition) {
    try {
      const result = await this.anomalyDetector.detectAnomaly(
        condition.metric,
        value,
        condition.sensitivity || 0.8
      );
      
      return result.isAnomaly;
    } catch (error) {
      console.error('Anomaly detection failed:', error);
      return false;
    }
  }
}

class AnomalyDetector {
  constructor(config) {
    this.config = config;
    this.models = new Map();
    this.queryEngine = new QueryEngine(config.timeSeriesDB);
  }

  async detectAnomaly(metricName, currentValue, sensitivity = 0.8) {
    let model = this.models.get(metricName);
    
    if (!model || this.shouldUpdateModel(model)) {
      model = await this.trainModel(metricName);
      this.models.set(metricName, model);
    }
    
    const prediction = model.predict(currentValue);
    const anomalyScore = this.calculateAnomalyScore(currentValue, prediction, model);
    
    return {
      isAnomaly: anomalyScore > sensitivity,
      score: anomalyScore,
      prediction: prediction.value,
      confidence: prediction.confidence
    };
  }

  async trainModel(metricName) {
    // Get historical data for the last 30 days
    const endTime = new Date();
    const startTime = new Date(endTime.getTime() - 30 * 24 * 60 * 60 * 1000);
    
    const historicalData = await this.queryEngine.execute(
      `${metricName}[30d]`,
      { start: startTime, end: endTime }
    );
    
    if (historicalData.length < 100) {
      // Not enough data for meaningful anomaly detection
      return this.createSimpleModel();
    }
    
    return this.trainStatisticalModel(historicalData);
  }

  trainStatisticalModel(data) {
    const values = data.map(d => d.value);
    
    // Calculate basic statistics
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);
    
    // Calculate percentiles for more robust anomaly detection
    const sortedValues = [...values].sort((a, b) => a - b);
    const p5 = sortedValues[Math.floor(sortedValues.length * 0.05)];
    const p95 = sortedValues[Math.floor(sortedValues.length * 0.95)];
    
    return {
      type: 'statistical',
      mean,
      stdDev,
      p5,
      p95,
      lastTrained: new Date(),
      predict: (value) => ({
        value: mean,
        confidence: Math.max(0, Math.min(1, 1 - (stdDev / mean)))
      })
    };
  }

  createSimpleModel() {
    return {
      type: 'simple',
      lastTrained: new Date(),
      predict: (value) => ({
        value: value,
        confidence: 0.5
      })
    };
  }

  calculateAnomalyScore(actual, prediction, model) {
    if (model.type === 'simple') {
      return 0; // No anomaly detection for simple model
    }
    
    // Use z-score for anomaly detection
    const zScore = Math.abs(actual - model.mean) / model.stdDev;
    
    // Normalize to 0-1 range (3 standard deviations = score of 1)
    return Math.min(1, zScore / 3);
  }

  shouldUpdateModel(model) {
    const maxAge = 24 * 60 * 60 * 1000; // 24 hours
    return Date.now() - model.lastTrained.getTime() > maxAge;
  }
}
```

This comprehensive technical design specification provides the foundation for implementing a robust, scalable, and feature-rich monitoring and alerting system with real-time capabilities, advanced anomaly detection, and multi-channel notification support.