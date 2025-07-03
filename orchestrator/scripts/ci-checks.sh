#!/bin/bash
#
# CI Checks Script
# This script mirrors the checks that will run in GitHub Actions CI
# Run this before pushing to ensure code will pass CI
#

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${YELLOW}[CI CHECK]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Change to orchestrator directory
cd "$(dirname "$0")/.."

print_status "Starting CI checks..."

# 1. Format check
print_status "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting check passed"
else
    print_error "Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi

# 2. Clippy linting
print_status "Running Clippy linter..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Clippy check passed"
else
    print_error "Clippy check failed. Fix the warnings above."
    exit 1
fi

# 3. Build check
print_status "Building project..."
if cargo build --all-features --all-targets; then
    print_success "Build check passed"
else
    print_error "Build failed"
    exit 1
fi

# 4. Test execution
print_status "Running tests..."
if cargo test --all-features --all-targets; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    exit 1
fi

# 5. Documentation build
print_status "Building documentation..."
if cargo doc --no-deps --all-features; then
    print_success "Documentation build passed"
else
    print_error "Documentation build failed"
    exit 1
fi

# 6. Check for uncommitted Cargo.lock changes
print_status "Checking Cargo.lock..."
if git diff --exit-code Cargo.lock > /dev/null 2>&1; then
    print_success "Cargo.lock is up to date"
else
    print_error "Cargo.lock has uncommitted changes. Commit the changes."
    exit 1
fi

# 7. Security audit (optional but recommended)
if command -v cargo-audit &> /dev/null; then
    print_status "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_error "Security vulnerabilities found. Run 'cargo audit' for details."
        # Don't exit on audit failures as they might be in dependencies
    fi
else
    print_status "Skipping security audit (cargo-audit not installed)"
    echo "  Install with: cargo install cargo-audit"
fi

# 8. Check for TODO/FIXME comments (optional)
print_status "Checking for TODO/FIXME comments..."
TODO_COUNT=$(grep -r "TODO\|FIXME" --include="*.rs" src/ 2>/dev/null | wc -l || echo "0")
if [ "$TODO_COUNT" -gt 0 ]; then
    echo "  Found $TODO_COUNT TODO/FIXME comments"
    grep -r "TODO\|FIXME" --include="*.rs" src/ | head -5
    if [ "$TODO_COUNT" -gt 5 ]; then
        echo "  ... and $((TODO_COUNT - 5)) more"
    fi
fi

# Final summary
echo ""
print_success "All CI checks passed! ✨"
echo ""
echo "Your code is ready to be pushed and should pass CI."
echo "Remember to:"
echo "  1. Commit your changes with a descriptive message"
echo "  2. Push to your feature branch"
echo "  3. Create a pull request"
echo ""
echo "If you need to fix formatting issues, run:"
echo "  cargo fmt --all"
echo ""
echo "If you need to see clippy suggestions in detail, run:"
echo "  cargo clippy --all-targets --all-features"