#!/bin/bash
# Script to sync dependency versions from orchestrator's Cargo.lock to the Dockerfile

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_LOCK="${SCRIPT_DIR}/../../../orchestrator/Cargo.lock"
DOCKERFILE="${SCRIPT_DIR}/Dockerfile"

# Function to get exact version from Cargo.lock
get_version() {
    local package_name="$1"
    # Handle packages that might have multiple versions by getting the latest
    grep -A1 "^name = \"$package_name\"" "$CARGO_LOCK" | grep "^version" | tail -1 | cut -d'"' -f2
}

echo "Syncing dependency versions from Cargo.lock to Dockerfile..."

# Create the complete updated Cargo.toml content
cat > /tmp/dockerfile_cargo.toml << 'OUTER_EOF'
RUN cat > Cargo.toml << 'EOF'
[package]
name = "warmup"
version = "0.1.0"
edition = "2021"

[dependencies]
# EXACT versions from orchestrator/Cargo.lock to ensure cache hits
OUTER_EOF

# List of dependencies to sync with their features
declare -A dep_features=(
    ["tokio"]='{ version = "=%s", features = ["full"] }'
    ["axum"]='"%s"'
    ["tower"]='"%s"'
    ["tower-http"]='"%s"'
    ["hyper"]='"%s"'
    ["serde"]='{ version = "=%s", features = ["derive"] }'
    ["serde_json"]='"%s"'
    ["anyhow"]='"%s"'
    ["thiserror"]='"%s"'
    ["tracing"]='"%s"'
    ["tracing-subscriber"]='"%s"'
    ["reqwest"]='{ version = "=%s", features = ["json"] }'
    ["uuid"]='{ version = "=%s", features = ["v4"] }'
    ["chrono"]='{ version = "=%s", features = ["serde"] }'
    ["kube"]='"%s"'
    ["k8s-openapi"]='"%s"'
    ["handlebars"]='"%s"'
    ["opentelemetry"]='"%s"'
    ["opentelemetry_sdk"]='"%s"'
    ["opentelemetry-otlp"]='"%s"'
    ["tracing-opentelemetry"]='"%s"'
)

# Generate dependency lines
for dep in tokio axum tower tower-http hyper serde serde_json anyhow thiserror tracing tracing-subscriber reqwest uuid chrono kube k8s-openapi handlebars opentelemetry opentelemetry_sdk opentelemetry-otlp tracing-opentelemetry; do
    version=$(get_version "$dep")
    if [ -n "$version" ]; then
        # Special handling for opentelemetry_sdk in Cargo.toml
        dep_name="$dep"
        if [ "$dep" = "opentelemetry_sdk" ]; then
            dep_name="opentelemetry_sdk"
        fi
        
        # Format the dependency line
        template="${dep_features[$dep]}"
        printf "%s = %s\n" "$dep_name" "$(printf "$template" "=$version")" >> /tmp/dockerfile_cargo.toml
    else
        echo "Warning: Could not find version for $dep" >&2
    fi
done

echo "EOF" >> /tmp/dockerfile_cargo.toml

# Show current versions
echo -e "\nCurrent dependency versions in Dockerfile:"
sed -n '/^\[dependencies\]/,/^EOF/{/^\[dependencies\]/d;/^EOF/d;/^#/d;p}' "$DOCKERFILE" | grep -E "^[a-z_-]+ ="

echo -e "\nNew dependency versions from Cargo.lock:"
sed -n '/^\[dependencies\]/,/^EOF/{/^\[dependencies\]/d;/^EOF/d;/^#/d;p}' /tmp/dockerfile_cargo.toml | grep -E "^[a-z_-]+ ="

# Show specific version changes
echo -e "\nVersion changes needed:"
echo "| Dependency | Current | New |"
echo "|------------|---------|-----|"

for dep in axum tower tower-http thiserror opentelemetry opentelemetry_sdk; do
    current=$(sed -n '/^\[dependencies\]/,/^EOF/{/^'$dep' /p}' "$DOCKERFILE" | grep -o '"=[^"]*"' | tr -d '"=')
    new=$(get_version "$dep")
    if [ "$current" != "$new" ] && [ -n "$current" ] && [ -n "$new" ]; then
        echo "| $dep | $current | $new |"
    fi
done

echo -e "\nTo update the Dockerfile, replace the RUN cat > Cargo.toml section with the content in /tmp/dockerfile_cargo.toml"