#!/bin/bash

# Orchestrator Integration Test Runner
# Usage: ./run-integration-tests.sh [test-suite]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_color() {
    echo -e "${1}${2}${NC}"
}

print_header() {
    echo
    print_color $BLUE "=================================="
    print_color $BLUE "$1"
    print_color $BLUE "=================================="
    echo
}

print_success() {
    print_color $GREEN "✅ $1"
}

print_warning() {
    print_color $YELLOW "⚠️  $1"
}

print_error() {
    print_color $RED "❌ $1"
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    print_success "Rust toolchain found"
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        print_warning "kubectl not found. Kubernetes tests will be skipped."
        export SKIP_K8S_TESTS=1
        return
    fi
    print_success "kubectl found"
    
    # Check Kubernetes cluster access
    if ! kubectl cluster-info &> /dev/null; then
        print_warning "Kubernetes cluster not accessible. Kubernetes tests will be skipped."
        export SKIP_K8S_TESTS=1
        return
    fi
    print_success "Kubernetes cluster accessible"
    
    # Check/create test namespace
    if ! kubectl get namespace test-orchestrator &> /dev/null; then
        print_warning "Test namespace not found. Creating test-orchestrator namespace..."
        kubectl create namespace test-orchestrator
        print_success "Test namespace created"
    else
        print_success "Test namespace exists"
    fi
    
    # Check/create test secret
    if ! kubectl get secret claude-api-key -n test-orchestrator &> /dev/null; then
        print_warning "Test secret not found. Creating claude-api-key secret..."
        kubectl create secret generic claude-api-key \
            --from-literal=api-key=test-key-for-integration-tests \
            -n test-orchestrator
        print_success "Test secret created"
    else
        print_success "Test secret exists"
    fi
}

# Clean up test resources
cleanup_resources() {
    if [[ "${SKIP_K8S_TESTS}" == "1" ]]; then
        return
    fi
    
    print_header "Cleaning Up Test Resources"
    
    # Delete test resources but keep namespace and secret for next run
    kubectl delete configmaps,jobs -n test-orchestrator -l source=github --ignore-not-found=true
    print_success "Test resources cleaned up"
}

# Run specific test suite
run_test_suite() {
    local test_name="$1"
    local test_command="$2"
    
    print_header "Running Test Suite: $test_name"
    
    if cargo test "$test_command" -- --nocapture; then
        print_success "$test_name tests passed"
        return 0
    else
        print_error "$test_name tests failed"
        return 1
    fi
}

# Main test runner
main() {
    local test_suite="${1:-all}"
    local failed_tests=0
    
    print_header "Orchestrator Integration Test Runner"
    
    # Set up environment
    export RUST_LOG="${RUST_LOG:-info}"
    
    if [[ "${CI}" == "true" ]]; then
        print_success "Running in CI environment"
    else
        print_success "Running in local development environment"
    fi
    
    # Check prerequisites
    check_prerequisites
    
    # Clean up any existing test resources
    cleanup_resources
    
    case $test_suite in
        "all")
            print_header "Running All Integration Tests"
            if ! cargo test run_all_integration_tests -- --nocapture; then
                ((failed_tests++))
            fi
            ;;
            
        "smoke")
            run_test_suite "Smoke Test" "smoke_test" || ((failed_tests++))
            ;;
            
        "pipeline")
            run_test_suite "Webhook Pipeline" "test_webhook_pipeline_only -- --ignored" || ((failed_tests++))
            ;;
            
        "errors")
            run_test_suite "Error Scenarios" "test_error_scenarios_only -- --ignored" || ((failed_tests++))
            ;;
            
        "idempotency")
            run_test_suite "Idempotency" "test_idempotency_only -- --ignored" || ((failed_tests++))
            ;;
            
        "cleanup")
            run_test_suite "Resource Cleanup" "test_resource_cleanup_only -- --ignored" || ((failed_tests++))
            ;;
            
        "concurrent")
            run_test_suite "Concurrent Processing" "test_concurrent_processing_only -- --ignored" || ((failed_tests++))
            ;;
            
        "quick")
            print_header "Running Quick Test Suite"
            run_test_suite "Smoke Test" "smoke_test" || ((failed_tests++))
            run_test_suite "Basic Pipeline" "test_full_webhook_pipeline" || ((failed_tests++))
            run_test_suite "Basic Error Handling" "test_invalid_json_payload" || ((failed_tests++))
            ;;
            
        *)
            print_error "Unknown test suite: $test_suite"
            echo
            echo "Available test suites:"
            echo "  all         - Run all integration tests (default)"
            echo "  smoke       - Run basic smoke tests"
            echo "  pipeline    - Run webhook pipeline tests"
            echo "  errors      - Run error scenario tests"
            echo "  idempotency - Run idempotency tests"
            echo "  cleanup     - Run resource cleanup tests"
            echo "  concurrent  - Run concurrent processing tests"
            echo "  quick       - Run essential tests quickly"
            echo
            echo "Usage: ./run-integration-tests.sh [test-suite]"
            exit 1
            ;;
    esac
    
    # Final cleanup
    cleanup_resources
    
    # Report results
    print_header "Test Results"
    if [[ $failed_tests -eq 0 ]]; then
        print_success "All tests passed! 🎉"
        
        if [[ "${SKIP_K8S_TESTS}" != "1" ]]; then
            echo
            print_success "Kubernetes integration verified"
            print_success "Resources cleaned up successfully"
        fi
        
        exit 0
    else
        print_error "$failed_tests test suite(s) failed"
        exit 1
    fi
}

# Handle script interruption
trap 'print_warning "Test run interrupted. Cleaning up..."; cleanup_resources; exit 1' INT TERM

# Run main function
main "$@"