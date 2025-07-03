#!/bin/bash

# Create dashboards directory if it doesn't exist
mkdir -p /Users/jonathonfritz/platform/manifests/dashboards

# Engineering Metrics Dashboard
cat > /Users/jonathonfritz/platform/manifests/dashboards/engineering-metrics-configmap.yaml << 'EOF'
apiVersion: v1
kind: ConfigMap
metadata:
  name: engineering-metrics-dashboard
  namespace: telemetry
  labels:
    grafana_dashboard: "1"
data:
  engineering-metrics.json: |
    {
      "annotations": {
        "list": [
          {
            "builtIn": 1,
            "datasource": {
              "type": "grafana",
              "uid": "-- Grafana --"
            },
            "enable": true,
            "hide": true,
            "iconColor": "rgba(0, 211, 255, 1)",
            "name": "Annotations & Alerts",
            "type": "dashboard"
          }
        ]
      },
      "editable": true,
      "fiscalYearStartMonth": 0,
      "graphTooltip": 0,
      "id": null,
      "links": [],
      "liveNow": false,
      "panels": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "axisCenteredZero": false,
                "axisColorMode": "text",
                "axisLabel": "",
                "axisPlacement": "auto",
                "fillOpacity": 20,
                "lineWidth": 1,
                "drawStyle": "line",
                "stacking": {
                  "mode": "normal",
                  "group": "A"
                }
              },
              "mappings": [],
              "unit": "short"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 0,
            "y": 0
          },
          "id": 1,
          "options": {
            "legend": {
              "calcs": [],
              "displayMode": "list",
              "placement": "bottom",
              "showLegend": true
            },
            "tooltip": {
              "mode": "multi",
              "sort": "none"
            }
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum by (language) (increase(claude_code_lines_modified_total[1h]))",
              "refId": "A",
              "legendFormat": "{{language}}"
            }
          ],
          "title": "Lines of Code Modified by Language",
          "type": "timeseries"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "mappings": [],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  }
                ]
              },
              "unit": "short"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 4,
            "w": 6,
            "x": 12,
            "y": 0
          },
          "id": 2,
          "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "auto",
            "orientation": "auto",
            "reduceOptions": {
              "calcs": ["lastNotNull"],
              "fields": "",
              "values": false
            },
            "text": {},
            "textMode": "auto"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum(increase(claude_code_pull_requests_created_total[24h]))",
              "refId": "A"
            }
          ],
          "title": "Pull Requests Created (24h)",
          "type": "stat"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "mappings": [],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  }
                ]
              },
              "unit": "short"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 4,
            "w": 6,
            "x": 18,
            "y": 0
          },
          "id": 3,
          "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "auto",
            "orientation": "auto",
            "reduceOptions": {
              "calcs": ["lastNotNull"],
              "fields": "",
              "values": false
            },
            "text": {},
            "textMode": "auto"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum(increase(claude_code_commits_total[24h]))",
              "refId": "A"
            }
          ],
          "title": "Commits (24h)",
          "type": "stat"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "hideFrom": {
                  "tooltip": false,
                  "viz": false,
                  "legend": false
                }
              },
              "mappings": []
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 12,
            "y": 4
          },
          "id": 4,
          "options": {
            "legend": {
              "displayMode": "list",
              "placement": "right",
              "showLegend": true
            },
            "pieType": "donut",
            "reduceOptions": {
              "values": false,
              "calcs": ["lastNotNull"],
              "fields": ""
            },
            "tooltip": {
              "mode": "single",
              "sort": "none"
            }
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum by (tool_name) (increase(claude_code_tool_usage_total[24h]))",
              "refId": "A",
              "legendFormat": "{{tool_name}}"
            }
          ],
          "title": "Tool Usage Distribution (24h)",
          "type": "piechart"
        },
        {
          "datasource": {
            "type": "victoriametrics-logs-datasource",
            "uid": "${logs_datasource}"
          },
          "gridPos": {
            "h": 8,
            "w": 24,
            "x": 0,
            "y": 8
          },
          "id": 5,
          "options": {
            "showTime": true,
            "showLabels": false,
            "showCommonLabels": false,
            "wrapLogMessage": true,
            "prettifyLogMessage": false,
            "enableLogDetails": true,
            "dedupStrategy": "none",
            "sortOrder": "Descending"
          },
          "targets": [
            {
              "datasource": {
                "type": "victoriametrics-logs-datasource",
                "uid": "${logs_datasource}"
              },
              "expr": "_stream:{app=\"claude-code\"} | json | line_format \"{{.timestamp}} [{{.level}}] {{.message}}\"",
              "refId": "A"
            }
          ],
          "title": "Developer Activity Timeline",
          "type": "logs"
        }
      ],
      "refresh": "30s",
      "schemaVersion": 39,
      "style": "dark",
      "tags": ["claude-code", "engineering"],
      "templating": {
        "list": [
          {
            "current": {
              "selected": false,
              "text": "VictoriaMetrics",
              "value": "P4169E866C3094E38"
            },
            "hide": 0,
            "includeAll": false,
            "label": "Metrics Data Source",
            "multi": false,
            "name": "datasource",
            "options": [],
            "query": "prometheus",
            "refresh": 1,
            "regex": "",
            "skipUrlSync": false,
            "type": "datasource"
          },
          {
            "current": {
              "selected": false,
              "text": "VictoriaLogs",
              "value": "PD775F2863313E6C7"
            },
            "hide": 0,
            "includeAll": false,
            "label": "Logs Data Source",
            "multi": false,
            "name": "logs_datasource",
            "options": [],
            "query": "victoriametrics-logs-datasource",
            "refresh": 1,
            "regex": "",
            "skipUrlSync": false,
            "type": "datasource"
          }
        ]
      },
      "time": {
        "from": "now-6h",
        "to": "now"
      },
      "timepicker": {},
      "timezone": "",
      "title": "Claude Code - Engineering Metrics",
      "uid": "claude-code-engineering",
      "version": 1,
      "weekStart": ""
    }
EOF

# Operations Monitoring Dashboard
cat > /Users/jonathonfritz/platform/manifests/dashboards/operations-monitoring-configmap.yaml << 'EOF'
apiVersion: v1
kind: ConfigMap
metadata:
  name: operations-monitoring-dashboard
  namespace: telemetry
  labels:
    grafana_dashboard: "1"
data:
  operations-monitoring.json: |
    {
      "annotations": {
        "list": [
          {
            "builtIn": 1,
            "datasource": {
              "type": "grafana",
              "uid": "-- Grafana --"
            },
            "enable": true,
            "hide": true,
            "iconColor": "rgba(0, 211, 255, 1)",
            "name": "Annotations & Alerts",
            "type": "dashboard"
          }
        ]
      },
      "editable": true,
      "fiscalYearStartMonth": 0,
      "graphTooltip": 0,
      "id": null,
      "links": [],
      "liveNow": false,
      "panels": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "mappings": [
                {
                  "options": {
                    "0": {
                      "color": "red",
                      "index": 0,
                      "text": "DOWN"
                    },
                    "1": {
                      "color": "green",
                      "index": 1,
                      "text": "UP"
                    }
                  },
                  "type": "value"
                }
              ],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "red",
                    "value": null
                  },
                  {
                    "color": "green",
                    "value": 1
                  }
                ]
              }
            },
            "overrides": []
          },
          "gridPos": {
            "h": 4,
            "w": 6,
            "x": 0,
            "y": 0
          },
          "id": 1,
          "options": {
            "colorMode": "background",
            "graphMode": "none",
            "justifyMode": "center",
            "orientation": "auto",
            "reduceOptions": {
              "calcs": ["lastNotNull"],
              "fields": "",
              "values": false
            },
            "text": {},
            "textMode": "value"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "up{job=\"otel-collector\"}",
              "refId": "A"
            }
          ],
          "title": "OTLP Collector Health",
          "type": "stat"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "mappings": [
                {
                  "options": {
                    "0": {
                      "color": "red",
                      "index": 0,
                      "text": "DOWN"
                    },
                    "1": {
                      "color": "green",
                      "index": 1,
                      "text": "UP"
                    }
                  },
                  "type": "value"
                }
              ],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "red",
                    "value": null
                  },
                  {
                    "color": "green",
                    "value": 1
                  }
                ]
              }
            },
            "overrides": []
          },
          "gridPos": {
            "h": 4,
            "w": 6,
            "x": 6,
            "y": 0
          },
          "id": 2,
          "options": {
            "colorMode": "background",
            "graphMode": "none",
            "justifyMode": "center",
            "orientation": "auto",
            "reduceOptions": {
              "calcs": ["lastNotNull"],
              "fields": "",
              "values": false
            },
            "text": {},
            "textMode": "value"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "up{job=\"victoria-metrics\"}",
              "refId": "A"
            }
          ],
          "title": "VictoriaMetrics Health",
          "type": "stat"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "axisCenteredZero": false,
                "axisColorMode": "text",
                "axisLabel": "",
                "axisPlacement": "auto",
                "barAlignment": 0,
                "drawStyle": "line",
                "fillOpacity": 10,
                "gradientMode": "none",
                "hideFrom": {
                  "tooltip": false,
                  "viz": false,
                  "legend": false
                },
                "insertNulls": false,
                "lineInterpolation": "linear",
                "lineWidth": 1,
                "pointSize": 5,
                "scaleDistribution": {
                  "type": "linear"
                },
                "showPoints": "never",
                "spanNulls": false,
                "stacking": {
                  "group": "A",
                  "mode": "none"
                },
                "thresholdsStyle": {
                  "mode": "area"
                }
              },
              "mappings": [],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  },
                  {
                    "color": "yellow",
                    "value": 0.01
                  },
                  {
                    "color": "red",
                    "value": 0.05
                  }
                ]
              },
              "unit": "percentunit"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 12,
            "y": 0
          },
          "id": 3,
          "options": {
            "legend": {
              "calcs": [],
              "displayMode": "list",
              "placement": "bottom",
              "showLegend": true
            },
            "tooltip": {
              "mode": "single",
              "sort": "none"
            }
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "rate(claude_code_api_errors_total[5m]) / rate(claude_code_api_requests_total[5m])",
              "refId": "A"
            }
          ],
          "title": "API Error Rate",
          "type": "timeseries"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "axisCenteredZero": false,
                "axisColorMode": "text",
                "axisLabel": "",
                "axisPlacement": "auto",
                "barAlignment": 0,
                "drawStyle": "line",
                "fillOpacity": 10,
                "gradientMode": "none",
                "hideFrom": {
                  "tooltip": false,
                  "viz": false,
                  "legend": false
                },
                "insertNulls": false,
                "lineInterpolation": "linear",
                "lineWidth": 1,
                "pointSize": 5,
                "scaleDistribution": {
                  "type": "linear"
                },
                "showPoints": "never",
                "spanNulls": false,
                "stacking": {
                  "group": "A",
                  "mode": "none"
                },
                "thresholdsStyle": {
                  "mode": "off"
                }
              },
              "mappings": [],
              "unit": "ms"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 0,
            "y": 4
          },
          "id": 4,
          "options": {
            "legend": {
              "calcs": [],
              "displayMode": "list",
              "placement": "bottom",
              "showLegend": true
            },
            "tooltip": {
              "mode": "multi",
              "sort": "none"
            }
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "histogram_quantile(0.95, sum(rate(claude_code_api_duration_bucket[5m])) by (le))",
              "refId": "A",
              "legendFormat": "p95"
            },
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "histogram_quantile(0.99, sum(rate(claude_code_api_duration_bucket[5m])) by (le))",
              "refId": "B",
              "legendFormat": "p99"
            }
          ],
          "title": "API Response Times",
          "type": "timeseries"
        },
        {
          "datasource": {
            "type": "victoriametrics-logs-datasource",
            "uid": "${logs_datasource}"
          },
          "gridPos": {
            "h": 8,
            "w": 24,
            "x": 0,
            "y": 12
          },
          "id": 5,
          "options": {
            "showTime": true,
            "showLabels": true,
            "showCommonLabels": false,
            "wrapLogMessage": true,
            "prettifyLogMessage": false,
            "enableLogDetails": true,
            "dedupStrategy": "none",
            "sortOrder": "Descending"
          },
          "targets": [
            {
              "datasource": {
                "type": "victoriametrics-logs-datasource",
                "uid": "${logs_datasource}"
              },
              "expr": "_stream:{app=\"claude-code\"} | json | level:error",
              "refId": "A"
            }
          ],
          "title": "Recent Errors",
          "type": "logs"
        }
      ],
      "refresh": "10s",
      "schemaVersion": 39,
      "style": "dark",
      "tags": ["claude-code", "operations"],
      "templating": {
        "list": [
          {
            "current": {
              "selected": false,
              "text": "VictoriaMetrics",
              "value": "P4169E866C3094E38"
            },
            "hide": 0,
            "includeAll": false,
            "label": "Metrics Data Source",
            "multi": false,
            "name": "datasource",
            "options": [],
            "query": "prometheus",
            "refresh": 1,
            "regex": "",
            "skipUrlSync": false,
            "type": "datasource"
          },
          {
            "current": {
              "selected": false,
              "text": "VictoriaLogs",
              "value": "PD775F2863313E6C7"
            },
            "hide": 0,
            "includeAll": false,
            "label": "Logs Data Source",
            "multi": false,
            "name": "logs_datasource",
            "options": [],
            "query": "victoriametrics-logs-datasource",
            "refresh": 1,
            "regex": "",
            "skipUrlSync": false,
            "type": "datasource"
          }
        ]
      },
      "time": {
        "from": "now-1h",
        "to": "now"
      },
      "timepicker": {},
      "timezone": "",
      "title": "Claude Code - Operations Monitoring",
      "uid": "claude-code-operations",
      "version": 1,
      "weekStart": ""
    }
EOF

# Cost Management Dashboard
cat > /Users/jonathonfritz/platform/manifests/dashboards/cost-management-configmap.yaml << 'EOF'
apiVersion: v1
kind: ConfigMap
metadata:
  name: cost-management-dashboard
  namespace: telemetry
  labels:
    grafana_dashboard: "1"
data:
  cost-management.json: |
    {
      "annotations": {
        "list": [
          {
            "builtIn": 1,
            "datasource": {
              "type": "grafana",
              "uid": "-- Grafana --"
            },
            "enable": true,
            "hide": true,
            "iconColor": "rgba(0, 211, 255, 1)",
            "name": "Annotations & Alerts",
            "type": "dashboard"
          }
        ]
      },
      "editable": true,
      "fiscalYearStartMonth": 0,
      "graphTooltip": 0,
      "id": null,
      "links": [],
      "liveNow": false,
      "panels": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "decimals": 2,
              "mappings": [],
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  },
                  {
                    "color": "yellow",
                    "value": 5
                  },
                  {
                    "color": "red",
                    "value": 10
                  }
                ]
              },
              "unit": "currencyUSD"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 6,
            "w": 6,
            "x": 0,
            "y": 0
          },
          "id": 1,
          "options": {
            "colorMode": "background",
            "graphMode": "area",
            "justifyMode": "center",
            "orientation": "auto",
            "reduceOptions": {
              "calcs": ["lastNotNull"],
              "fields": "",
              "values": false
            },
            "text": {},
            "textMode": "auto"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum(increase(claude_code_token_cost_dollars_total[1h]))",
              "refId": "A"
            }
          ],
          "title": "Current Hour Spend",
          "type": "stat"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "axisCenteredZero": false,
                "axisColorMode": "text",
                "axisLabel": "",
                "axisPlacement": "auto",
                "barAlignment": 0,
                "drawStyle": "line",
                "fillOpacity": 20,
                "gradientMode": "opacity",
                "hideFrom": {
                  "tooltip": false,
                  "viz": false,
                  "legend": false
                },
                "insertNulls": false,
                "lineInterpolation": "linear",
                "lineWidth": 2,
                "pointSize": 5,
                "scaleDistribution": {
                  "type": "linear"
                },
                "showPoints": "never",
                "spanNulls": false,
                "stacking": {
                  "group": "A",
                  "mode": "none"
                },
                "thresholdsStyle": {
                  "mode": "off"
                }
              },
              "mappings": [],
              "unit": "currencyUSD"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 18,
            "x": 6,
            "y": 0
          },
          "id": 2,
          "options": {
            "legend": {
              "calcs": ["sum"],
              "displayMode": "list",
              "placement": "bottom",
              "showLegend": true
            },
            "tooltip": {
              "mode": "single",
              "sort": "none"
            }
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum(increase(claude_code_token_cost_dollars_total[1d]))",
              "refId": "A"
            }
          ],
          "title": "Daily Cost Trend",
          "type": "timeseries"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "palette-classic"
              },
              "custom": {
                "axisCenteredZero": false,
                "axisColorMode": "text",
                "axisLabel": "",
                "axisPlacement": "auto",
                "fillOpacity": 80,
                "gradientMode": "none",
                "hideFrom": {
                  "tooltip": false,
                  "viz": false,
                  "legend": false
                },
                "lineWidth": 1,
                "scaleDistribution": {
                  "type": "linear"
                },
                "thresholdsStyle": {
                  "mode": "off"
                }
              },
              "mappings": [],
              "unit": "currencyUSD"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 0,
            "y": 8
          },
          "id": 3,
          "options": {
            "barRadius": 0,
            "barWidth": 0.97,
            "fullHighlight": false,
            "groupWidth": 0.7,
            "legend": {
              "calcs": [],
              "displayMode": "list",
              "placement": "bottom",
              "showLegend": true
            },
            "orientation": "auto",
            "showValue": "auto",
            "stacking": "normal",
            "tooltip": {
              "mode": "single",
              "sort": "none"
            },
            "xTickLabelRotation": 0,
            "xTickLabelSpacing": 0
          },
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum by (model) (increase(claude_code_token_cost_dollars_total[24h]))",
              "refId": "A",
              "legendFormat": "{{model}}"
            }
          ],
          "title": "Cost by Model (24h)",
          "type": "barchart"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "continuous-GrYlRd"
              },
              "mappings": [],
              "max": 100,
              "min": 0,
              "thresholds": {
                "mode": "absolute",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  }
                ]
              },
              "unit": "currencyUSD"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 8,
            "w": 12,
            "x": 12,
            "y": 8
          },
          "id": 4,
          "options": {
            "displayMode": "lcd",
            "minVizHeight": 10,
            "minVizWidth": 0,
            "orientation": "horizontal",
            "reduceOptions": {
              "values": false,
              "calcs": ["lastNotNull"],
              "fields": ""
            },
            "showUnfilled": false,
            "valueMode": "color"
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "topk(10, sum by (user_id) (increase(claude_code_token_cost_dollars_total[7d])))",
              "format": "table",
              "instant": true,
              "refId": "A"
            }
          ],
          "transformations": [
            {
              "id": "organize",
              "options": {
                "excludeByName": {
                  "Time": true
                },
                "indexByName": {},
                "renameByName": {
                  "user_id": "User",
                  "Value": "Cost"
                }
              }
            }
          ],
          "title": "User Cost Ranking (7d)",
          "type": "bargauge"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "${datasource}"
          },
          "fieldConfig": {
            "defaults": {
              "color": {
                "mode": "thresholds"
              },
              "mappings": [],
              "max": 1000,
              "min": 0,
              "thresholds": {
                "mode": "percentage",
                "steps": [
                  {
                    "color": "green",
                    "value": null
                  },
                  {
                    "color": "yellow",
                    "value": 70
                  },
                  {
                    "color": "red",
                    "value": 90
                  }
                ]
              },
              "unit": "percentunit"
            },
            "overrides": []
          },
          "gridPos": {
            "h": 6,
            "w": 6,
            "x": 0,
            "y": 6
          },
          "id": 5,
          "options": {
            "orientation": "auto",
            "reduceOptions": {
              "values": false,
              "calcs": ["lastNotNull"],
              "fields": ""
            },
            "showThresholdLabels": false,
            "showThresholdMarkers": true
          },
          "pluginVersion": "11.1.0",
          "targets": [
            {
              "datasource": {
                "type": "prometheus",
                "uid": "${datasource}"
              },
              "expr": "sum(increase(claude_code_token_cost_dollars_total[30d])) / 1000",
              "refId": "A"
            }
          ],
          "title": "Monthly Budget Burn ($1000)",
          "type": "gauge"
        }
      ],
      "refresh": "30s",
      "schemaVersion": 39,
      "style": "dark",
      "tags": ["claude-code", "cost"],
      "templating": {
        "list": [
          {
            "current": {
              "selected": false,
              "text": "VictoriaMetrics",
              "value": "P4169E866C3094E38"
            },
            "hide": 0,
            "includeAll": false,
            "label": "Data Source",
            "multi": false,
            "name": "datasource",
            "options": [],
            "query": "prometheus",
            "refresh": 1,
            "regex": "",
            "skipUrlSync": false,
            "type": "datasource"
          }
        ]
      },
      "time": {
        "from": "now-24h",
        "to": "now"
      },
      "timepicker": {},
      "timezone": "",
      "title": "Claude Code - Cost Management",
      "uid": "claude-code-cost",
      "version": 1,
      "weekStart": ""
    }
EOF

echo "Applying all dashboards..."
kubectl apply -f /Users/jonathonfritz/platform/manifests/dashboards/engineering-metrics-configmap.yaml
kubectl apply -f /Users/jonathonfritz/platform/manifests/dashboards/operations-monitoring-configmap.yaml
kubectl apply -f /Users/jonathonfritz/platform/manifests/dashboards/cost-management-configmap.yaml

echo "Dashboards created and applied successfully!"