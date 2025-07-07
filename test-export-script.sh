#!/bin/bash
# Test if we see HTML encoding locally

# This should have equals sign
HOOK_INPUT=$(cat)

# This should have quotes
echo "Testing quotes"

# This should have single quotes  
echo 'Testing single quotes'

# Test comparison
if [ "$HOOK_INPUT" == "test" ]; then
  echo "Comparison works"
fi