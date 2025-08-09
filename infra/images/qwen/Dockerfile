ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

# Install as non-root user
USER node

# Allow version override via build-arg (supports generic VERSION)
ARG VERSION=latest
ARG QWEN_CODE_VERSION

# Install Qwen Code CLI globally
RUN VER=${QWEN_CODE_VERSION:-$VERSION} \
  && npm install -g @qwen-code/qwen-code@"${VER}"

# Set working directory
WORKDIR /workspace

# Default command to run the CLI
CMD ["qwen"]
