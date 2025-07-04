#!/bin/bash

# Test script for Claude continue functionality
# This script tests whether Claude can remember context from previous conversations

set -e  # Exit on any error

# Colors for output formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if claude command is available
if ! command -v claude &> /dev/null; then
    print_error "Claude CLI not found. Please install it first."
    exit 1
fi

print_status "Starting Claude continue functionality test..."

# Create a test directory
TEST_DIR="claude-continue-test"
print_status "Creating test directory: $TEST_DIR"

if [ -d "$TEST_DIR" ]; then
    print_warning "Test directory already exists. Removing it..."
    rm -rf "$TEST_DIR"
fi

mkdir "$TEST_DIR" && cd "$TEST_DIR"
print_success "Created and entered test directory"

# First conversation - establish context
print_status "Starting first conversation with Claude..."
echo "Hello Claude, remember this: my favorite color is blue" | claude --model sonnet

print_success "First conversation completed"

# Wait a moment to ensure conversation is saved
print_status "Waiting 2 seconds for conversation to be saved..."
sleep 2

# Test continue functionality - should remember the previous conversation
print_status "Testing continue functionality..."
echo "What was my favorite color that I just told you?" | claude --continue --model sonnet

print_success "Continue test completed"

# Clean up
cd ..
print_status "Cleaning up test directory..."
rm -rf "$TEST_DIR"
print_success "Test directory cleaned up"

print_success "Claude continue functionality test completed successfully!"