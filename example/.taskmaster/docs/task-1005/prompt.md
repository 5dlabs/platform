# AI Agent Prompt: Implement Monitoring and Alerting System

## Task Context

You are tasked with implementing a comprehensive monitoring and alerting system for application health and performance. This system must provide real-time visibility, proactive alerting, and actionable insights through customizable dashboards and multiple notification channels.

## Primary Objective

Build a robust monitoring and alerting system that includes:
- Comprehensive metrics collection (application, system, business KPIs)
- Real-time alerting with threshold-based and anomaly detection
- Multi-channel notification system (email, Slack, webhooks, SMS)
- Interactive dashboards with customizable visualizations
- Historical data analysis and trending capabilities
- Integration with external monitoring systems (Prometheus, Grafana, DataDog)
- SLA monitoring and compliance reporting

## Technical Requirements

### Core Components to Implement

1. **Metrics Collection System**
   - Application performance metrics instrumentation
   - System resource monitoring (CPU, memory, disk, network)
   - Business metrics and KPI tracking
   - Custom metrics API and SDK
   - Time series data storage and retention management

2. **Alerting Engine**
   - Configurable alert rules with multiple criteria types
   - Threshold-based alerting (static and dynamic thresholds)
   - Anomaly detection using statistical models
   - Alert correlation and noise reduction
   - Escalation policies and routing logic

3. **Notification System**
   - Multi-channel notification delivery
   - Notification templates and customization
   - Rate limiting and notification suppression
   - Delivery tracking and retry mechanisms
   - On-call schedule integration

4. **Dashboard and Visualization**
   - Real-time dashboard with customizable widgets
   - Interactive charts and visualizations
   - Drill-down capabilities and data exploration
   - Dashboard sharing and embedding
   - Mobile-responsive design

### Performance Requirements

- Metrics ingestion: Handle 100,000+ metrics per second
- Alert evaluation: Complete within 30 seconds of threshold breach
- Dashboard loading: Render within 2 seconds for standard timeframes
- Data retention: 90 days for high-resolution, 2 years for downsampled data
- System overhead: Less than 2% impact on monitored applications

### Scalability Requirements

- Horizontal scaling for metrics collection and storage
- Distributed alert evaluation across multiple instances
- High availability with failover capabilities
- Multi-region deployment support
- Auto-scaling based on metric ingestion rate

## Implementation Approach

### Phase 1: Metrics Collection Infrastructure
1. Design time series database schema and storage
2. Implement metrics collection SDK and instrumentation
3. Create metrics ingestion API with batching and compression
4. Set up automatic system metrics collection
5. Add metrics retention and downsampling policies

### Phase 2: Alerting System
1. Build alert rule engine with flexible query language
2. Implement threshold-based and anomaly detection algorithms
3. Create alert correlation and grouping logic
4. Add escalation policies and notification routing
5. Build alert management UI and API

### Phase 3: Notification System
1. Implement multi-channel notification handlers
2. Create notification templates and rendering engine
3. Add rate limiting and suppression mechanisms
4. Build delivery tracking and retry logic
5. Integrate with external notification services

### Phase 4: Dashboard and Analytics
1. Create dashboard builder with drag-and-drop interface
2. Implement various chart types and visualizations
3. Add real-time data streaming and updates
4. Build query builder and data exploration tools
5. Add dashboard sharing and export capabilities

## Code Structure Expectations

```
src/
├── metrics/
│   ├── collectors/
│   │   ├── application-collector.js
│   │   ├── system-collector.js
│   │   └── business-collector.js
│   ├── storage/
│   │   ├── timeseries-storage.js
│   │   ├── retention-manager.js
│   │   └── downsampler.js
│   ├── ingestion/
│   │   ├── metrics-api.js
│   │   ├── batch-processor.js
│   │   └── compression.js
│   └── exporters/
│       ├── prometheus-exporter.js
│       └── grafana-exporter.js
├── alerting/
│   ├── engine/
│   │   ├── alert-manager.js
│   │   ├── rule-engine.js
│   │   └── evaluator.js
│   ├── detection/
│   │   ├── threshold-detector.js
│   │   ├── anomaly-detector.js
│   │   └── trend-detector.js
│   ├── correlation/
│   │   ├── alert-correlator.js
│   │   └── noise-reducer.js
│   └── policies/
│       ├── escalation-manager.js
│       └── routing-engine.js
├── notifications/
│   ├── channels/
│   │   ├── email-channel.js
│   │   ├── slack-channel.js
│   │   ├── webhook-channel.js
│   │   └── sms-channel.js
│   ├── templates/
│   │   ├── template-engine.js
│   │   └── template-manager.js
│   ├── delivery/
│   │   ├── delivery-manager.js
│   │   ├── rate-limiter.js
│   │   └── retry-handler.js
│   └── tracking/
│       └── delivery-tracker.js
├── dashboards/
│   ├── builders/
│   │   ├── dashboard-builder.js
│   │   └── widget-factory.js
│   ├── visualizations/
│   │   ├── chart-renderer.js
│   │   ├── table-renderer.js
│   │   └── stat-renderer.js
│   ├── queries/
│   │   ├── query-engine.js
│   │   └── query-optimizer.js
│   └── sharing/
│       ├── dashboard-sharing.js
│       └── embed-generator.js
└── tests/
    ├── unit/
    ├── integration/
    └── performance/
```

## Metrics Collection Implementation

### Metrics SDK and Instrumentation
```javascript
class MetricsCollector {
  constructor(config) {
    this.config = config;
    this.registry = new MetricsRegistry();
    this.storage = new TimeSeriesStorage(config.storage);
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

  // HTTP request instrumentation
  instrumentHttpRequests(app) {
    app.use((req, res, next) => {
      const startTime = Date.now();
      
      res.on('finish', () => {
        const duration = Date.now() - startTime;
        const labels = {
          method: req.method,
          endpoint: req.route?.path || req.path,
          status: res.statusCode.toString()
        };
        
        this.incrementCounter('http_requests_total', labels);
        this.observeHistogram('http_request_duration_ms', labels, duration);
      });
      
      next();
    });
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
    await this.db.query(`
      INSERT INTO metric_samples (metric_id, timestamp, value, labels)
      VALUES ($1, $2, $3, $4)
      ON CONFLICT (metric_id, timestamp, labels) 
      DO UPDATE SET value = EXCLUDED.value
    `, [sample.metricId, sample.timestamp, sample.value, JSON.stringify(sample.labels)]);
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
    this.evaluationInterval = config.evaluationInterval || 30000;
    this.activeAlerts = new Map();
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

  shouldAlert(value, rule) {
    switch (rule.condition) {
      case 'greater_than':
        return value > rule.threshold;
      case 'less_than':
        return value < rule.threshold;
      case 'equals':
        return Math.abs(value - rule.threshold) < 0.001;
      case 'not_equals':
        return Math.abs(value - rule.threshold) >= 0.001;
      default:
        return false;
    }
  }
}
```

### Anomaly Detection
```javascript
class AnomalyDetector {
  constructor(config) {
    this.config = config;
    this.models = new Map();
  }

  async detectAnomalies(metricName, currentValue, historicalData) {
    let model = this.models.get(metricName);
    
    if (!model) {
      model = await this.trainModel(metricName, historicalData);
      this.models.set(metricName, model);
    }
    
    const prediction = model.predict(currentValue);
    const anomalyScore = this.calculateAnomalyScore(currentValue, prediction);
    
    return {
      isAnomaly: anomalyScore > this.config.anomalyThreshold,
      score: anomalyScore,
      prediction: prediction,
      confidence: model.confidence
    };
  }

  async trainModel(metricName, historicalData) {
    // Simple moving average with standard deviation for anomaly detection
    const values = historicalData.map(d => d.value);
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);
    
    return {
      mean,
      stdDev,
      predict: (value) => mean,
      confidence: Math.max(0, Math.min(1, 1 - (stdDev / mean)))
    };
  }

  calculateAnomalyScore(actual, predicted) {
    return Math.abs(actual - predicted) / Math.max(predicted, 1);
  }
}
```

## Notification System Implementation

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
    if (await this.rateLimiter.isRateLimited(channel, alert)) {
      console.log(`Notification rate limited for channel ${channel.name}`);
      return;
    }
    
    const template = await this.templates.getTemplate(channel.type, action);
    const content = await this.renderNotification(template, alert, action);
    
    const channelHandler = this.channels[channel.type];
    await channelHandler.send(channel, content);
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
      from: channel.from,
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
      channel: channel.channel,
      username: 'AlertBot',
      text: content.subject,
      attachments: [{
        color: this.getSeverityColor(content.alert?.labels?.severity),
        text: content.body,
        ts: Math.floor(Date.now() / 1000)
      }]
    };
    
    await fetch(this.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
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
    const data = await renderer.render(widget, timeRange);
    
    return {
      ...widget,
      data
    };
  }
}

class MetricWidgetRenderer {
  async render(widget, timeRange) {
    const queryResult = await this.metricsService.query(widget.query, {
      start: timeRange.start,
      end: timeRange.end,
      step: widget.options.step || '1m'
    });
    
    switch (widget.visualization) {
      case 'line_chart':
        return this.renderLineChart(queryResult, widget.options);
      case 'single_stat':
        return this.renderSingleStat(queryResult, widget.options);
      case 'table':
        return this.renderTable(queryResult, widget.options);
      default:
        throw new Error(`Unknown visualization: ${widget.visualization}`);
    }
  }
}
```

## Testing Requirements

### Unit Tests (Minimum Coverage: 90%)
- Metrics collection and storage operations
- Alert rule evaluation and condition checking
- Notification delivery and template rendering
- Dashboard widget rendering and data transformation
- Anomaly detection algorithms
- Query execution and optimization

### Integration Tests
- End-to-end alert firing and notification delivery
- Dashboard data loading and real-time updates
- External system integrations (Prometheus, Grafana)
- Multi-channel notification scenarios
- Database operations and data consistency
- API endpoint functionality

### Performance Tests
- Metrics ingestion rate (target: 100,000 metrics/second)
- Alert evaluation latency (target: <30 seconds)
- Dashboard loading time (target: <2 seconds)
- Concurrent user dashboard access
- Memory usage under sustained load
- Database query performance optimization

## Environment Configuration

Required environment variables:
```env
# Database Configuration
TIMESERIES_DB_HOST=localhost
TIMESERIES_DB_PORT=5432
TIMESERIES_DB_NAME=monitoring
TIMESERIES_DB_USER=monitor_user
TIMESERIES_DB_PASSWORD=secure_password

# Metrics Configuration
METRICS_RETENTION_DAYS=90
METRICS_DOWNSAMPLING_ENABLED=true
METRICS_INGESTION_BATCH_SIZE=1000

# Alerting Configuration
ALERT_EVALUATION_INTERVAL=30000
ALERT_CORRELATION_ENABLED=true
ANOMALY_DETECTION_ENABLED=true

# Notification Configuration
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=alerts@example.com
SMTP_PASSWORD=smtp_password
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...

# Dashboard Configuration
DASHBOARD_REFRESH_INTERVAL=30000
DASHBOARD_CACHE_TTL=300
DASHBOARD_MAX_SERIES=1000

# Integration Configuration
PROMETHEUS_ENDPOINT=http://prometheus:9090
GRAFANA_URL=http://grafana:3000
GRAFANA_API_KEY=grafana_api_key
```

## Quality Assurance Checklist

Before marking this task complete, ensure:

- [ ] Metrics collection instruments all critical application components
- [ ] Time series storage handles high ingestion rates efficiently
- [ ] Alert rules evaluate correctly and trigger notifications
- [ ] Multi-channel notifications (email, Slack, webhook) work reliably
- [ ] Dashboards render within performance requirements
- [ ] Anomaly detection identifies actual anomalies with low false positive rate
- [ ] Historical data analysis and trending work correctly
- [ ] External system integrations (Prometheus, Grafana) function properly
- [ ] Rate limiting prevents notification spam
- [ ] System scales horizontally with increased load
- [ ] Security measures protect sensitive monitoring data
- [ ] Documentation covers all configuration and operational procedures
- [ ] Tests achieve minimum 90% coverage
- [ ] Performance requirements are met under expected load

## Success Metrics

- Metrics ingestion rate: 100,000+ metrics per second
- Alert evaluation latency: <30 seconds from threshold breach
- Dashboard loading time: <2 seconds for standard timeframes
- System availability: 99.9% uptime
- False positive rate: <5% for anomaly detection
- Notification delivery success rate: >99%
- Test coverage: >90%
- Performance overhead: <2% impact on monitored applications

## Important Notes

1. **Performance First**: Monitoring should not significantly impact application performance
2. **Reliability**: The monitoring system must be more reliable than the systems it monitors
3. **Scalability**: Design for horizontal scaling from day one
4. **Usability**: Dashboards and alerts should provide actionable insights
5. **Security**: Protect sensitive metrics and configuration data
6. **Integration**: Seamless integration with existing tooling and workflows
7. **Documentation**: Comprehensive operational procedures and troubleshooting guides

Begin implementation with the metrics collection infrastructure and time series storage. Focus on performance and scalability throughout the development process.