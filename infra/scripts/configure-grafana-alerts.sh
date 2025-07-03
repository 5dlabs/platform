#!/bin/bash
set -e

# Grafana API configuration
GRAFANA_URL="http://localhost:3000"
GRAFANA_USER="admin"
GRAFANA_PASS="admin123!"

# Wait for Grafana to be ready
echo "Waiting for Grafana to be ready..."
until curl -s "${GRAFANA_URL}/api/health" > /dev/null; do
    echo "Grafana not ready yet, waiting..."
    sleep 5
done

echo "Grafana is ready. Configuring alerts..."

# Create folder for Claude Code alerts if it doesn't exist
FOLDER_RESPONSE=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{"title":"Claude Code Alerts","uid":"claude-code-alerts"}' \
  "${GRAFANA_URL}/api/folders" 2>/dev/null || true)

# Get folder UID
FOLDER_UID=$(echo "$FOLDER_RESPONSE" | jq -r '.uid // "claude-code-alerts"')

# Convert Prometheus alerting rules to Grafana unified alerting format
# This is a simplified example - in production you'd want a more robust converter

# Alert 1: High API Error Rate
curl -X POST \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{
    "uid": "claude-code-high-error-rate",
    "title": "Claude Code High Error Rate",
    "condition": "A",
    "data": [
      {
        "refId": "A",
        "queryType": "",
        "relativeTimeRange": {
          "from": 300,
          "to": 0
        },
        "datasourceUid": "VictoriaMetrics",
        "model": {
          "expr": "(sum(rate(claude_code_api_error[5m])) by (github_user, working_service) / sum(rate(claude_code_api_request[5m])) by (github_user, working_service)) * 100",
          "refId": "A",
          "interval": "",
          "datasource": {
            "type": "prometheus",
            "uid": "VictoriaMetrics"
          }
        }
      }
    ],
    "noDataState": "NoData",
    "execErrState": "Alerting",
    "for": "5m",
    "annotations": {
      "summary": "High API error rate for Claude Code",
      "description": "Error rate is above 5% threshold"
    },
    "labels": {
      "severity": "warning",
      "component": "claude-code"
    },
    "folderUID": "'"${FOLDER_UID}"'"
  }' \
  "${GRAFANA_URL}/api/v1/provisioning/alert-rules" || echo "Alert rule already exists or error creating"

# Alert 2: High User Spend
curl -X POST \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{
    "uid": "claude-code-high-user-spend",
    "title": "Claude Code High User Spend",
    "condition": "A",
    "data": [
      {
        "refId": "A",
        "queryType": "",
        "relativeTimeRange": {
          "from": 3600,
          "to": 0
        },
        "datasourceUid": "VictoriaMetrics",
        "model": {
          "expr": "sum(increase(claude_code_cost_usage[1h])) by (github_user, working_service)",
          "refId": "A",
          "interval": "",
          "datasource": {
            "type": "prometheus",
            "uid": "VictoriaMetrics"
          }
        }
      }
    ],
    "noDataState": "NoData",
    "execErrState": "Alerting",
    "for": "5m",
    "annotations": {
      "summary": "High spending detected",
      "description": "User spending over $100/hour"
    },
    "labels": {
      "severity": "warning",
      "component": "claude-code",
      "cost_alert": "true"
    },
    "folderUID": "'"${FOLDER_UID}"'"
  }' \
  "${GRAFANA_URL}/api/v1/provisioning/alert-rules" || echo "Alert rule already exists or error creating"

# Alert 3: Component Down
curl -X POST \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{
    "uid": "telemetry-component-down",
    "title": "Telemetry Component Down",
    "condition": "A",
    "data": [
      {
        "refId": "A",
        "queryType": "",
        "relativeTimeRange": {
          "from": 300,
          "to": 0
        },
        "datasourceUid": "VictoriaMetrics",
        "model": {
          "expr": "up{namespace=\"telemetry\"}",
          "refId": "A",
          "interval": "",
          "datasource": {
            "type": "prometheus",
            "uid": "VictoriaMetrics"
          }
        }
      }
    ],
    "noDataState": "Alerting",
    "execErrState": "Alerting",
    "for": "2m",
    "annotations": {
      "summary": "Telemetry component is down",
      "description": "One or more telemetry components are not responding"
    },
    "labels": {
      "severity": "critical",
      "component": "infrastructure"
    },
    "folderUID": "'"${FOLDER_UID}"'"
  }' \
  "${GRAFANA_URL}/api/v1/provisioning/alert-rules" || echo "Alert rule already exists or error creating"

# Alert 4: High Memory Usage
curl -X POST \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{
    "uid": "high-memory-usage",
    "title": "High Memory Usage",
    "condition": "A",
    "data": [
      {
        "refId": "A",
        "queryType": "",
        "relativeTimeRange": {
          "from": 300,
          "to": 0
        },
        "datasourceUid": "VictoriaMetrics",
        "model": {
          "expr": "(container_memory_working_set_bytes{namespace=\"telemetry\",container!=\"\"} / container_spec_memory_limit_bytes{namespace=\"telemetry\",container!=\"\"}) * 100",
          "refId": "A",
          "interval": "",
          "datasource": {
            "type": "prometheus",
            "uid": "VictoriaMetrics"
          }
        }
      }
    ],
    "noDataState": "NoData",
    "execErrState": "Alerting",
    "for": "5m",
    "annotations": {
      "summary": "High memory usage detected",
      "description": "Container memory usage is above 80%"
    },
    "labels": {
      "severity": "warning",
      "component": "infrastructure"
    },
    "folderUID": "'"${FOLDER_UID}"'"
  }' \
  "${GRAFANA_URL}/api/v1/provisioning/alert-rules" || echo "Alert rule already exists or error creating"

# Create notification policy (route alerts to default contact point)
curl -X PUT \
  -H "Content-Type: application/json" \
  -u "${GRAFANA_USER}:${GRAFANA_PASS}" \
  -d '{
    "receiver": "grafana-default-email",
    "group_by": ["alertname", "cluster", "service"],
    "group_wait": "30s",
    "group_interval": "5m",
    "repeat_interval": "12h",
    "routes": [
      {
        "receiver": "grafana-default-email",
        "matchers": [
          "severity=critical"
        ],
        "continue": true,
        "group_wait": "10s",
        "group_interval": "2m",
        "repeat_interval": "1h"
      },
      {
        "receiver": "grafana-default-email",
        "matchers": [
          "cost_alert=true"
        ],
        "continue": true,
        "group_interval": "5m",
        "repeat_interval": "4h"
      }
    ]
  }' \
  "${GRAFANA_URL}/api/v1/provisioning/policies" || echo "Notification policy already exists or error creating"

echo "Alert configuration complete!"