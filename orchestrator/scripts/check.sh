#!/bin/bash
set -e

echo "🧪 Running tests..."
cargo test --verbose

echo "📋 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🎨 Checking formatting..."
cargo fmt --all -- --check

echo "✅ All checks passed!"