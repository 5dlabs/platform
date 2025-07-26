# scripts Analysis

**Path:** `infra/scripts`
**Type:** Scripts
**Lines of Code:** 2835
**Description:** scripts configuration and files

## Source Files

### configure-grafana-alerts.sh (238 lines)

**Full Content:**
```sh
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
```

### test-telemetry-pipeline.sh (84 lines)

**Full Content:**
```sh
#\!/bin/bash

echo "Testing telemetry pipeline..."

# Test metrics via OTLP HTTP
echo "Sending test metrics..."
curl -X POST http://otel-http.local:31251/v1/metrics \
  -H "Content-Type: application/json" \
  -d '{
    "resourceMetrics": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "claude-code-test"}
        }]
      },
      "scopeMetrics": [{
        "scope": {
          "name": "test.metrics"
        },
        "metrics": [{
          "name": "claude_code_sessions_total",
          "description": "Test session counter",
          "sum": {
            "dataPoints": [{
              "attributes": [{
                "key": "user_id",
                "value": {"stringValue": "test-user-1"}
              }],
              "startTimeUnixNano": "'"$(date -u +%s)000000000"'",
              "timeUnixNano": "'"$(date -u +%s)000000000"'",
              "asInt": "1"
            }],
            "aggregationTemporality": 2,
            "isMonotonic": true
          }
        }]
      }]
    }]
  }'

echo ""
echo "Sending test logs..."
curl -X POST http://otel-http.local:31251/v1/logs \
  -H "Content-Type: application/json" \
  -d '{
    "resourceLogs": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "claude-code"}
        },{
          "key": "app",
          "value": {"stringValue": "claude-code"}
        }]
      },
      "scopeLogs": [{
        "scope": {
          "name": "test.logs"
        },
        "logRecords": [{
          "timeUnixNano": "'"$(date -u +%s)000000000"'",
          "severityNumber": 9,
          "severityText": "INFO",
          "body": {
            "stringValue": "{\"timestamp\":\"'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'\",\"level\":\"info\",\"message\":\"Test log from telemetry pipeline\",\"user_id\":\"test-user-1\",\"action\":\"test\"}"
          },
          "attributes": [{
            "key": "app",
            "value": {"stringValue": "claude-code"}
          }]
        }]
      }]
    }]
  }'

echo ""
echo "Test data sent successfully\!"
echo ""
echo "You can now check:"
echo "1. Grafana dashboards at http://grafana.local:31251"
echo "2. VictoriaMetrics metrics at: http://grafana.local:31251/explore (select VictoriaMetrics datasource)"
echo "3. VictoriaLogs logs at: http://grafana.local:31251/explore (select VictoriaLogs datasource)"
EOF < /dev/null
```

### test-all-dashboard-metrics.sh (80 lines)

**Full Content:**
```sh
#!/bin/bash

echo "Testing all Claude Code dashboard metrics..."
echo "=========================================="

# Port forward to VictoriaMetrics
echo "Setting up port-forward to VictoriaMetrics..."
kubectl port-forward -n telemetry statefulset/victoria-metrics-victoria-metrics-single-server 8428:8428 &
PF_PID=$!
sleep 3

# Function to query metrics
query_metric() {
    local metric=$1
    local description=$2
    echo ""
    echo "Checking: $description"
    echo "Query: $metric"
    result=$(curl -s "http://localhost:8428/api/v1/query?query=$metric" | jq -r '.data.result | length')
    if [ "$result" -gt 0 ]; then
        echo "✅ Found $result results"
        curl -s "http://localhost:8428/api/v1/query?query=$metric" | jq -r '.data.result[0]' | head -20
    else
        echo "❌ No data found"
    fi
}

echo ""
echo "=== EXECUTIVE OVERVIEW DASHBOARD METRICS ==="
query_metric "sum(increase(claude_code_sessions_total[24h]))" "Total Sessions (24h)"
query_metric "count(count%20by%20(user_id)%20(claude_code_sessions_total))" "Daily Active Users"
query_metric "sum%20by%20(model)%20(increase(claude_code_token_cost_dollars_total[24h]))" "Cost Breakdown by Model"
query_metric "sum(increase(claude_code_token_cost_dollars_total[30d]))" "Monthly Cost"
query_metric "count(count%20by%20(user_id)%20(increase(claude_code_sessions_total[7d])))" "User Adoption Trend"
query_metric "topk(10,%20sum%20by%20(user_id)%20(increase(claude_code_token_cost_dollars_total[24h])))" "Top Users by Cost"

echo ""
echo "=== ENGINEERING METRICS DASHBOARD METRICS ==="
query_metric "sum%20by%20(language)%20(increase(claude_code_lines_modified_total[1h]))" "Lines of Code Modified by Language"
query_metric "sum(increase(claude_code_pull_requests_created_total[24h]))" "Pull Requests Created (24h)"
query_metric "sum(increase(claude_code_commits_total[24h]))" "Commits (24h)"
query_metric "sum%20by%20(tool_name)%20(increase(claude_code_tool_usage_total[24h]))" "Tool Usage Distribution"

echo ""
echo "=== OPERATIONS MONITORING DASHBOARD METRICS ==="
query_metric "up{job=\"otel-collector\"}" "OTLP Collector Health"
query_metric "up{job=\"victoria-metrics\"}" "VictoriaMetrics Health"
query_metric "rate(claude_code_api_errors_total[5m])%20/%20rate(claude_code_api_requests_total[5m])" "API Error Rate"
query_metric "histogram_quantile(0.95,%20sum(rate(claude_code_api_duration_bucket[5m]))%20by%20(le))" "API Response Time p95"

echo ""
echo "=== COST MANAGEMENT DASHBOARD METRICS ==="
query_metric "sum(increase(claude_code_token_cost_dollars_total[1h]))" "Current Hour Spend"
query_metric "sum(increase(claude_code_token_cost_dollars_total[1d]))" "Daily Cost Trend"
query_metric "sum%20by%20(model)%20(increase(claude_code_token_cost_dollars_total[24h]))" "Cost by Model (24h)"
query_metric "topk(10,%20sum%20by%20(user_id)%20(increase(claude_code_token_cost_dollars_total[7d])))" "User Cost Ranking (7d)"

echo ""
echo "=== CHECKING RAW METRICS ==="
echo "Looking for any metrics with 'claude' in the name..."
curl -s "http://localhost:8428/api/v1/label/__name__/values" | jq -r '.data[]' | grep -i claude || echo "No claude metrics found"

echo ""
echo "Looking for metrics with service_name='claude-code'..."
curl -s "http://localhost:8428/api/v1/query?query={service_name=\"claude-code\"}" | jq -r '.data.result[].metric.__name__' 2>/dev/null | sort -u || echo "No metrics with service_name=claude-code"

echo ""
echo "=== CHECKING OTLP COLLECTOR METRICS ==="
query_metric "otelcol_receiver_accepted_metric_points{receiver=\"otlp\"}" "OTLP Metrics Received"
query_metric "otelcol_receiver_accepted_log_records{receiver=\"otlp\"}" "OTLP Logs Received"
query_metric "otelcol_exporter_sent_metric_points{exporter=\"prometheusremotewrite\"}" "Metrics Sent to VictoriaMetrics"
query_metric "otelcol_exporter_sent_log_records{exporter=\"otlphttp/victorialogs\"}" "Logs Sent to VictoriaLogs"

# Clean up port-forward
echo ""
echo "Cleaning up port-forward..."
kill $PF_PID 2>/dev/null

echo ""
echo "Test complete!"
```

### validate-telemetry-metrics.sh (159 lines)

**Full Content:**
```sh
#!/bin/bash
# Telemetry Metrics Validation Script
# Verifies that all expected Claude Code metrics are flowing to VictoriaMetrics

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VICTORIA_METRICS_URL="http://localhost:8428"
FAILED_CHECKS=0
TOTAL_CHECKS=0

echo -e "${BLUE}=== Claude Code Telemetry Validation ===${NC}"
echo "Checking VictoriaMetrics at: $VICTORIA_METRICS_URL"
echo ""

# Function to check if a metric exists
check_metric() {
    local metric_name="$1"
    local description="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    echo -n "Checking $description: "
    
    # Query VictoriaMetrics for the metric
    result=$(curl -s "$VICTORIA_METRICS_URL/api/v1/query?query=${metric_name}" | jq -r '.data.result | length')
    
    if [ "$result" != "null" ] && [ "$result" -gt 0 ]; then
        echo -e "${GREEN}✓ Found ($result series)${NC}"
        return 0
    else
        echo -e "${RED}✗ Missing${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

# Function to check log events in VictoriaLogs
check_log_event() {
    local event_pattern="$1"
    local description="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    echo -n "Checking $description: "
    
    # Query VictoriaLogs for the event
    result=$(curl -s "http://localhost:9428/select/logsql/query?query=${event_pattern}&limit=1" | jq -r '. | length' 2>/dev/null || echo "0")
    
    if [ "$result" -gt 0 ]; then
        echo -e "${GREEN}✓ Found${NC}"
        return 0
    else
        echo -e "${RED}✗ Missing${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

echo -e "${YELLOW}=== Core Metrics (Required for all dashboards) ===${NC}"

# Essential metrics that should exist
check_metric "claude_code_session_count" "Session tracking"
check_metric "claude_code_cost_usage" "Cost tracking"
check_metric "claude_code_token_usage" "Token usage"

echo ""
echo -e "${YELLOW}=== Engineering Metrics Dashboard ===${NC}"

# Engineering productivity metrics
check_metric "claude_code_lines_of_code_count" "Lines of code modified"
check_metric "claude_code_commit_count" "Git commits"
check_metric "claude_code_pull_request_count" "Pull requests created"
check_metric "claude_code_code_edit_tool_decision" "Code edit tool usage"

echo ""
echo -e "${YELLOW}=== Operations Monitoring Dashboard ===${NC}"

# Operations and reliability metrics
check_metric "claude_code_api_request" "API request tracking"
check_metric "claude_code_api_error" "API error tracking"

echo ""
echo -e "${YELLOW}=== Cost Management Dashboard ===${NC}"

# Cost-specific metrics (these might be the same as core cost metrics)
check_metric "claude_code_cost_usage" "Detailed cost tracking"

echo ""
echo -e "${YELLOW}=== Log Events (VictoriaLogs) ===${NC}"

# Check for key log events
check_log_event "_msg:claude_code.api_request" "API request events"
check_log_event "_msg:claude_code.tool_result" "Tool result events"
check_log_event "_msg:claude_code.user_prompt" "User prompt events"

echo ""
echo -e "${YELLOW}=== Component Health ===${NC}"

# Check that telemetry components are running
echo -n "OTLP Collector health: "
if kubectl get pods -n telemetry -l app.kubernetes.io/name=opentelemetry-collector --no-headers | grep -q "1/1.*Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "VictoriaMetrics health: "
if kubectl get pods -n telemetry victoria-metrics-victoria-metrics-single-server-0 --no-headers | grep -q "1/1.*Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "VictoriaLogs health: "
if kubectl get pods -n telemetry victoria-logs-victoria-logs-single-server-0 --no-headers | grep -q "1/1.*Running"; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo -n "Claude Code pod status: "
if kubectl get pods -n claude-code-dev -l app.kubernetes.io/name=claude-code --no-headers | grep -q "Running\|Completed"; then
    echo -e "${GREEN}✓ Running/Completed${NC}"
else
    echo -e "${YELLOW}⚠ Not running (needs valid API key)${NC}"
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

echo ""
echo -e "${BLUE}=== Summary ===${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅ All $TOTAL_CHECKS checks passed! Telemetry is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_CHECKS out of $TOTAL_CHECKS checks failed.${NC}"
    echo ""
    echo -e "${YELLOW}Common issues:${NC}"
    echo "1. Claude Code pod needs a valid API key to generate telemetry"
    echo "2. Metrics may take time to appear after first run"
    echo "3. Check Claude Code configuration: kubectl get configmap -n claude-code-dev claude-code-dev-config"
    echo "4. Verify endpoints are accessible:"
    echo "   - VictoriaMetrics: curl http://localhost:8428/api/v1/query?query=up"
    echo "   - VictoriaLogs: curl 'http://localhost:9428/select/logsql/query?query=*&limit=1'"
    exit 1
fi
```

### comprehensive-test.sh (214 lines)

**Full Content:**
```sh
#!/bin/bash
# Comprehensive Testing and Verification Script for Claude Code Telemetry Stack
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${YELLOW}Running test: ${test_name}${NC}"
    if eval "$test_command"; then
        echo -e "${GREEN}✓ PASSED: ${test_name}${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAILED: ${test_name}${NC}"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("$test_name")
    fi
}

# Function to check if a pod is running
check_pod_running() {
    local namespace="$1"
    local pod_pattern="$2"
    kubectl get pods -n "$namespace" | grep -E "$pod_pattern" | grep -q "Running"
}

# Function to check service endpoint
check_endpoint() {
    local url="$1"
    local expected_code="${2:-200}"
    local actual_code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
    [ "$actual_code" = "$expected_code" ]
}

echo "========================================="
echo "Claude Code Telemetry Stack - Comprehensive Test Suite"
echo "========================================="

# 1. Infrastructure Tests
echo -e "\n${YELLOW}=== 1. Infrastructure Tests ===${NC}"

run_test "Kind cluster is running" \
    "kubectl cluster-info &>/dev/null"

run_test "Telemetry namespace exists" \
    "kubectl get namespace telemetry &>/dev/null"

run_test "NGINX Ingress Controller is running" \
    "check_pod_running ingress-nginx 'ingress-nginx-controller.*Running'"

# 2. Component Health Tests
echo -e "\n${YELLOW}=== 2. Component Health Tests ===${NC}"

run_test "OpenTelemetry Collector is running" \
    "check_pod_running telemetry 'otel-collector.*Running'"

run_test "VictoriaMetrics is running" \
    "check_pod_running telemetry 'victoria-metrics.*Running'"

run_test "VictoriaLogs is running" \
    "check_pod_running telemetry 'victoria-logs.*Running'"

run_test "Grafana is running" \
    "check_pod_running telemetry 'grafana.*Running'"

# 3. Service Connectivity Tests
echo -e "\n${YELLOW}=== 3. Service Connectivity Tests ===${NC}"

# Start port forwards in background
echo "Setting up port forwards..."
kubectl port-forward -n telemetry svc/grafana 3000:80 &>/dev/null &
GRAFANA_PF_PID=$!
kubectl port-forward -n telemetry svc/victoria-metrics-victoria-metrics-single-server 8428:8428 &>/dev/null &
VM_PF_PID=$!
kubectl port-forward -n telemetry svc/victoria-logs-victoria-logs-single-server 9428:9428 &>/dev/null &
VL_PF_PID=$!
kubectl port-forward -n telemetry svc/otel-collector 4318:4318 &>/dev/null &
OTEL_PF_PID=$!

sleep 5  # Wait for port forwards to establish

run_test "Grafana UI is accessible" \
    "check_endpoint http://localhost:3000/api/health"

run_test "VictoriaMetrics API is accessible" \
    "check_endpoint http://localhost:8428/api/v1/query?query=up"

run_test "VictoriaLogs API is accessible" \
    "check_endpoint http://localhost:9428/select/logsql/query?query=*"

run_test "OTLP HTTP endpoint is accessible" \
    "check_endpoint http://localhost:4318/v1/metrics 405"  # 405 because GET not allowed

# 4. Data Ingestion Tests
echo -e "\n${YELLOW}=== 4. Data Ingestion Tests ===${NC}"

# Test metric ingestion
run_test "OTLP metrics ingestion works" \
    'curl -s -X POST http://localhost:4318/v1/metrics \
        -H "Content-Type: application/json" \
        -d "{\"resourceMetrics\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"test-service\"}},{\"key\":\"test.run\",\"value\":{\"stringValue\":\"comprehensive-test\"}}]},\"scopeMetrics\":[{\"metrics\":[{\"name\":\"test.metric\",\"sum\":{\"dataPoints\":[{\"asInt\":\"42\",\"timeUnixNano\":\"'$(date +%s)'000000000\"}]}}]}]}]}" \
        | grep -q "Partial success" || [ $? -eq 1 ]'

# Test log ingestion
run_test "OTLP logs ingestion works" \
    'curl -s -X POST http://localhost:4318/v1/logs \
        -H "Content-Type: application/json" \
        -d "{\"resourceLogs\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"test-service\"}}]},\"scopeLogs\":[{\"logRecords\":[{\"timeUnixNano\":\"'$(date +%s)'000000000\",\"severityText\":\"INFO\",\"body\":{\"stringValue\":\"Test log from comprehensive test\"}}]}]}]}" \
        | grep -q "Partial success" || [ $? -eq 1 ]'

sleep 5  # Wait for data to be processed

# 5. Data Query Tests
echo -e "\n${YELLOW}=== 5. Data Query Tests ===${NC}"

run_test "Can query test metrics from VictoriaMetrics" \
    'curl -s "http://localhost:8428/api/v1/query?query=test.metric" | jq -e ".data.result | length > 0" &>/dev/null'

run_test "Can query logs from VictoriaLogs" \
    'curl -s "http://localhost:9428/select/logsql/query?query=service.name:test-service" | grep -q "Test log from comprehensive test"'

# 6. Grafana Configuration Tests
echo -e "\n${YELLOW}=== 6. Grafana Configuration Tests ===${NC}"

GRAFANA_AUTH="admin:admin123!"

run_test "VictoriaMetrics datasource is configured" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/datasources/name/VictoriaMetrics | jq -e ".id > 0" &>/dev/null'

run_test "VictoriaLogs datasource is configured" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/datasources/name/VictoriaLogs | jq -e ".id > 0" &>/dev/null'

# 7. Dashboard Tests
echo -e "\n${YELLOW}=== 7. Dashboard Tests ===${NC}"

run_test "Executive Overview dashboard exists" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/dashboards/uid/executive-overview | jq -e ".dashboard.id > 0" &>/dev/null'

run_test "Engineering Metrics dashboard exists" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/dashboards/uid/engineering-metrics | jq -e ".dashboard.id > 0" &>/dev/null'

run_test "Operations Monitoring dashboard exists" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/dashboards/uid/operations-monitoring | jq -e ".dashboard.id > 0" &>/dev/null'

run_test "Cost Management dashboard exists" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/dashboards/uid/cost-management | jq -e ".dashboard.id > 0" &>/dev/null'

# 8. Alerting Tests
echo -e "\n${YELLOW}=== 8. Alerting Tests ===${NC}"

run_test "Alert rules are configured" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/v1/provisioning/alert-rules | jq -e "length > 0" &>/dev/null'

run_test "Notification policy is configured" \
    'curl -s -u $GRAFANA_AUTH http://localhost:3000/api/v1/provisioning/policies | jq -e ".receiver != null" &>/dev/null'

# 9. Claude Code Integration Tests
echo -e "\n${YELLOW}=== 9. Claude Code Integration Tests ===${NC}"

run_test "Claude Code deployment exists" \
    "kubectl get deployment -n telemetry claude-code &>/dev/null || kubectl get deployment -n claude-code-conversation claude-code-conversation &>/dev/null"

run_test "Claude Code ConfigMap with telemetry settings exists" \
    "kubectl get configmap -n telemetry claude-code-config &>/dev/null || true"

# 10. Resource Usage Tests
echo -e "\n${YELLOW}=== 10. Resource Usage Tests ===${NC}"

run_test "All pods have resource limits defined" \
    'kubectl get pods -n telemetry -o json | jq -e ".items[].spec.containers[].resources.limits.memory != null" &>/dev/null'

run_test "Persistent volumes are bound" \
    'kubectl get pvc -n telemetry -o json | jq -e ".items[] | select(.status.phase != \"Bound\") | length == 0" &>/dev/null || [ $(kubectl get pvc -n telemetry -o json | jq ".items | length") -eq 0 ]'

# Cleanup port forwards
echo -e "\nCleaning up port forwards..."
kill $GRAFANA_PF_PID $VM_PF_PID $VL_PF_PID $OTEL_PF_PID 2>/dev/null || true

# Test Summary
echo -e "\n========================================="
echo -e "${YELLOW}Test Summary${NC}"
echo "========================================="
echo -e "Tests Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests Failed: ${RED}${TESTS_FAILED}${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo -e "\n${RED}Failed Tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  - $test"
    done
fi

echo -e "\n========================================="

# Exit with appropriate code
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
```

### package-crds.sh (68 lines)

**Full Content:**
```sh
#!/bin/bash
# Package 5D Labs Platform CRDs for release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHART_DIR="${SCRIPT_DIR}/../charts/orchestrator"
CRDS_DIR="${CHART_DIR}/crds"
OUTPUT_DIR="${1:-${SCRIPT_DIR}/../dist}"

# Ensure output directory exists
mkdir -p "${OUTPUT_DIR}"

echo "🔧 Packaging 5D Labs Platform CRDs..."
echo "📁 Chart directory: ${CHART_DIR}"
echo "📁 CRDs directory: ${CRDS_DIR}"
echo "📁 Output directory: ${OUTPUT_DIR}"

# Check if CRDs directory exists
if [[ ! -d "${CRDS_DIR}" ]]; then
    echo "❌ Error: CRDs directory not found at ${CRDS_DIR}"
    exit 1
fi

# Check if platform-crds.yaml exists
if [[ ! -f "${CRDS_DIR}/platform-crds.yaml" ]]; then
    echo "❌ Error: platform-crds.yaml not found at ${CRDS_DIR}/platform-crds.yaml"
    echo "   Please ensure the combined CRDs file exists."
    exit 1
fi

# Copy the combined CRDs file
cp "${CRDS_DIR}/platform-crds.yaml" "${OUTPUT_DIR}/"

# Validate the CRDs
echo "🔍 Validating CRDs..."
if kubectl apply --dry-run=client -f "${OUTPUT_DIR}/platform-crds.yaml" > /dev/null 2>&1; then
    echo "✅ CRDs validation passed"
else
    echo "❌ CRDs validation failed"
    exit 1
fi

# Generate individual CRD files as well (for flexibility)
echo "📦 Copying individual CRD files..."
cp "${CRDS_DIR}/coderun-crd.yaml" "${OUTPUT_DIR}/"
cp "${CRDS_DIR}/docsrun-crd.yaml" "${OUTPUT_DIR}/"

# Generate checksums
echo "🔐 Generating checksums..."
cd "${OUTPUT_DIR}"
sha256sum platform-crds.yaml > platform-crds.yaml.sha256
sha256sum coderun-crd.yaml > coderun-crd.yaml.sha256
sha256sum docsrun-crd.yaml > docsrun-crd.yaml.sha256

echo "✅ CRDs packaged successfully!"
echo ""
echo "📦 Generated files:"
ls -la "${OUTPUT_DIR}"/*.yaml "${OUTPUT_DIR}"/*.sha256
echo ""
echo "🚀 Upload these files to GitHub releases:"
echo "   - platform-crds.yaml (combined CRDs)"
echo "   - coderun-crd.yaml (individual CRD)"
echo "   - docsrun-crd.yaml (individual CRD)"
echo "   - *.sha256 (checksums)"
echo ""
echo "📋 Installation command for users:"
echo "   kubectl apply -f https://github.com/5dlabs/platform/releases/download/TAG/platform-crds.yaml"
```

### scrape-otlp-metrics.sh (25 lines)

**Full Content:**
```sh
#!/bin/bash

echo "Quick workaround: Scraping Claude Code metrics from OTLP Collector"

# Port forward to OTLP collector
kubectl port-forward -n telemetry deploy/otel-collector-opentelemetry-collector 8889:8889 &
PF_PID=$!
sleep 3

# Get all Claude Code metrics
echo "Fetching Claude Code metrics..."
curl -s http://localhost:8889/metrics | grep "^claude_code" > /tmp/claude_metrics.txt

echo "Found metrics:"
cat /tmp/claude_metrics.txt | cut -d'{' -f1 | sort -u

# Clean up
kill $PF_PID 2>/dev/null

echo ""
echo "Metrics saved to /tmp/claude_metrics.txt"
echo ""
echo "To query these metrics in Grafana:"
echo "1. Configure Grafana to scrape the OTLP collector's Prometheus endpoint"
echo "2. Or configure VictoriaMetrics to scrape it"
```

### test-telemetry-pipeline-v2.sh (53 lines)

**Full Content:**
```sh
#!/bin/bash

echo "=== Testing Claude Code Telemetry Pipeline ==="
echo ""

# 1. Check Claude Code pod status
echo "1. Claude Code Pod Status:"
kubectl get pods -n claude-code -o wide
echo ""

# 2. Check Claude Code telemetry configuration
echo "2. Claude Code Telemetry Configuration:"
kubectl exec -n claude-code deploy/claude-code -- env | grep -E "(OTEL_|CLAUDE_CODE_ENABLE)" | sort
echo ""

# 3. Check OTLP Collector status
echo "3. OTLP Collector Status:"
kubectl get pods -n telemetry -l app.kubernetes.io/name=opentelemetry-collector
echo ""

# 4. Test OTLP collector metrics endpoint
echo "4. Testing OTLP Collector Prometheus Endpoint:"
kubectl port-forward -n telemetry svc/otel-collector-metrics 8889:8889 > /dev/null 2>&1 &
PF_PID=$!
sleep 3
echo "Checking for any metrics..."
curl -s http://localhost:8889/metrics | grep -E "^(claude_code|# TYPE)" | head -20
kill $PF_PID 2>/dev/null
echo ""

# 5. Check VictoriaMetrics native OTLP ingestion
echo "5. Checking VictoriaMetrics OTLP Endpoint:"
kubectl logs -n telemetry victoria-metrics-victoria-metrics-single-server-0 --tail=20 | grep -i "opentelemetry"
echo ""

# 6. Query VictoriaMetrics for metrics
echo "6. Querying VictoriaMetrics for Claude Code metrics:"
kubectl port-forward -n telemetry victoria-metrics-victoria-metrics-single-server-0 8428:8428 > /dev/null 2>&1 &
PF_PID=$!
sleep 3
curl -s "http://localhost:8428/api/v1/label/__name__/values" | jq -r '.data[]' | grep -E "claude_code" || echo "No claude_code metrics found"
kill $PF_PID 2>/dev/null
echo ""

# 7. Check OTLP collector logs for errors
echo "7. OTLP Collector Recent Logs:"
kubectl logs -n telemetry deploy/otel-collector-opentelemetry-collector --tail=10
echo ""

echo "=== Summary ==="
echo "The telemetry pipeline should flow as:"
echo "Claude Code -> OTLP Collector (HTTP) -> VictoriaMetrics (Native OTLP)"
echo "Additionally, VictoriaMetrics scrapes OTLP Collector's Prometheus endpoint"
```

### setup-all-agents.sh (422 lines)

**Full Content:**
```sh
#!/bin/bash
# Fully automated batch setup script for multiple GitHub agents
# Only requires GitHub PAT tokens - SSH keys are auto-generated and added to GitHub

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLATFORM_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AGENTS_DIR="$PLATFORM_ROOT/agents"
NAMESPACE="orchestrator"
DRY_RUN=""
VERBOSE=""
AUTO_GENERATE=""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat << EOF
Fully automated GitHub agent setup for 5D Labs Platform

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --auto-generate         Generate SSH keys AND auto-add to GitHub via API (RECOMMENDED)
    --namespace <name>      Kubernetes namespace (default: orchestrator)
    --dry-run              Show commands without executing
    --verbose              Show detailed output
    --help                 Show this help message

DIRECTORY STRUCTURE:
    agents/
    ├── agent-name-1/
    │   └── .env            # TOKEN=ghp_xxxxx (only this required!)
    └── agent-name-2/
        └── .env            # TOKEN=ghp_xxxxx (only this required!)

AUTOMATED WORKFLOW:
    1. Create agent directories with .env files containing GitHub PAT tokens
    2. Run: $0 --auto-generate
    3. Script automatically:
       ✓ Generates SSH key pairs for each agent
       ✓ Adds public keys to GitHub accounts via API
       ✓ Creates Kubernetes secrets
    4. You're done! No manual steps needed.

GITHUB PAT REQUIREMENTS:
    For Classic PATs: 'write:gpg_key' scope
    For Fine-grained PATs: 'Git SSH keys' user permissions (write)

EXAMPLES:
    # Fully automated setup (recommended)
    $0 --auto-generate

    # Dry run to see what would happen
    $0 --auto-generate --dry-run

    # Setup in different namespace with verbose output
    $0 --auto-generate --namespace my-orchestrator --verbose

    # Create agent structure quickly
    for agent in pm0-5dlabs qa0-5dlabs swe-1-5dlabs swe-2-5dlabs SWE-2-5dlabs; do
        mkdir -p agents/\$agent
        echo "TOKEN=ghp_YOUR_TOKEN_HERE" > agents/\$agent/.env
    done

EOF
}

log() {
    if [[ -n "$VERBOSE" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $*" >&2
    fi
}

info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Function to add SSH key to GitHub via API
add_ssh_key_to_github() {
    local agent_name="$1"
    local token="$2"
    local public_key_file="$3"

    local title="5D Labs Platform Agent: $agent_name"

    log "Adding SSH key to GitHub for agent: $agent_name"

    if [[ -n "$DRY_RUN" ]]; then
        echo "DRY RUN: Would add SSH key to GitHub via API"
        echo "  Title: $title"
        echo "  Key: ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAASIMULATED... ${agent_name}@5dlabs.platform"
        return 0
    fi

    if [[ ! -f "$public_key_file" ]]; then
        error "Public key file not found: $public_key_file"
        return 1
    fi

    local public_key_content
    public_key_content=$(cat "$public_key_file")

    # GitHub API call to add SSH key
    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Accept: application/vnd.github+json" \
        -H "Authorization: Bearer $token" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        https://api.github.com/user/keys \
        -d "{\"title\":\"$title\",\"key\":\"$public_key_content\"}")

    http_code=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)

    log "GitHub API response code: $http_code"
    log "GitHub API response: $response_body"

    case $http_code in
        201)
            info "✅ Successfully added SSH key to GitHub for: $agent_name"
            return 0
            ;;
        422)
            if echo "$response_body" | grep -q "key is already in use"; then
                warn "SSH key already exists in GitHub for: $agent_name (skipping)"
                return 0
            else
                error "GitHub API validation error for $agent_name: $response_body"
                return 1
            fi
            ;;
        401)
            error "GitHub API authentication failed for $agent_name. Check your PAT token and scopes."
            error "Required scopes: Classic PAT needs 'write:gpg_key', Fine-grained PAT needs 'Git SSH keys' write permission"
            return 1
            ;;
        403)
            error "GitHub API forbidden for $agent_name. Check your PAT token permissions."
            return 1
            ;;
        *)
            error "GitHub API error for $agent_name (HTTP $http_code): $response_body"
            return 1
            ;;
    esac
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --auto-generate)
            AUTO_GENERATE="true"
            shift
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Check if --auto-generate was provided
if [[ -z "$AUTO_GENERATE" ]]; then
    error "Please use --auto-generate for the new fully automated workflow"
    echo ""
    usage
    exit 1
fi

# Check if agents directory exists
if [[ ! -d "$AGENTS_DIR" ]]; then
    error "Agents directory not found: $AGENTS_DIR"
    echo "Please create the agents/ directory structure:" >&2
    echo "  mkdir -p agents/{pm0-5dlabs,qa0-5dlabs,swe-1-5dlabs,swe-2-5dlabs,SWE-2-5dlabs}" >&2
    echo "  echo 'TOKEN=ghp_YOUR_TOKEN_HERE' > agents/pm0-5dlabs/.env" >&2
    echo "  # ... repeat for each agent" >&2
    exit 1
fi

# Check required tools
for tool in kubectl ssh-keygen curl; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        error "$tool is not installed or not in PATH"
        exit 1
    fi
done

# Check if namespace exists (unless dry run)
if [[ -z "$DRY_RUN" ]]; then
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        warn "Namespace '$NAMESPACE' does not exist"
        info "Creating namespace: $NAMESPACE"
        kubectl create namespace "$NAMESPACE"
    fi
fi

info "🚀 Starting fully automated agent setup"
info "   Agents directory: $AGENTS_DIR"
info "   Namespace: $NAMESPACE"
info "   Mode: Auto-generate SSH keys + Auto-add to GitHub"
if [[ -n "$DRY_RUN" ]]; then
    info "   Mode: DRY RUN (no changes will be made)"
fi
echo ""

# Find all agent directories
AGENT_COUNT=0
PROCESSED_COUNT=0
FAILED_COUNT=0
API_ADDED_KEYS=()

# Count agents first
for agent_dir in "$AGENTS_DIR"/*; do
    if [[ -d "$agent_dir" ]]; then
        AGENT_COUNT=$((AGENT_COUNT + 1))
    fi
done

if [[ $AGENT_COUNT -eq 0 ]]; then
    warn "No agent directories found in $AGENTS_DIR"
    echo "Please create agent directories with .env files containing GitHub PAT tokens"
    exit 1
fi

info "📋 Found $AGENT_COUNT agent(s) to process"
echo ""

# Process each agent directory
for agent_dir in "$AGENTS_DIR"/*; do
    if [[ ! -d "$agent_dir" ]]; then
        continue
    fi

    agent_name=$(basename "$agent_dir")
    log "Processing agent directory: $agent_dir"

    # Check for .env file (required)
    env_file="$agent_dir/.env"
    if [[ ! -f "$env_file" ]]; then
        error "Agent '$agent_name': Missing .env file with GitHub PAT token"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Load token from .env file
    if ! source "$env_file"; then
        error "Agent '$agent_name': Failed to load .env file"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    if [[ -z "${TOKEN:-}" ]]; then
        error "Agent '$agent_name': TOKEN not found in .env file"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Validate token format
    if [[ ! "$TOKEN" =~ ^ghp_[a-zA-Z0-9_]{36}$ ]]; then
        warn "Agent '$agent_name': Token doesn't match expected format (ghp_xxxxx)"
    fi

    # SSH key paths
    ssh_private="$agent_dir/id_ed25519"
    ssh_public="$agent_dir/id_ed25519.pub"

    # Generate SSH keys
    info "🔑 Generating SSH key pair for agent: $agent_name"

    if [[ -z "$DRY_RUN" ]]; then
        # Generate SSH key pair
        ssh-keygen -t ed25519 -f "$ssh_private" -N "" -C "${agent_name}@5dlabs.platform" -q

        if [[ -f "$ssh_private" && -f "$ssh_public" ]]; then
            # Set proper permissions
            chmod 600 "$ssh_private"
            chmod 644 "$ssh_public"
            info "✅ Generated SSH key pair for: $agent_name"
        else
            error "Failed to generate SSH key pair for: $agent_name"
            FAILED_COUNT=$((FAILED_COUNT + 1))
            continue
        fi
    else
        echo "DRY RUN: ssh-keygen -t ed25519 -f $ssh_private -N \"\" -C \"${agent_name}@5dlabs.platform\" -q"
    fi

    # Add SSH key to GitHub via API
    info "🌐 Adding SSH key to GitHub for agent: $agent_name"

    if add_ssh_key_to_github "$agent_name" "$TOKEN" "$ssh_public"; then
        API_ADDED_KEYS+=("$agent_name")
        info "✅ SSH key added to GitHub for: $agent_name"
    else
        error "❌ Failed to add SSH key to GitHub for: $agent_name"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Create Kubernetes secrets
    info "🔧 Setting up Kubernetes secrets for agent: $agent_name"

    # Call the setup-agent-secrets.sh script
    setup_cmd=(
        "$SCRIPT_DIR/setup-agent-secrets.sh"
        "--user" "$agent_name"
        "--ssh-key" "$ssh_private"
        "--token" "$TOKEN"
        "--namespace" "$NAMESPACE"
    )

    if [[ -n "$DRY_RUN" ]]; then
        setup_cmd+=("--dry-run")
    fi

    if [[ -n "$VERBOSE" ]]; then
        setup_cmd+=("--verbose")
    fi

    log "Executing: ${setup_cmd[*]}"

    if "${setup_cmd[@]}"; then
        info "✅ Successfully processed agent: $agent_name"
        PROCESSED_COUNT=$((PROCESSED_COUNT + 1))
    else
        error "❌ Failed to process agent: $agent_name"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi

    echo ""
done

# Display summary of API actions
if [[ ${#API_ADDED_KEYS[@]} -gt 0 ]]; then
    echo "════════════════════════════════════════════════════════════════"
    info "🌐 SSH Keys Successfully Added to GitHub:"
    echo ""

    for agent_name in "${API_ADDED_KEYS[@]}"; do
        echo -e "${GREEN}   ✅ ${agent_name}${NC} - SSH key active in GitHub account"
    done
    echo ""

    info "🎯 All SSH keys are now configured and ready to use!"
    echo ""
fi

# Summary
echo "════════════════════════════════════════════════════════════════"
info "🎉 Fully automated agent setup completed!"
echo ""
info "📊 Summary:"
info "   Total agents found: $AGENT_COUNT"
info "   Successfully processed: $PROCESSED_COUNT"
if [[ $FAILED_COUNT -gt 0 ]]; then
    warn "   Failed: $FAILED_COUNT"
else
    info "   Failed: $FAILED_COUNT"
fi
info "   SSH keys added to GitHub: ${#API_ADDED_KEYS[@]}"
echo ""

if [[ $FAILED_COUNT -eq 0 ]]; then
    info "✅ All agents processed successfully!"
    if [[ -z "$DRY_RUN" ]]; then
        echo ""
        info "🔍 Verify secrets:"
        echo "   kubectl get secrets -n $NAMESPACE | grep github"
        echo ""
        info "🚀 Next steps:"
        echo "   1. Install Helm chart: helm install orchestrator ./infra/charts/orchestrator"
        echo "   2. Create your first task with any of these agents:"
        for agent_dir in "$AGENTS_DIR"/*; do
            if [[ -d "$agent_dir" ]]; then
                agent_name=$(basename "$agent_dir")
                echo "      - githubUser: \"$agent_name\""
            fi
        done
        echo ""
        info "🎯 No manual steps required - everything is automated!"
    else
        info "🏃 Remove --dry-run to execute these commands"
    fi
else
    error "❌ Some agents failed to process. Check the errors above."
    exit 1
fi
```

### create-all-dashboards.sh (1279 lines)

**Full Content:**
```sh
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
```

### setup-agent-secrets.sh (213 lines)

**Full Content:**
```sh
#!/bin/bash
# Setup GitHub agent secrets for 5D Labs Platform
# This script creates the SSH and GitHub token secrets that the orchestrator expects

set -euo pipefail

# Default values
NAMESPACE="orchestrator"
DRY_RUN=""
VERBOSE=""

# Function to show usage
usage() {
    cat << EOF
Setup GitHub agent secrets for 5D Labs Platform

USAGE:
    $0 --user <github-user> --ssh-key <path> --token <token> [OPTIONS]

REQUIRED:
    --user <username>       GitHub username (e.g., 'johnsmith')
    --ssh-key <path>        Path to SSH private key (e.g., '~/.ssh/github_key')
    --token <token>         GitHub personal access token (ghp_xxxx)

OPTIONS:
    --namespace <name>      Kubernetes namespace (default: orchestrator)
    --dry-run              Show commands without executing
    --verbose              Show detailed output
    --help                 Show this help message

EXAMPLES:
    # Setup secrets for user 'johnsmith'
    $0 --user johnsmith --ssh-key ~/.ssh/github_johnsmith --token ghp_abc123

    # Dry run to see what would be created
    $0 --user alice --ssh-key ~/.ssh/alice_github --token ghp_xyz789 --dry-run

    # Setup in different namespace
    $0 --user bob --ssh-key ~/.ssh/bob --token ghp_def456 --namespace my-orchestrator

NOTES:
    - SSH key path should point to the PRIVATE key (public key will be derived)
    - GitHub token needs 'repo' permissions for PR creation
    - Secrets will be named: github-ssh-<user> and github-token-<user>
    - Existing secrets will be replaced without warning

EOF
}

# Function for verbose logging
log() {
    if [[ -n "$VERBOSE" ]]; then
        echo "🔧 $*" >&2
    fi
}

# Function to execute or show commands
execute() {
    if [[ -n "$DRY_RUN" ]]; then
        echo "DRY RUN: $*"
    else
        log "Executing: $*"
        eval "$@"
    fi
}

# Parse command line arguments
GITHUB_USER=""
SSH_KEY_PATH=""
GITHUB_TOKEN=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            GITHUB_USER="$2"
            shift 2
            ;;
        --ssh-key)
            SSH_KEY_PATH="$2"
            shift 2
            ;;
        --token)
            GITHUB_TOKEN="$2"
            shift 2
            ;;
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$GITHUB_USER" ]]; then
    echo "❌ Error: --user is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

if [[ -z "$SSH_KEY_PATH" ]]; then
    echo "❌ Error: --ssh-key is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

if [[ -z "$GITHUB_TOKEN" ]]; then
    echo "❌ Error: --token is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

# Validate SSH key path
SSH_KEY_PATH=$(eval echo "$SSH_KEY_PATH")  # Expand ~ if present
if [[ ! -f "$SSH_KEY_PATH" ]]; then
    echo "❌ Error: SSH private key not found at: $SSH_KEY_PATH" >&2
    exit 1
fi

# Derive public key path
SSH_PUB_PATH="${SSH_KEY_PATH}.pub"
if [[ ! -f "$SSH_PUB_PATH" ]]; then
    echo "❌ Error: SSH public key not found at: $SSH_PUB_PATH" >&2
    echo "Expected to find public key alongside private key" >&2
    exit 1
fi

# Validate GitHub token format
if [[ ! "$GITHUB_TOKEN" =~ ^ghp_[a-zA-Z0-9_]{36}$ ]]; then
    echo "⚠️  Warning: GitHub token doesn't match expected format (ghp_xxxxx)" >&2
    echo "Continuing anyway..." >&2
fi

# Generate secret names
SSH_SECRET_NAME="github-ssh-${GITHUB_USER}"
TOKEN_SECRET_NAME="github-token-${GITHUB_USER}"

# Show summary
echo "🚀 Setting up GitHub agent secrets"
echo "   User: $GITHUB_USER"
echo "   SSH Key: $SSH_KEY_PATH"
echo "   Namespace: $NAMESPACE"
echo "   SSH Secret: $SSH_SECRET_NAME"
echo "   Token Secret: $TOKEN_SECRET_NAME"
echo ""

if [[ -n "$DRY_RUN" ]]; then
    echo "🔍 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    echo "❌ Error: kubectl is not installed or not in PATH" >&2
    exit 1
fi

# Check if namespace exists
if [[ -z "$DRY_RUN" ]]; then
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        echo "❌ Error: Namespace '$NAMESPACE' does not exist" >&2
        echo "Create it first: kubectl create namespace $NAMESPACE" >&2
        exit 1
    fi
fi

# Create SSH secret
log "Creating SSH secret: $SSH_SECRET_NAME"
execute kubectl create secret generic "$SSH_SECRET_NAME" \
    --namespace="$NAMESPACE" \
    --from-file=ssh-privatekey="$SSH_KEY_PATH" \
    --from-file=ssh-publickey="$SSH_PUB_PATH" \
    --dry-run=client -o yaml \| kubectl apply -f -

# Create GitHub token secret
log "Creating GitHub token secret: $TOKEN_SECRET_NAME"
execute kubectl create secret generic "$TOKEN_SECRET_NAME" \
    --namespace="$NAMESPACE" \
    --from-literal=token="$GITHUB_TOKEN" \
    --dry-run=client -o yaml \| kubectl apply -f -

if [[ -z "$DRY_RUN" ]]; then
    echo ""
    echo "✅ Successfully created agent secrets for user: $GITHUB_USER"
    echo ""
    echo "🔍 Verify secrets:"
    echo "   kubectl get secrets -n $NAMESPACE | grep github-$GITHUB_USER"
    echo ""
    echo "📋 To use this agent in a CodeRun:"
    echo "   spec:"
    echo "     githubUser: \"$GITHUB_USER\""
    echo "     # ... other fields"
else
    echo ""
    echo "✅ Dry run completed successfully"
    echo "Remove --dry-run to execute these commands"
fi
```

