#!/bin/bash
# Manual cleanup script for orchestrator resources

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="orchestrator"

echo -e "${BLUE}Orchestrator Cleanup Script${NC}"
echo "================================="
echo ""

# Parse command line arguments
FORCE=false
DRY_RUN=false
CLEANUP_ALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --all)
            CLEANUP_ALL=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --force     Skip confirmation prompts"
            echo "  --dry-run   Show what would be deleted without actually deleting"
            echo "  --all       Clean up all resources (including running jobs)"
            echo "  --help      Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No resources will be deleted${NC}"
    echo ""
fi

# Function to delete resource with dry-run support
delete_resource() {
    local resource_type=$1
    local resource_name=$2
    local extra_args=${3:-}
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would delete $resource_type: $resource_name${NC}"
    else
        kubectl delete $resource_type -n $NAMESPACE "$resource_name" $extra_args
    fi
}

# Count resources
echo -e "${BLUE}Current Resource Status:${NC}"
TOTAL_PODS=$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l)
FAILED_PODS=$(kubectl get pods -n $NAMESPACE --field-selector=status.phase=Failed --no-headers 2>/dev/null | wc -l)
ERROR_PODS=$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | grep Error | wc -l)
TOTAL_JOBS=$(kubectl get jobs -n $NAMESPACE --no-headers 2>/dev/null | wc -l)
COMPLETED_JOBS=$(kubectl get jobs -n $NAMESPACE -o json 2>/dev/null | jq '.items[] | select(.status.succeeded > 0)' | jq -s 'length')
TOTAL_CONFIGMAPS=$(kubectl get configmaps -n $NAMESPACE --no-headers 2>/dev/null | grep -E '^[a-zA-Z0-9-]+-[0-9]+-v[0-9]+-files' | wc -l)

echo "- Total Pods: $TOTAL_PODS (Failed: $FAILED_PODS, Error: $ERROR_PODS)"
echo "- Total Jobs: $TOTAL_JOBS (Completed: $COMPLETED_JOBS)"
echo "- Task ConfigMaps: $TOTAL_CONFIGMAPS"
echo ""

# Check what will be cleaned
TO_DELETE_FAILED_PODS=$(($FAILED_PODS + $ERROR_PODS))
TO_DELETE_COMPLETED_JOBS=$COMPLETED_JOBS
TO_DELETE_ALL_JOBS=0

if [ "$CLEANUP_ALL" = true ]; then
    TO_DELETE_ALL_JOBS=$TOTAL_JOBS
fi

if [ $TO_DELETE_FAILED_PODS -eq 0 ] && [ $TO_DELETE_COMPLETED_JOBS -eq 0 ] && [ $TO_DELETE_ALL_JOBS -eq 0 ]; then
    echo -e "${GREEN}Nothing to clean up!${NC}"
    exit 0
fi

echo -e "${YELLOW}Resources to be cleaned:${NC}"
if [ $TO_DELETE_FAILED_PODS -gt 0 ]; then
    echo "- Failed/Error Pods: $TO_DELETE_FAILED_PODS"
fi
if [ $TO_DELETE_COMPLETED_JOBS -gt 0 ]; then
    echo "- Completed Jobs: $TO_DELETE_COMPLETED_JOBS"
fi
if [ "$CLEANUP_ALL" = true ] && [ $TO_DELETE_ALL_JOBS -gt 0 ]; then
    echo -e "${RED}- ALL Jobs: $TO_DELETE_ALL_JOBS${NC}"
fi
echo ""

# Confirmation
if [ "$FORCE" != true ] && [ "$DRY_RUN" != true ]; then
    echo -n "Do you want to proceed with cleanup? (y/N): "
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Cleanup cancelled."
        exit 0
    fi
fi

echo ""

# Clean up failed pods
if [ $TO_DELETE_FAILED_PODS -gt 0 ]; then
    echo -e "${BLUE}Cleaning up failed/error pods...${NC}"
    kubectl get pods -n $NAMESPACE --no-headers | grep -E 'Failed|Error' | awk '{print $1}' | while read -r pod; do
        delete_resource "pod" "$pod" "--grace-period=0 --force"
    done
    echo ""
fi

# Clean up jobs
if [ "$CLEANUP_ALL" = true ] && [ $TO_DELETE_ALL_JOBS -gt 0 ]; then
    echo -e "${BLUE}Cleaning up ALL jobs...${NC}"
    kubectl get jobs -n $NAMESPACE --no-headers | awk '{print $1}' | while read -r job; do
        delete_resource "job" "$job" "--cascade=background"
    done
elif [ $TO_DELETE_COMPLETED_JOBS -gt 0 ]; then
    echo -e "${BLUE}Cleaning up completed jobs...${NC}"
    kubectl get jobs -n $NAMESPACE -o json | jq -r '.items[] | select(.status.succeeded > 0) | .metadata.name' | while read -r job; do
        delete_resource "job" "$job" "--cascade=background"
    done
fi

echo ""

# Clean up orphaned ConfigMaps
echo -e "${BLUE}Checking for orphaned ConfigMaps...${NC}"
ORPHANED_COUNT=0
kubectl get configmaps -n $NAMESPACE --no-headers | grep -E '^[a-zA-Z0-9-]+-[0-9]+-v[0-9]+-files' | awk '{print $1}' | while read -r cm; do
    # Extract job name from ConfigMap name
    job_base=$(echo "$cm" | sed 's/-files$//')
    
    # Check if corresponding job exists
    if ! kubectl get job -n $NAMESPACE "$job_base" &>/dev/null; then
        delete_resource "configmap" "$cm"
        ((ORPHANED_COUNT++))
    fi
done

if [ $ORPHANED_COUNT -eq 0 ]; then
    echo "No orphaned ConfigMaps found."
fi

echo ""

if [ "$DRY_RUN" != true ]; then
    # Show final status
    echo -e "${GREEN}Cleanup completed!${NC}"
    echo ""
    echo -e "${BLUE}Final Resource Status:${NC}"
    echo "- Pods: $(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l)"
    echo "- Jobs: $(kubectl get jobs -n $NAMESPACE --no-headers 2>/dev/null | wc -l)"
    echo "- ConfigMaps: $(kubectl get configmaps -n $NAMESPACE --no-headers 2>/dev/null | grep -E '^[a-zA-Z0-9-]+-[0-9]+-v[0-9]+-files' | wc -l)"
fi