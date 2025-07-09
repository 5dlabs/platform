#!/bin/bash
# Script to sync dependency versions from orchestrator's Cargo.lock to the Dockerfile

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_LOCK="${SCRIPT_DIR}/../../../orchestrator/Cargo.lock"
DOCKERFILE="${SCRIPT_DIR}/Dockerfile"

# Function to get exact version from Cargo.lock
get_version() {
    local package_name="$1"
    # Get the latest version if multiple exist
    grep -A1 "^name = \"$package_name\"" "$CARGO_LOCK" | grep "^version" | tail -1 | cut -d'"' -f2
}

echo "Syncing dependency versions from Cargo.lock to Dockerfile..."
echo ""
echo "Checking version mismatches:"
echo ""

# Check key dependencies
echo "Dependency    | Dockerfile | Cargo.lock | Match?"
echo "--------------|------------|------------|-------"

# Function to check version
check_dep() {
    local dep="$1"
    local dockerfile_version=$(grep "^$dep =" "$DOCKERFILE" 2>/dev/null | grep -o '"=[^"]*"' | tr -d '"=' || echo "NOT FOUND")
    local cargo_version=$(get_version "$dep")
    
    if [ "$dockerfile_version" = "NOT FOUND" ]; then
        printf "%-13s | %-10s | %-10s | ❌\n" "$dep" "NOT FOUND" "$cargo_version"
    elif [ "$dockerfile_version" = "$cargo_version" ]; then
        printf "%-13s | %-10s | %-10s | ✅\n" "$dep" "$dockerfile_version" "$cargo_version"
    else
        printf "%-13s | %-10s | %-10s | ❌\n" "$dep" "$dockerfile_version" "$cargo_version"
    fi
}

# Check all dependencies
for dep in tokio axum tower tower-http hyper serde serde_json anyhow thiserror tracing tracing-subscriber reqwest uuid chrono kube k8s-openapi handlebars opentelemetry opentelemetry_sdk opentelemetry-otlp tracing-opentelemetry; do
    check_dep "$dep"
done

echo ""
echo "Generating updated Cargo.toml section..."

# Generate the updated section
cat > /tmp/updated_cargo_toml.txt << 'EOF'
# Create Cargo.toml with EXACT versions from orchestrator Cargo.lock
RUN cat > Cargo.toml << 'EOF'
[package]
name = "warmup"
version = "0.1.0"
edition = "2021"

[dependencies]
# EXACT versions from orchestrator/Cargo.lock to ensure cache hits
EOF

# Add each dependency with correct version
echo "tokio = { version = \"=$(get_version tokio)\", features = [\"full\"] }" >> /tmp/updated_cargo_toml.txt
echo "axum = \"=$(get_version axum)\"" >> /tmp/updated_cargo_toml.txt
echo "tower = \"=$(get_version tower)\"" >> /tmp/updated_cargo_toml.txt
echo "tower-http = \"=$(get_version tower-http)\"" >> /tmp/updated_cargo_toml.txt
echo "hyper = \"=$(get_version hyper)\"" >> /tmp/updated_cargo_toml.txt
echo "serde = { version = \"=$(get_version serde)\", features = [\"derive\"] }" >> /tmp/updated_cargo_toml.txt
echo "serde_json = \"=$(get_version serde_json)\"" >> /tmp/updated_cargo_toml.txt
echo "anyhow = \"=$(get_version anyhow)\"" >> /tmp/updated_cargo_toml.txt
echo "thiserror = \"=$(get_version thiserror)\"" >> /tmp/updated_cargo_toml.txt
echo "tracing = \"=$(get_version tracing)\"" >> /tmp/updated_cargo_toml.txt
echo "tracing-subscriber = \"=$(get_version tracing-subscriber)\"" >> /tmp/updated_cargo_toml.txt
echo "reqwest = { version = \"=$(get_version reqwest)\", features = [\"json\"] }" >> /tmp/updated_cargo_toml.txt
echo "uuid = { version = \"=$(get_version uuid)\", features = [\"v4\"] }" >> /tmp/updated_cargo_toml.txt
echo "chrono = { version = \"=$(get_version chrono)\", features = [\"serde\"] }" >> /tmp/updated_cargo_toml.txt
echo "kube = \"=$(get_version kube)\"" >> /tmp/updated_cargo_toml.txt
echo "k8s-openapi = \"=$(get_version k8s-openapi)\"" >> /tmp/updated_cargo_toml.txt
echo "handlebars = \"=$(get_version handlebars)\"" >> /tmp/updated_cargo_toml.txt
echo "opentelemetry = \"=$(get_version opentelemetry)\"" >> /tmp/updated_cargo_toml.txt
echo "opentelemetry_sdk = \"=$(get_version opentelemetry_sdk)\"" >> /tmp/updated_cargo_toml.txt
echo "opentelemetry-otlp = \"=$(get_version opentelemetry-otlp)\"" >> /tmp/updated_cargo_toml.txt
echo "tracing-opentelemetry = \"=$(get_version tracing-opentelemetry)\"" >> /tmp/updated_cargo_toml.txt
echo "EOF" >> /tmp/updated_cargo_toml.txt

echo ""
echo "Updated Cargo.toml section saved to: /tmp/updated_cargo_toml.txt"
echo ""
echo "To apply the changes:"
echo "1. Review the updated section: cat /tmp/updated_cargo_toml.txt"
echo "2. Edit the Dockerfile and replace lines 111-141 with the content from /tmp/updated_cargo_toml.txt"