#!/bin/bash

# Test webhook script for local development
# Usage: ./test-webhook.sh [port]

PORT=${1:-8080}
WEBHOOK_URL="http://localhost:${PORT}/webhook/github"

echo "Testing webhook endpoint at ${WEBHOOK_URL}"

# Check if server is running
if ! curl -s -o /dev/null http://localhost:${PORT}/health; then
    echo "Error: Server not running on port ${PORT}"
    echo "Start the server with: cargo run"
    exit 1
fi

echo "Server is healthy, sending test webhook..."

# Send the test webhook payload
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${WEBHOOK_URL}" \
    -H "Content-Type: application/json" \
    -H "X-GitHub-Event: issues" \
    -H "X-GitHub-Delivery: test-delivery-12345" \
    -d @test-data/github-issue-opened.json)

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "Response Code: ${HTTP_CODE}"
echo "Response Body: ${BODY}"

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "202" ]; then
    echo "✓ Webhook processed successfully"
else
    echo "✗ Webhook processing failed"
    exit 1
fi