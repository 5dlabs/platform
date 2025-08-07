#!/bin/bash
# Local testing script for ArgoCD applications
# Run this before committing changes to GitOps files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GITOPS_DIR="$PROJECT_ROOT/infra/gitops"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Testing ArgoCD Applications Locally"
echo "======================================"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        return 1
    fi
}

# Track overall test status
TESTS_FAILED=0

# 1. YAML Syntax Check
echo ""
echo "1️⃣  Checking YAML syntax..."
if command_exists yamllint; then
    if [ -f "$PROJECT_ROOT/.yamllint.yaml" ]; then
        yamllint -c "$PROJECT_ROOT/.yamllint.yaml" "$GITOPS_DIR" 2>/dev/null
        print_result $? "YAML syntax validation" || TESTS_FAILED=$((TESTS_FAILED + 1))
    else
        echo -e "${YELLOW}⚠️  .yamllint.yaml not found, using default config${NC}"
        yamllint "$GITOPS_DIR" 2>/dev/null
        print_result $? "YAML syntax validation" || TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  yamllint not installed. Install with: pip install yamllint${NC}"
fi

# 2. Check for required labels
echo ""
echo "2️⃣  Checking required labels..."
LABEL_ERRORS=0
for file in "$GITOPS_DIR"/applications/*.yaml; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # Check for required labels using yq or grep
        if command_exists yq; then
            labels=$(yq eval '.metadata.labels' "$file")
            if ! echo "$labels" | grep -q "app.kubernetes.io/name"; then
                echo -e "${RED}   Missing label 'app.kubernetes.io/name' in $filename${NC}"
                LABEL_ERRORS=$((LABEL_ERRORS + 1))
            fi
            if ! echo "$labels" | grep -q "app.kubernetes.io/part-of"; then
                echo -e "${RED}   Missing label 'app.kubernetes.io/part-of' in $filename${NC}"
                LABEL_ERRORS=$((LABEL_ERRORS + 1))
            fi
        else
            # Fallback to grep if yq not available
            if ! grep -q "app.kubernetes.io/name:" "$file"; then
                echo -e "${YELLOW}   Warning: Might be missing 'app.kubernetes.io/name' in $filename${NC}"
            fi
        fi
    fi
done
print_result $LABEL_ERRORS "Required labels check" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 3. Validate ArgoCD Application structure
echo ""
echo "3️⃣  Validating ArgoCD Application structure..."
STRUCTURE_ERRORS=0
for file in "$GITOPS_DIR"/applications/*.yaml; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # Check for required fields
        if ! grep -q "kind: Application" "$file"; then
            echo -e "${RED}   Not an Application resource: $filename${NC}"
            STRUCTURE_ERRORS=$((STRUCTURE_ERRORS + 1))
        fi
        
        if ! grep -q "spec:" "$file"; then
            echo -e "${RED}   Missing spec section: $filename${NC}"
            STRUCTURE_ERRORS=$((STRUCTURE_ERRORS + 1))
        fi
        
        if ! grep -q "destination:" "$file"; then
            echo -e "${RED}   Missing destination: $filename${NC}"
            STRUCTURE_ERRORS=$((STRUCTURE_ERRORS + 1))
        fi
        
        if ! grep -q "source:" "$file"; then
            echo -e "${RED}   Missing source: $filename${NC}"
            STRUCTURE_ERRORS=$((STRUCTURE_ERRORS + 1))
        fi
    fi
done
print_result $STRUCTURE_ERRORS "Application structure validation" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 4. Check for duplicate application names
echo ""
echo "4️⃣  Checking for duplicate application names..."
DUPLICATES=0
declare -A app_names
for file in "$GITOPS_DIR"/applications/*.yaml; do
    if [ -f "$file" ]; then
        if command_exists yq; then
            name=$(yq eval '.metadata.name' "$file")
            if [ "${app_names[$name]}" ]; then
                echo -e "${RED}   Duplicate application name: $name${NC}"
                DUPLICATES=$((DUPLICATES + 1))
            fi
            app_names[$name]=1
        fi
    fi
done
print_result $DUPLICATES "Duplicate names check" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 5. Validate Helm values if present
echo ""
echo "5️⃣  Checking Helm configurations..."
HELM_ERRORS=0
for file in "$GITOPS_DIR"/applications/*.yaml; do
    if [ -f "$file" ] && grep -q "helm:" "$file"; then
        filename=$(basename "$file")
        
        # Basic check for Helm chart specification
        if ! grep -q "chart:" "$file" && ! grep -q "path:" "$file"; then
            echo -e "${RED}   Helm source missing chart or path: $filename${NC}"
            HELM_ERRORS=$((HELM_ERRORS + 1))
        fi
    fi
done
print_result $HELM_ERRORS "Helm configuration check" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 6. Check database resources
echo ""
echo "6️⃣  Checking database resources..."
DB_ERRORS=0
for file in "$GITOPS_DIR"/databases/*.yaml; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # Skip README and examples
        if [[ "$filename" == "README.md" ]] || [[ "$file" == *"examples"* ]]; then
            continue
        fi
        
        # Check for kind specification
        if ! grep -q "kind:" "$file"; then
            echo -e "${RED}   Missing kind specification: $filename${NC}"
            DB_ERRORS=$((DB_ERRORS + 1))
        fi
    fi
done
print_result $DB_ERRORS "Database resources check" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 7. Security checks
echo ""
echo "7️⃣  Running security checks..."
SECURITY_ERRORS=0

# Check for hardcoded passwords
if grep -r "password:" "$GITOPS_DIR" --include="*.yaml" | grep -v "secretKeyRef" | grep -v "CHANGE_THIS" | grep -v "PASSWORD" > /dev/null; then
    echo -e "${RED}   Found potential hardcoded passwords${NC}"
    SECURITY_ERRORS=$((SECURITY_ERRORS + 1))
fi

# Check for missing resource limits
for file in "$GITOPS_DIR"/databases/*.yaml; do
    if [ -f "$file" ] && [[ ! "$file" == *"examples"* ]]; then
        if ! grep -q "resources:" "$file"; then
            echo -e "${YELLOW}   Warning: No resource limits in $(basename "$file")${NC}"
        fi
    fi
done

print_result $SECURITY_ERRORS "Security checks" || TESTS_FAILED=$((TESTS_FAILED + 1))

# 8. Optional: Kubernetes validation with kubeconform
echo ""
echo "8️⃣  Kubernetes manifest validation..."
if command_exists kubeconform; then
    # Only validate standard Kubernetes resources, skip CRDs
    KUBE_ERRORS=0
    for file in "$GITOPS_DIR"/databases/*.yaml; do
        if [ -f "$file" ] && [[ ! "$file" == *"examples"* ]] && [[ ! "$file" == *"README"* ]]; then
            # Check if it's a standard K8s resource
            if command_exists yq; then
                KIND=$(yq eval '.kind' "$file" 2>/dev/null || echo "unknown")
                case "$KIND" in
                    Service|ConfigMap|Secret|Deployment|StatefulSet|DaemonSet|Job|CronJob|Ingress|PersistentVolumeClaim)
                        kubeconform -summary "$file" 2>/dev/null || KUBE_ERRORS=$((KUBE_ERRORS + 1))
                        ;;
                esac
            fi
        fi
    done
    print_result $KUBE_ERRORS "Kubernetes manifest validation" || TESTS_FAILED=$((TESTS_FAILED + 1))
else
    echo -e "${YELLOW}⚠️  kubeconform not installed. Install from: https://github.com/yannh/kubeconform${NC}"
fi

# Final summary
echo ""
echo "======================================"
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ $TESTS_FAILED test(s) failed${NC}"
    echo ""
    echo "Please fix the issues before committing."
    exit 1
fi
