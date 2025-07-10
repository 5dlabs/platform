# Task 1005: Implement Monitoring and Alerting System

## Overview

This task involves building a comprehensive monitoring and alerting system for application health and performance. The system will track application metrics, system health, and business KPIs, featuring customizable alerting rules, multiple notification channels, and dashboard visualization with integration to existing telemetry infrastructure.

## Task Details

- **Priority**: High
- **Status**: Pending
- **Dependencies**: None
- **Estimated Effort**: 3-4 weeks

## Description

Create a monitoring system that tracks application metrics, system health, and business KPIs. The system should include customizable alerting rules, multiple notification channels, and dashboard visualization, with seamless integration to existing telemetry infrastructure.

## Implementation Guide

### Phase 1: Metrics Collection Infrastructure
- Set up comprehensive metrics collection for application and system monitoring
- Implement custom metrics instrumentation throughout the application
- Create metrics aggregation and storage system
- Build metrics export and integration capabilities

### Phase 2: Alerting Rules Engine
- Implement configurable alerting system with threshold-based and anomaly detection
- Create alerting rule management and validation
- Add escalation policies and alert routing
- Build alert suppression and correlation logic

### Phase 3: Dashboard and Visualization
- Create comprehensive monitoring dashboard with real-time metrics
- Implement customizable views for different stakeholders
- Add historical data analysis and trending
- Build automated reporting capabilities

### Phase 4: Integration and Advanced Features
- Integrate with external monitoring systems (Prometheus, Grafana, DataDog)
- Implement distributed tracing and performance monitoring
- Add SLA monitoring and compliance reporting
- Create advanced analytics and machine learning-based alerting

## Technical Requirements

### Core Components
- Metrics collection and instrumentation
- Time series database for metrics storage
- Alerting rules engine with multiple criteria types
- Multi-channel notification system
- Real-time dashboard with customizable widgets
- Historical data analysis and reporting

### Metrics Categories
- **Application Metrics**: Response times, error rates, throughput, business KPIs
- **System Metrics**: CPU, memory, disk, network utilization
- **Infrastructure Metrics**: Database performance, cache hit rates, queue depths
- **Business Metrics**: User activity, revenue, conversion rates

### Alerting Features
- Threshold-based alerts (static and dynamic thresholds)
- Anomaly detection using statistical models
- Alert correlation and noise reduction
- Escalation policies with multiple notification channels
- Alert acknowledgment and resolution tracking

## API Specifications

### Metrics API

#### POST /api/metrics
```json
{
  "metrics": [
    {
      "name": "http_requests_total",
      "type": "counter",
      "value": 1,
      "labels": {
        "method": "GET",
        "endpoint": "/api/users",
        "status": "200"
      },
      "timestamp": "2024-01-15T10:30:00Z"
    },
    {
      "name": "response_time_ms",
      "type": "histogram",
      "value": 150,
      "labels": {
        "endpoint": "/api/users"
      },
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### GET /api/metrics/query
```json
{
  "query": "avg(response_time_ms{endpoint='/api/users'}) by (endpoint)",
  "start": "2024-01-15T09:00:00Z",
  "end": "2024-01-15T10:00:00Z",
  "step": "1m"
}

Response:
{
  "success": true,
  "data": {
    "resultType": "matrix",
    "result": [
      {
        "metric": {
          "endpoint": "/api/users"
        },
        "values": [
          ["1705316400", "145.2"],
          ["1705316460", "152.8"],
          ["1705316520", "148.1"]
        ]
      }
    ]
  }
}
```

### Alerting API

#### GET /api/alerts/rules
```json
{
  "success": true,
  "data": {
    "rules": [
      {
        "id": "rule-uuid",
        "name": "High Error Rate",
        "description": "Alert when error rate exceeds 5%",
        "query": "rate(http_requests_total{status=~'5..'}[5m]) / rate(http_requests_total[5m]) > 0.05",
        "severity": "critical",
        "duration": "2m",
        "enabled": true,
        "labels": {
          "team": "backend",
          "service": "user-api"
        },
        "annotations": {
          "summary": "High error rate detected",
          "description": "Error rate is {{ $value | humanizePercentage }} for {{ $labels.endpoint }}"
        }
      }
    ]
  }
}
```

#### POST /api/alerts/rules
```json
{
  "name": "Database Connection Pool Exhaustion",
  "description": "Alert when database connection pool is nearly exhausted",
  "query": "db_connection_pool_active / db_connection_pool_max > 0.9",
  "severity": "warning",
  "duration": "1m",
  "enabled": true,
  "labels": {
    "team": "infrastructure",
    "component": "database"
  },
  "annotations": {
    "summary": "Database connection pool nearly exhausted",
    "description": "Connection pool utilization is {{ $value | humanizePercentage }}"
  }
}
```

#### GET /api/alerts/active
```json
{
  "success": true,
  "data": {
    "alerts": [
      {
        "id": "alert-uuid",
        "rule": "High Error Rate",
        "severity": "critical",
        "status": "firing",
        "startsAt": "2024-01-15T10:25:00Z",
        "endsAt": null,
        "labels": {
          "team": "backend",
          "service": "user-api",
          "endpoint": "/api/users"
        },
        "annotations": {
          "summary": "High error rate detected",
          "description": "Error rate is 8.5% for /api/users"
        },
        "value": "0.085",
        "acknowledgedBy": null,
        "acknowledgedAt": null
      }
    ]
  }
}
```

### Dashboard API

#### GET /api/dashboards
```json
{
  "success": true,
  "data": {
    "dashboards": [
      {
        "id": "dashboard-uuid",
        "name": "Application Overview",
        "description": "High-level application health and performance metrics",
        "tags": ["overview", "application"],
        "widgets": [
          {
            "id": "widget-uuid",
            "type": "metric",
            "title": "Request Rate",
            "query": "rate(http_requests_total[5m])",
            "visualization": "line_chart",
            "position": { "x": 0, "y": 0, "width": 6, "height": 4 }
          },
          {
            "id": "widget-uuid-2",
            "type": "metric",
            "title": "Error Rate",
            "query": "rate(http_requests_total{status=~'5..'}[5m])",
            "visualization": "single_stat",
            "position": { "x": 6, "y": 0, "width": 3, "height": 2 }
          }
        ]
      }
    ]
  }
}
```

#### POST /api/dashboards
```json
{
  "name": "Database Performance",
  "description": "Database performance and health metrics",
  "tags": ["database", "performance"],
  "widgets": [
    {
      "type": "metric",
      "title": "Query Duration",
      "query": "histogram_quantile(0.95, rate(db_query_duration_seconds_bucket[5m]))",
      "visualization": "line_chart",
      "position": { "x": 0, "y": 0, "width": 8, "height": 4 },
      "options": {
        "yAxis": { "unit": "seconds" },
        "legend": { "show": true }
      }
    }
  ]
}
```

## Database Schema

### Metrics Tables
```sql
CREATE TABLE metric_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    type metric_type_enum NOT NULL,
    description TEXT,
    unit VARCHAR(50),
    labels JSONB,
    retention_days INTEGER DEFAULT 90,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_metric_definitions_name (name),
    INDEX idx_metric_definitions_type (type)
);

CREATE TABLE metric_samples (
    metric_id UUID NOT NULL REFERENCES metric_definitions(id),
    timestamp TIMESTAMP NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB,
    
    PRIMARY KEY (metric_id, timestamp, labels),
    INDEX idx_metric_samples_timestamp (timestamp),
    INDEX idx_metric_samples_metric_timestamp (metric_id, timestamp)
);
```

### Alerting Tables
```sql
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query TEXT NOT NULL,
    severity alert_severity_enum NOT NULL,
    duration INTERVAL DEFAULT '1 minute',
    enabled BOOLEAN DEFAULT true,
    labels JSONB,
    annotations JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    INDEX idx_alert_rules_enabled (enabled),
    INDEX idx_alert_rules_severity (severity)
);

CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id),
    fingerprint VARCHAR(255) NOT NULL,
    status alert_status_enum DEFAULT 'firing',
    value DOUBLE PRECISION,
    labels JSONB,
    annotations JSONB,
    starts_at TIMESTAMP NOT NULL,
    ends_at TIMESTAMP,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMP,
    resolved_at TIMESTAMP,
    
    INDEX idx_alerts_rule_id (rule_id),
    INDEX idx_alerts_status (status),
    INDEX idx_alerts_starts_at (starts_at),
    INDEX idx_alerts_fingerprint (fingerprint)
);

CREATE TABLE alert_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES alerts(id),
    channel VARCHAR(100) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    status notification_status_enum DEFAULT 'pending',
    sent_at TIMESTAMP,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    
    INDEX idx_alert_notifications_alert_id (alert_id),
    INDEX idx_alert_notifications_status (status)
);
```

### Dashboard Tables
```sql
CREATE TABLE dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    tags TEXT[],
    layout JSONB,
    permissions JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    INDEX idx_dashboards_name (name),
    INDEX idx_dashboards_tags (tags),
    INDEX idx_dashboards_created_by (created_by)
);

CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    type widget_type_enum NOT NULL,
    query TEXT,
    visualization VARCHAR(50),
    position JSONB NOT NULL,
    options JSONB,
    
    INDEX idx_dashboard_widgets_dashboard_id (dashboard_id)
);
```

### Enums
```sql
CREATE TYPE metric_type_enum AS ENUM ('counter', 'gauge', 'histogram', 'summary');
CREATE TYPE alert_severity_enum AS ENUM ('info', 'warning', 'critical');
CREATE TYPE alert_status_enum AS ENUM ('firing', 'resolved', 'suppressed');
CREATE TYPE notification_status_enum AS ENUM ('pending', 'sent', 'failed');
CREATE TYPE widget_type_enum AS ENUM ('metric', 'log', 'alert', 'text', 'iframe');
```

## Metrics Collection Implementation

### Metrics Collector
```javascript
class MetricsCollector {
  constructor(config) {
    this.config = config;
    this.registry = new MetricsRegistry();
    this.storage = new TimeSeriesStorage(config.storage);
    this.exporters = this.initializeExporters(config.exporters);
    this.collectInterval = config.collectInterval || 15000; // 15 seconds
  }

  // Counter metric
  incrementCounter(name, labels = {}, value = 1) {
    const metric = this.registry.getOrCreateCounter(name, labels);
    metric.inc(value);
    
    this.recordSample(name, 'counter', value, labels);
  }

  // Gauge metric
  setGauge(name, labels = {}, value) {
    const metric = this.registry.getOrCreateGauge(name, labels);
    metric.set(value);
    
    this.recordSample(name, 'gauge', value, labels);
  }

  // Histogram metric
  observeHistogram(name, labels = {}, value) {
    const metric = this.registry.getOrCreateHistogram(name, labels);
    metric.observe(value);
    
    this.recordSample(name, 'histogram', value, labels);
  }

  // Record sample in time series database
  async recordSample(metricName, type, value, labels) {
    const sample = {
      metric: metricName,
      type,
      value,
      labels,
      timestamp: new Date()
    };
    
    await this.storage.writeSample(sample);
  }

  // Application performance monitoring
  measureExecutionTime(name, labels = {}) {
    const startTime = process.hrtime();
    
    return {
      end: () => {
        const [seconds, nanoseconds] = process.hrtime(startTime);
        const duration = seconds * 1000 + nanoseconds / 1000000; // Convert to milliseconds
        
        this.observeHistogram(`${name}_duration_ms`, labels, duration);
        return duration;
      }
    };
  }

  // HTTP request instrumentation
  instrumentHttpRequests(app) {
    app.use((req, res, next) => {
      const startTime = Date.now();
      
      res.on('finish', () => {
        const duration = Date.now() - startTime;
        const labels = {
          method: req.method,
          endpoint: this.normalizeEndpoint(req.route?.path || req.path),
          status: res.statusCode.toString()
        };
        
        this.incrementCounter('http_requests_total', labels);
        this.observeHistogram('http_request_duration_ms', labels, duration);
        
        if (res.statusCode >= 400) {
          this.incrementCounter('http_errors_total', labels);
        }
      });
      
      next();
    });
  }

  // Database query instrumentation
  instrumentDatabaseQueries(db) {
    const originalQuery = db.query;
    
    db.query = async function(sql, params) {
      const timer = this.measureExecutionTime('db_query', {
        operation: this.extractOperation(sql)
      });
      
      try {
        const result = await originalQuery.call(this, sql, params);
        timer.end();
        
        this.incrementCounter('db_queries_total', {
          operation: this.extractOperation(sql),
          status: 'success'
        });
        
        return result;
      } catch (error) {
        timer.end();
        
        this.incrementCounter('db_queries_total', {
          operation: this.extractOperation(sql),
          status: 'error'
        });
        
        throw error;
      }
    }.bind(this);
  }

  // System metrics collection
  async collectSystemMetrics() {
    const metrics = await this.gatherSystemMetrics();
    
    this.setGauge('system_cpu_usage_percent', {}, metrics.cpu.usage);
    this.setGauge('system_memory_usage_bytes', {}, metrics.memory.used);
    this.setGauge('system_memory_total_bytes', {}, metrics.memory.total);
    this.setGauge('system_disk_usage_bytes', {}, metrics.disk.used);
    this.setGauge('system_disk_total_bytes', {}, metrics.disk.total);
    
    // Network metrics
    Object.entries(metrics.network).forEach(([interface, stats]) => {
      this.incrementCounter('system_network_bytes_received_total', { interface }, stats.rx_bytes);
      this.incrementCounter('system_network_bytes_transmitted_total', { interface }, stats.tx_bytes);
    });
  }

  // Start automatic metrics collection
  startCollection() {
    setInterval(async () => {
      try {
        await this.collectSystemMetrics();
        await this.exportMetrics();
      } catch (error) {
        console.error('Metrics collection error:', error);
      }
    }, this.collectInterval);
  }

  // Export metrics to external systems
  async exportMetrics() {
    const metrics = await this.storage.getRecentMetrics();
    
    for (const exporter of this.exporters) {
      try {
        await exporter.export(metrics);
      } catch (error) {
        console.error(`Export failed for ${exporter.name}:`, error);
      }
    }
  }
}
```

### Time Series Storage
```javascript
class TimeSeriesStorage {
  constructor(config) {
    this.config = config;
    this.db = this.initializeDatabase(config);
    this.retentionManager = new RetentionManager(this, config.retention);
  }

  async writeSample(sample) {
    const metricId = await this.getOrCreateMetricId(sample.metric, sample.type);
    
    await this.db.query(`
      INSERT INTO metric_samples (metric_id, timestamp, value, labels)
      VALUES ($1, $2, $3, $4)
      ON CONFLICT (metric_id, timestamp, labels) 
      DO UPDATE SET value = EXCLUDED.value
    `, [metricId, sample.timestamp, sample.value, JSON.stringify(sample.labels)]);
  }

  async writeBatch(samples) {
    const values = [];
    const params = [];
    let paramIndex = 1;
    
    for (const sample of samples) {
      const metricId = await this.getOrCreateMetricId(sample.metric, sample.type);
      
      values.push(`($${paramIndex++}, $${paramIndex++}, $${paramIndex++}, $${paramIndex++})`);
      params.push(metricId, sample.timestamp, sample.value, JSON.stringify(sample.labels));
    }
    
    if (values.length > 0) {
      await this.db.query(`
        INSERT INTO metric_samples (metric_id, timestamp, value, labels)
        VALUES ${values.join(', ')}
        ON CONFLICT (metric_id, timestamp, labels) 
        DO UPDATE SET value = EXCLUDED.value
      `, params);
    }
  }

  async query(metricName, startTime, endTime, labels = {}) {
    const result = await this.db.query(`
      SELECT ms.timestamp, ms.value, ms.labels
      FROM metric_samples ms
      JOIN metric_definitions md ON ms.metric_id = md.id
      WHERE md.name = $1 
        AND ms.timestamp >= $2 
        AND ms.timestamp <= $3
        AND ($4::jsonb IS NULL OR ms.labels @> $4::jsonb)
      ORDER BY ms.timestamp
    `, [metricName, startTime, endTime, Object.keys(labels).length > 0 ? JSON.stringify(labels) : null]);
    
    return result.rows.map(row => ({
      timestamp: row.timestamp,
      value: row.value,
      labels: row.labels
    }));
  }

  async aggregateQuery(metricName, aggregation, startTime, endTime, step, labels = {}) {
    const stepSeconds = this.parseStep(step);
    
    const result = await this.db.query(`
      SELECT 
        date_trunc('seconds', ms.timestamp) - 
        (EXTRACT(epoch FROM date_trunc('seconds', ms.timestamp))::int % $5) * interval '1 second' as bucket,
        ${this.getAggregationFunction(aggregation)}(ms.value) as value
      FROM metric_samples ms
      JOIN metric_definitions md ON ms.metric_id = md.id
      WHERE md.name = $1 
        AND ms.timestamp >= $2 
        AND ms.timestamp <= $3
        AND ($4::jsonb IS NULL OR ms.labels @> $4::jsonb)
      GROUP BY bucket
      ORDER BY bucket
    `, [metricName, startTime, endTime, Object.keys(labels).length > 0 ? JSON.stringify(labels) : null, stepSeconds]);
    
    return result.rows.map(row => ({
      timestamp: row.bucket,
      value: parseFloat(row.value)
    }));
  }

  getAggregationFunction(aggregation) {
    const functions = {
      'avg': 'AVG',
      'sum': 'SUM',
      'min': 'MIN',
      'max': 'MAX',
      'count': 'COUNT',
      'stddev': 'STDDEV'
    };
    
    return functions[aggregation] || 'AVG';
  }

  parseStep(step) {
    const match = step.match(/^(\d+)([smhd])$/);
    if (!match) return 60; // Default to 1 minute
    
    const value = parseInt(match[1]);
    const unit = match[2];
    
    const multipliers = {
      's': 1,
      'm': 60,
      'h': 3600,
      'd': 86400
    };
    
    return value * (multipliers[unit] || 60);
  }
}
```

## Alerting Engine Implementation

### Alert Manager
```javascript
class AlertManager {
  constructor(config) {
    this.config = config;
    this.ruleEngine = new AlertRuleEngine(config);
    this.notificationManager = new NotificationManager(config.notifications);
    this.evaluationInterval = config.evaluationInterval || 30000; // 30 seconds
    this.activeAlerts = new Map();
  }

  async start() {
    console.log('Starting alert manager...');
    
    // Load alert rules
    await this.ruleEngine.loadRules();
    
    // Start rule evaluation loop
    this.startEvaluationLoop();
    
    // Start notification processing
    this.notificationManager.start();
  }

  startEvaluationLoop() {
    setInterval(async () => {
      try {
        await this.evaluateRules();
      } catch (error) {
        console.error('Alert evaluation error:', error);
      }
    }, this.evaluationInterval);
  }

  async evaluateRules() {
    const rules = await this.ruleEngine.getEnabledRules();
    
    for (const rule of rules) {
      try {
        await this.evaluateRule(rule);
      } catch (error) {
        console.error(`Rule evaluation failed for ${rule.name}:`, error);
      }
    }
  }

  async evaluateRule(rule) {
    const queryResult = await this.executeQuery(rule.query);
    
    for (const series of queryResult) {
      const alertKey = this.generateAlertKey(rule, series.labels);
      const isAlerting = this.shouldAlert(series.value, rule);
      
      if (isAlerting) {
        await this.handleAlertFiring(rule, series, alertKey);
      } else {
        await this.handleAlertResolved(rule, series, alertKey);
      }
    }
  }

  async handleAlertFiring(rule, series, alertKey) {
    let alert = this.activeAlerts.get(alertKey);
    
    if (!alert) {
      // New alert
      alert = await this.createAlert(rule, series);
      this.activeAlerts.set(alertKey, alert);
      
      console.log(`New alert firing: ${rule.name}`);
      await this.sendNotifications(alert, 'firing');
    } else {
      // Update existing alert
      alert.value = series.value;
      alert.lastEvaluated = new Date();
      
      await this.updateAlert(alert);
    }
  }

  async handleAlertResolved(rule, series, alertKey) {
    const alert = this.activeAlerts.get(alertKey);
    
    if (alert && alert.status === 'firing') {
      alert.status = 'resolved';
      alert.endsAt = new Date();
      alert.resolvedAt = new Date();
      
      await this.updateAlert(alert);
      this.activeAlerts.delete(alertKey);
      
      console.log(`Alert resolved: ${rule.name}`);
      await this.sendNotifications(alert, 'resolved');
    }
  }

  async createAlert(rule, series) {
    const alert = {
      id: crypto.randomUUID(),
      ruleId: rule.id,
      fingerprint: this.generateFingerprint(rule, series.labels),
      status: 'firing',
      value: series.value,
      labels: { ...rule.labels, ...series.labels },
      annotations: this.renderAnnotations(rule.annotations, series),
      startsAt: new Date(),
      endsAt: null,
      lastEvaluated: new Date()
    };
    
    await this.storeAlert(alert);
    return alert;
  }

  renderAnnotations(templates, series) {
    const annotations = {};
    
    for (const [key, template] of Object.entries(templates)) {
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
        // Simple template evaluation
        if (expression.startsWith('$value')) {
          const formatter = expression.split('|')[1]?.trim();
          return this.formatValue(context.value, formatter);
        }
        
        if (expression.startsWith('$labels.')) {
          const labelKey = expression.replace('$labels.', '');
          return context.labels[labelKey] || '';
        }
        
        return match;
      } catch (error) {
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
      default:
        return value.toString();
    }
  }

  async sendNotifications(alert, action) {
    const channels = await this.getNotificationChannels(alert.labels);
    
    for (const channel of channels) {
      try {
        await this.notificationManager.sendNotification(channel, alert, action);
      } catch (error) {
        console.error(`Notification failed for channel ${channel.name}:`, error);
      }
    }
  }

  generateAlertKey(rule, labels) {
    const sortedLabels = Object.keys(labels)
      .sort()
      .map(key => `${key}="${labels[key]}"`)
      .join(',');
    
    return `${rule.id}:${sortedLabels}`;
  }

  generateFingerprint(rule, labels) {
    const crypto = require('crypto');
    const content = `${rule.name}:${JSON.stringify(labels)}`;
    return crypto.createHash('md5').update(content).digest('hex');
  }
}
```

### Notification Manager
```javascript
class NotificationManager {
  constructor(config) {
    this.config = config;
    this.channels = this.initializeChannels(config.channels);
    this.templates = new NotificationTemplateManager(config.templates);
    this.rateLimiter = new NotificationRateLimiter(config.rateLimiting);
  }

  async sendNotification(channel, alert, action) {
    // Check rate limiting
    if (await this.rateLimiter.isRateLimited(channel, alert)) {
      console.log(`Notification rate limited for channel ${channel.name}`);
      return;
    }
    
    // Get notification template
    const template = await this.templates.getTemplate(channel.type, action);
    
    // Render notification content
    const content = await this.renderNotification(template, alert, action);
    
    // Send notification
    const channelHandler = this.channels[channel.type];
    if (!channelHandler) {
      throw new Error(`Unknown notification channel type: ${channel.type}`);
    }
    
    const notification = {
      id: crypto.randomUUID(),
      alertId: alert.id,
      channel: channel.name,
      recipient: channel.recipient,
      content,
      status: 'pending',
      retryCount: 0
    };
    
    try {
      await channelHandler.send(channel, content);
      notification.status = 'sent';
      notification.sentAt = new Date();
      
      console.log(`Notification sent via ${channel.type} to ${channel.recipient}`);
    } catch (error) {
      notification.status = 'failed';
      notification.errorMessage = error.message;
      
      console.error(`Notification failed:`, error);
    }
    
    await this.storeNotification(notification);
    return notification;
  }

  async renderNotification(template, alert, action) {
    const context = {
      alert,
      action,
      timestamp: new Date().toISOString(),
      severity: alert.labels.severity || 'unknown'
    };
    
    return {
      subject: this.renderTemplate(template.subject, context),
      body: this.renderTemplate(template.body, context),
      html: template.html ? this.renderTemplate(template.html, context) : null
    };
  }
}

// Email notification channel
class EmailNotificationChannel {
  constructor(config) {
    this.transporter = nodemailer.createTransporter(config.smtp);
  }

  async send(channel, content) {
    const mailOptions = {
      from: channel.from || this.config.defaultFrom,
      to: channel.recipient,
      subject: content.subject,
      text: content.body,
      html: content.html
    };
    
    await this.transporter.sendMail(mailOptions);
  }
}

// Slack notification channel
class SlackNotificationChannel {
  constructor(config) {
    this.webhookUrl = config.webhookUrl;
  }

  async send(channel, content) {
    const payload = {
      channel: channel.channel || '#alerts',
      username: 'AlertBot',
      text: content.subject,
      attachments: [
        {
          color: this.getSeverityColor(content.alert?.labels?.severity),
          text: content.body,
          ts: Math.floor(Date.now() / 1000)
        }
      ]
    };
    
    const response = await fetch(this.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    
    if (!response.ok) {
      throw new Error(`Slack notification failed: ${response.statusText}`);
    }
  }

  getSeverityColor(severity) {
    const colors = {
      'info': 'good',
      'warning': 'warning',
      'critical': 'danger'
    };
    
    return colors[severity] || 'good';
  }
}

// Webhook notification channel
class WebhookNotificationChannel {
  async send(channel, content) {
    const payload = {
      alert: content.alert,
      action: content.action,
      timestamp: new Date().toISOString(),
      content: content
    };
    
    const response = await fetch(channel.url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...channel.headers
      },
      body: JSON.stringify(payload)
    });
    
    if (!response.ok) {
      throw new Error(`Webhook notification failed: ${response.statusText}`);
    }
  }
}
```

## Dashboard Implementation

### Dashboard Service
```javascript
class DashboardService {
  constructor(config) {
    this.config = config;
    this.storage = new DashboardStorage(config.storage);
    this.metricsService = new MetricsService(config.metrics);
    this.widgetRenderers = this.initializeWidgetRenderers();
  }

  async createDashboard(dashboardData) {
    const dashboard = {
      id: crypto.randomUUID(),
      name: dashboardData.name,
      description: dashboardData.description,
      tags: dashboardData.tags || [],
      layout: dashboardData.layout || { columns: 12, rows: 'auto' },
      permissions: dashboardData.permissions || { public: true },
      widgets: [],
      createdAt: new Date(),
      updatedAt: new Date()
    };
    
    await this.storage.saveDashboard(dashboard);
    
    // Create widgets
    if (dashboardData.widgets) {
      for (const widgetData of dashboardData.widgets) {
        await this.addWidget(dashboard.id, widgetData);
      }
    }
    
    return dashboard;
  }

  async addWidget(dashboardId, widgetData) {
    const widget = {
      id: crypto.randomUUID(),
      dashboardId,
      title: widgetData.title,
      type: widgetData.type,
      query: widgetData.query,
      visualization: widgetData.visualization,
      position: widgetData.position,
      options: widgetData.options || {}
    };
    
    await this.storage.saveWidget(widget);
    return widget;
  }

  async renderDashboard(dashboardId, timeRange) {
    const dashboard = await this.storage.getDashboard(dashboardId);
    const widgets = await this.storage.getWidgets(dashboardId);
    
    const renderedWidgets = await Promise.all(
      widgets.map(widget => this.renderWidget(widget, timeRange))
    );
    
    return {
      ...dashboard,
      widgets: renderedWidgets
    };
  }

  async renderWidget(widget, timeRange) {
    const renderer = this.widgetRenderers[widget.type];
    if (!renderer) {
      throw new Error(`Unknown widget type: ${widget.type}`);
    }
    
    const data = await renderer.render(widget, timeRange);
    
    return {
      ...widget,
      data
    };
  }
}

class MetricWidgetRenderer {
  constructor(metricsService) {
    this.metricsService = metricsService;
  }

  async render(widget, timeRange) {
    const queryResult = await this.metricsService.query(widget.query, {
      start: timeRange.start,
      end: timeRange.end,
      step: widget.options.step || '1m'
    });
    
    switch (widget.visualization) {
      case 'line_chart':
        return this.renderLineChart(queryResult, widget.options);
      case 'bar_chart':
        return this.renderBarChart(queryResult, widget.options);
      case 'single_stat':
        return this.renderSingleStat(queryResult, widget.options);
      case 'gauge':
        return this.renderGauge(queryResult, widget.options);
      case 'table':
        return this.renderTable(queryResult, widget.options);
      default:
        throw new Error(`Unknown visualization: ${widget.visualization}`);
    }
  }

  renderLineChart(queryResult, options) {
    return {
      type: 'line_chart',
      series: queryResult.map(series => ({
        name: this.formatSeriesName(series.metric),
        data: series.values.map(([timestamp, value]) => ({
          x: new Date(timestamp * 1000),
          y: parseFloat(value)
        }))
      })),
      options: {
        xAxis: { type: 'time' },
        yAxis: { 
          type: 'linear',
          unit: options.unit,
          min: options.yMin,
          max: options.yMax
        },
        legend: { show: options.showLegend !== false }
      }
    };
  }

  renderSingleStat(queryResult, options) {
    let value = 0;
    
    if (queryResult.length > 0 && queryResult[0].values.length > 0) {
      const latestValue = queryResult[0].values[queryResult[0].values.length - 1];
      value = parseFloat(latestValue[1]);
    }
    
    return {
      type: 'single_stat',
      value,
      formattedValue: this.formatValue(value, options.unit),
      options: {
        unit: options.unit,
        decimals: options.decimals || 2,
        thresholds: options.thresholds || [],
        colorize: options.colorize || false
      }
    };
  }

  formatSeriesName(metric) {
    const labels = Object.entries(metric)
      .filter(([key]) => key !== '__name__')
      .map(([key, value]) => `${key}="${value}"`)
      .join(', ');
    
    return labels || metric.__name__ || 'unknown';
  }

  formatValue(value, unit) {
    switch (unit) {
      case 'bytes':
        return this.humanizeBytes(value);
      case 'percent':
        return `${(value * 100).toFixed(2)}%`;
      case 'seconds':
        return `${value.toFixed(3)}s`;
      case 'milliseconds':
        return `${value.toFixed(1)}ms`;
      default:
        return value.toFixed(2);
    }
  }
}
```

## Integration and Export

### Prometheus Exporter
```javascript
class PrometheusExporter {
  constructor(config) {
    this.config = config;
    this.registry = new prom.Registry();
    this.metrics = new Map();
  }

  async export(samples) {
    for (const sample of samples) {
      this.updateMetric(sample);
    }
    
    return this.registry.metrics();
  }

  updateMetric(sample) {
    const metricName = this.sanitizeMetricName(sample.metric);
    let metric = this.metrics.get(metricName);
    
    if (!metric) {
      metric = this.createPrometheusMetric(sample);
      this.metrics.set(metricName, metric);
      this.registry.registerMetric(metric);
    }
    
    this.updateMetricValue(metric, sample);
  }

  createPrometheusMetric(sample) {
    const labelNames = Object.keys(sample.labels);
    
    switch (sample.type) {
      case 'counter':
        return new prom.Counter({
          name: this.sanitizeMetricName(sample.metric),
          help: `Counter metric for ${sample.metric}`,
          labelNames
        });
      case 'gauge':
        return new prom.Gauge({
          name: this.sanitizeMetricName(sample.metric),
          help: `Gauge metric for ${sample.metric}`,
          labelNames
        });
      case 'histogram':
        return new prom.Histogram({
          name: this.sanitizeMetricName(sample.metric),
          help: `Histogram metric for ${sample.metric}`,
          labelNames,
          buckets: [0.1, 0.5, 1, 2.5, 5, 10, 25, 50, 100, 250, 500, 1000]
        });
      default:
        throw new Error(`Unsupported metric type: ${sample.type}`);
    }
  }

  sanitizeMetricName(name) {
    return name.replace(/[^a-zA-Z0-9_:]/g, '_');
  }
}
```

## Success Criteria

1. Comprehensive metrics collection covers application, system, and business metrics
2. Alerting system accurately detects and notifies about issues
3. Dashboard provides real-time visibility into system health and performance
4. Multi-channel notifications (email, Slack, webhooks) work reliably
5. Historical data analysis and trending capabilities are functional
6. Integration with external monitoring systems (Prometheus, Grafana) works
7. Performance impact of monitoring is minimal (< 2% overhead)
8. Alert correlation and noise reduction prevent alert fatigue
9. SLA monitoring and compliance reporting are accurate
10. System scales to handle high metric ingestion rates
11. Comprehensive test coverage (>90%) for all components
12. Documentation and runbooks are complete and accurate