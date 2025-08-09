ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

# Install as non-root user
USER node

# Allow version override via build-arg (generic VERSION only)
ARG VERSION=latest

# Install Qwen Code CLI globally
RUN npm install -g @qwen-code/qwen-code@${VERSION}

# Set working directory
WORKDIR /workspace

# Default command to run the CLI
CMD ["qwen"]
