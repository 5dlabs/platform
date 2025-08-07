#!/bin/bash
# Script to fix common YAML linting issues

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔧 Fixing YAML lint issues..."
echo "=============================="

# Function to fix a YAML file
fix_yaml_file() {
    local file="$1"
    local modified=false
    
    # Create a temporary file
    local temp_file="${file}.tmp"
    
    # Remove trailing spaces from each line
    if sed 's/[[:space:]]*$//' "$file" > "$temp_file"; then
        if ! diff -q "$file" "$temp_file" > /dev/null 2>&1; then
            mv "$temp_file" "$file"
            echo -e "${GREEN}✅ Fixed trailing spaces: $file${NC}"
            modified=true
        else
            rm -f "$temp_file"
        fi
    fi
    
    # Ensure file ends with a newline
    if [ -s "$file" ] && [ "$(tail -c 1 "$file" | wc -l)" -eq 0 ]; then
        echo "" >> "$file"
        echo -e "${GREEN}✅ Added newline at end: $file${NC}"
        modified=true
    fi
    
    # Fix document start (add --- if missing)
    if ! head -n 1 "$file" | grep -q "^---$"; then
        # Only add if it's a YAML file that should have document start
        if grep -q "^apiVersion:\|^kind:" "$file"; then
            sed -i.bak '1s/^/---\n/' "$file" && rm -f "${file}.bak"
            echo -e "${GREEN}✅ Added document start: $file${NC}"
            modified=true
        fi
    fi
    
    return 0
}

# Fix all YAML files in gitops directory
echo ""
echo "📁 Processing infra/gitops directory..."

# Find all YAML files
find infra/gitops -name "*.yaml" -o -name "*.yml" | while read -r file; do
    # Skip README files
    if [[ "$file" == *"README"* ]]; then
        continue
    fi
    
    fix_yaml_file "$file"
done

# Fix the specific indentation issue in agent-docs-postgres.yaml
if [ -f "infra/gitops/databases/agent-docs-postgres.yaml" ]; then
    echo ""
    echo "🔧 Fixing specific indentation issue in agent-docs-postgres.yaml..."
    
    # Fix line 163 indentation (should be 6 spaces, not 4)
    sed -i.bak '163s/^    /      /' infra/gitops/databases/agent-docs-postgres.yaml 2>/dev/null || true
    rm -f infra/gitops/databases/agent-docs-postgres.yaml.bak
fi

# Run yamllint to check if issues are fixed
echo ""
echo "🔍 Checking remaining issues..."
echo "=============================="

REMAINING_ISSUES=0

# Check applications
if yamllint -c .yamllint.yaml infra/gitops/applications/*.yaml 2>/dev/null; then
    echo -e "${GREEN}✅ Applications: No issues${NC}"
else
    echo -e "${YELLOW}⚠️  Applications: Some issues remain${NC}"
    REMAINING_ISSUES=$((REMAINING_ISSUES + 1))
fi

# Check databases
if yamllint -c .yamllint.yaml infra/gitops/databases/*.yaml 2>/dev/null | grep -v "examples/"; then
    echo -e "${GREEN}✅ Databases: No issues${NC}"
else
    echo -e "${YELLOW}⚠️  Databases: Some issues remain${NC}"
    REMAINING_ISSUES=$((REMAINING_ISSUES + 1))
fi

echo ""
echo "=============================="
if [ $REMAINING_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✅ All YAML lint issues fixed!${NC}"
    echo ""
    echo "You can now commit these changes:"
    echo "  git add infra/gitops/"
    echo "  git commit -m 'fix: resolve YAML linting issues'"
else
    echo -e "${YELLOW}⚠️  Some issues may require manual review${NC}"
    echo ""
    echo "Run this to see remaining issues:"
    echo "  yamllint -c .yamllint.yaml infra/gitops/"
fi

exit 0
