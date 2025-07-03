#!/bin/bash
# Real-time monitoring script for Claude Code execution

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

POD_NAME=$(kubectl get pods -n claude-code-dev -l "app.kubernetes.io/name=claude-code" -o jsonpath="{.items[0].metadata.name}")

echo -e "${GREEN}=== Claude Code Telemetry Monitoring ===${NC}"
echo -e "${BLUE}Pod:${NC} $POD_NAME"
echo -e "${BLUE}Namespace:${NC} claude-code-dev"
echo -e "${BLUE}Service:${NC} telemetry-refactoring"
echo -e "${BLUE}GitHub User:${NC} ${USER}"
echo ""

# Function to check pod status
check_pod_status() {
    STATUS=$(kubectl get pod -n claude-code-dev $POD_NAME -o jsonpath='{.status.phase}' 2>/dev/null || echo "Not Found")
    READY=$(kubectl get pod -n claude-code-dev $POD_NAME -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
    echo -e "${YELLOW}Pod Status:${NC} $STATUS (Ready: $READY)"
}

# Function to check telemetry connectivity
check_telemetry() {
    echo -e "\n${YELLOW}=== Telemetry Connectivity ===${NC}"
    kubectl exec -n claude-code-dev $POD_NAME -- nc -zv otel-collector-opentelemetry-collector.telemetry.svc.cluster.local 4317 2>/dev/null && \
        echo -e "${GREEN}✓ OTLP gRPC endpoint reachable${NC}" || \
        echo -e "${RED}✗ OTLP gRPC endpoint unreachable${NC}"
}

# Function to show recent metrics
check_recent_metrics() {
    echo -e "\n${YELLOW}=== Recent Metrics (VictoriaMetrics) ===${NC}"
    curl -s "http://localhost:8428/api/v1/query?query=claude_code_session_count" 2>/dev/null | \
        jq -r '.data.result[]? | "Session Count: \(.value[1])"' 2>/dev/null || \
        echo "No metrics data yet"
}

# Function to show cost accumulation
check_cost() {
    echo -e "\n${YELLOW}=== Cost Tracking ===${NC}"
    curl -s "http://localhost:8428/api/v1/query?query=sum(claude_code_cost_usage)" 2>/dev/null | \
        jq -r '.data.result[]? | "Total Cost: $\(.value[1])"' 2>/dev/null || \
        echo "No cost data yet"
}

# Function to check alerts
check_alerts() {
    echo -e "\n${YELLOW}=== Active Alerts ===${NC}"
    curl -s -u admin:admin123! "http://localhost:3000/api/alertmanager/grafana/api/v1/alerts" 2>/dev/null | \
        jq -r '.data[]? | "🔥 \(.labels.alertname): \(.annotations.summary)"' 2>/dev/null || \
        echo "No active alerts"
}

# Main monitoring loop
echo -e "${BLUE}Starting monitoring... (Ctrl+C to stop)${NC}"
echo -e "${BLUE}Grafana Dashboard: http://localhost:3000${NC}"
echo -e "${BLUE}VictoriaMetrics: http://localhost:8428${NC}"
echo ""

while true; do
    clear
    echo -e "${GREEN}=== Claude Code Telemetry Monitoring - $(date) ===${NC}"
    
    check_pod_status
    check_telemetry
    check_recent_metrics
    check_cost
    check_alerts
    
    echo -e "\n${YELLOW}=== Live Logs (last 10 lines) ===${NC}"
    kubectl logs -n claude-code-dev $POD_NAME --tail=10 2>/dev/null || echo "No logs yet"
    
    echo -e "\n${BLUE}Refreshing in 30 seconds... (Ctrl+C to stop)${NC}"
    sleep 30
done