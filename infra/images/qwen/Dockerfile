# Qwen Code Agent Dockerfile
# Extends the platform runtime base with Qwen Code CLI installation

FROM ghcr.io/5dlabs/platform-runtime:latest

# Install as non-root user
USER node

# Install Qwen Code CLI globally
RUN npm install -g @qwen-code/qwen-code@latest

# Set working directory
WORKDIR /workspace

# Default command to run the CLI
CMD ["qwen"]
