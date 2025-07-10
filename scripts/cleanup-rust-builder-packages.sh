#!/bin/bash

# GitHub Container Registry Package Cleanup Script
# Cleans up old rust-builder package versions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ORG="5dlabs"
PACKAGE="agent-platform%2Frust-builder"
KEEP_COUNT=5  # Keep the 5 most recent versions
DRY_RUN=false
FORCE=false

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Clean up old rust-builder package versions from GitHub Container Registry

OPTIONS:
    -o, --org ORG           Organization name (default: $ORG)
    -k, --keep COUNT        Number of recent versions to keep (default: $KEEP_COUNT)
    --dry-run               Show what would be deleted without actually deleting
    -f, --force             Skip confirmation prompts
    -h, --help              Show this help message

EXAMPLES:
    $0 --dry-run            # Show what would be deleted
    $0 --keep 3             # Keep only the 3 most recent versions
    $0 --force              # Delete without confirmation

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--org)
            ORG="$2"
            shift 2
            ;;
        -k|--keep)
            KEEP_COUNT="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            usage
            exit 1
            ;;
    esac
done

echo -e "${BLUE}GitHub Container Registry Package Cleanup${NC}"
echo -e "${BLUE}===========================================${NC}"
echo -e "Organization: ${CYAN}$ORG${NC}"
echo -e "Package: ${CYAN}agent-platform/rust-builder${NC}"
echo -e "Keep count: ${CYAN}$KEEP_COUNT${NC}"
echo -e "Dry run: ${CYAN}$DRY_RUN${NC}"
echo ""

# Get all package versions sorted by creation date (newest first)
echo -e "${YELLOW}Fetching package versions...${NC}"
VERSIONS=$(gh api --paginate "/orgs/$ORG/packages/container/$PACKAGE/versions" \
    --jq 'sort_by(.created_at) | reverse | .[] | {id: .id, name: .name, tags: .metadata.container.tags, created_at: .created_at}')

if [ -z "$VERSIONS" ]; then
    echo -e "${RED}No package versions found or unable to fetch versions${NC}"
    exit 1
fi

# Convert to array and process
echo "$VERSIONS" | jq -c '.' | {
    versions_to_keep=()
    versions_to_delete=()
    count=0

    while IFS= read -r version; do
        id=$(echo "$version" | jq -r '.id')
        tags=$(echo "$version" | jq -r '.tags[]?' 2>/dev/null | tr '\n' ',' | sed 's/,$//')
        created_at=$(echo "$version" | jq -r '.created_at')
        name=$(echo "$version" | jq -r '.name' | cut -c1-12)

        # Format date for display
        date_display=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$created_at" +"%Y-%m-%d %H:%M" 2>/dev/null || echo "$created_at")

        count=$((count + 1))

        if [ $count -le $KEEP_COUNT ]; then
            versions_to_keep+=("$id")
            echo -e "✅ ${GREEN}Keep${NC}: Version $id [$name...] - Tags: ${tags:-"<none>"} - $date_display"
        else
            versions_to_delete+=("$id")
            echo -e "🗑️  ${RED}Delete${NC}: Version $id [$name...] - Tags: ${tags:-"<none>"} - $date_display"
        fi
    done

    echo ""
    echo -e "${YELLOW}Summary:${NC}"
    echo -e "  Versions to keep: ${GREEN}${#versions_to_keep[@]}${NC}"
    echo -e "  Versions to delete: ${RED}${#versions_to_delete[@]}${NC}"

    if [ ${#versions_to_delete[@]} -eq 0 ]; then
        echo -e "${GREEN}No versions to delete. All good!${NC}"
        exit 0
    fi

    if [ "$DRY_RUN" = true ]; then
        echo ""
        echo -e "${YELLOW}DRY RUN: No actual deletions performed${NC}"
        echo -e "Run without --dry-run to perform actual cleanup"
        exit 0
    fi

    # Confirmation
    if [ "$FORCE" != true ]; then
        echo ""
        echo -e "${YELLOW}Are you sure you want to delete ${#versions_to_delete[@]} package versions?${NC}"
        read -p "Type 'yes' to confirm: " -r
        if [[ ! $REPLY =~ ^yes$ ]]; then
            echo "Cancelled."
            exit 0
        fi
    fi

    # Delete versions
    echo ""
    echo -e "${YELLOW}Deleting package versions...${NC}"

    deleted_count=0
    failed_count=0

    for version_id in "${versions_to_delete[@]}"; do
        if gh api --method DELETE "/orgs/$ORG/packages/container/$PACKAGE/versions/$version_id" >/dev/null 2>&1; then
            echo -e "✓ ${GREEN}Deleted${NC}: Version $version_id"
            deleted_count=$((deleted_count + 1))
        else
            echo -e "✗ ${RED}Failed${NC}: Version $version_id"
            failed_count=$((failed_count + 1))
        fi
    done

    echo ""
    echo -e "${GREEN}Cleanup Summary:${NC}"
    echo -e "  Successfully deleted: ${GREEN}$deleted_count${NC}"
    echo -e "  Failed to delete: ${RED}$failed_count${NC}"
    echo -e "  Versions remaining: ${CYAN}$KEEP_COUNT${NC}"

    if [ $failed_count -eq 0 ]; then
        echo -e "${GREEN}✅ Package cleanup completed successfully!${NC}"
    else
        echo -e "${YELLOW}⚠️  Package cleanup completed with some failures${NC}"
    fi
}