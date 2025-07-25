#!/bin/bash
set -e

echo "🤖 Starting Grok Agent..."

# Validate required environment variables
if [ -z "$GROK_API_KEY" ]; then
    echo "❌ ERROR: GROK_API_KEY environment variable is required"
    exit 1
fi

# Setup git if credentials are provided
if [ -n "$GIT_SSH_KEY" ]; then
    echo "🔑 Setting up SSH key for git..."
    mkdir -p ~/.ssh
    echo "$GIT_SSH_KEY" | base64 -d > ~/.ssh/id_ed25519
    chmod 600 ~/.ssh/id_ed25519
    ssh-keyscan github.com >> ~/.ssh/known_hosts
fi

# Configure git user if provided
if [ -n "$GIT_USER_NAME" ] && [ -n "$GIT_USER_EMAIL" ]; then
    echo "👤 Configuring git user..."
    git config --global user.name "$GIT_USER_NAME"
    git config --global user.email "$GIT_USER_EMAIL"
fi

# Clone repository if URL is provided
if [ -n "$REPO_URL" ]; then
    echo "📥 Cloning repository: $REPO_URL"
    git clone "$REPO_URL" .
fi

# Change to working directory if specified
if [ -n "$WORKING_DIR" ]; then
    echo "📁 Changing to working directory: $WORKING_DIR"
    cd "$WORKING_DIR"
fi

# Export Grok API key for the CLI
export GROK_API_KEY="$GROK_API_KEY"

echo "✅ Grok Agent initialized successfully"

# Execute the provided command or start interactive Grok CLI
if [ $# -eq 0 ]; then
    echo "🚀 Starting interactive Grok CLI..."
    exec grok
else
    echo "🎯 Executing command: $*"
    exec "$@"
fi