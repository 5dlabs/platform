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