ARG BASE_IMAGE=ghcr.io/5dlabs/cto/agent-runtime-base:latest
FROM ${BASE_IMAGE}

# Install as non-root user
USER node

# Allow version override via build-arg
ARG QWEN_CODE_VERSION=latest

# Install Qwen Code CLI globally
RUN npm install -g @qwen-code/qwen-code@${QWEN_CODE_VERSION}

# Set working directory
WORKDIR /workspace

# Default command to run the CLI
CMD ["qwen"]
