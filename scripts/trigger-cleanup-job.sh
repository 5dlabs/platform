#!/bin/bash
# Trigger the cleanup CronJob manually

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Triggering cleanup job manually...${NC}"

# Create a job from the CronJob
kubectl create job --from=cronjob/orchestrator-cleanup manual-cleanup-$(date +%s) -n orchestrator

echo -e "${GREEN}✓ Cleanup job triggered${NC}"
echo ""
echo "To watch the job progress:"
echo "  kubectl get jobs -n orchestrator -w"
echo ""
echo "To see the cleanup logs:"
echo "  kubectl logs -n orchestrator -l job-name=manual-cleanup-* -f"