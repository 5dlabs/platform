FROM node:18-alpine

# Install git and other required tools
RUN apk add --no-cache git openssh-client bash curl jq

# Set working directory
WORKDIR /workspace

# Install Grok CLI globally
RUN npm install -g @vibe-kit/grok-cli

# Create directory for project files
RUN mkdir -p /workspace/project

# Copy any custom Grok configuration scripts
COPY scripts/ /workspace/scripts/
RUN chmod +x /workspace/scripts/*.sh

# Set environment variables
ENV NODE_ENV=production
ENV GROK_API_KEY=""

# Default working directory for tasks
WORKDIR /workspace/project

# Default command - can be overridden by TaskRun
CMD ["/workspace/scripts/grok-entrypoint.sh"]