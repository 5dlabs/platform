#!/bin/bash

# Test script for Claude continue functionality
# This script tests whether Claude can remember context from previous conversations

set -e  # Exit on any error

echo "[INFO] Starting Claude continue functionality test..."

# Check if claude command is available
if ! command -v claude > /dev/null 2>&1; then
    echo "[ERROR] Claude CLI not found. Please install it first."
    exit 1
fi

# Create a test directory
TEST_DIR="claude-continue-test"
echo "[INFO] Creating test directory: $TEST_DIR"

if [ -d "$TEST_DIR" ]; then
    echo "[WARNING] Test directory already exists. Removing it..."
    rm -rf "$TEST_DIR"
fi

mkdir "$TEST_DIR" && cd "$TEST_DIR"
echo "[SUCCESS] Created and entered test directory"

# First conversation - establish context
echo "[INFO] Starting first conversation with Claude..."
echo "Hello Claude, remember this: my favorite color is blue" | claude --model sonnet

echo "[SUCCESS] First conversation completed"

# Wait a moment to ensure conversation is saved
echo "[INFO] Waiting 2 seconds for conversation to be saved..."
sleep 2

# Test continue functionality - should remember the previous conversation
echo "[INFO] Testing continue functionality..."
echo "What was my favorite color that I just told you?" | claude --continue --model sonnet

echo "[SUCCESS] Continue test completed"

# Clean up
cd ..
echo "[INFO] Cleaning up test directory..."
rm -rf "$TEST_DIR"
echo "[SUCCESS] Test directory cleaned up"

echo "[SUCCESS] Claude continue functionality test completed successfully!"