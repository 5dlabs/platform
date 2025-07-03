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