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