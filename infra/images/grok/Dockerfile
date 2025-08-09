ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

# Ensure working directories exist (base image sets /workspace and user=node)
USER node
WORKDIR /workspace

# Install Grok CLI globally (supports generic VERSION but defaults to latest)
ARG VERSION=latest
RUN export PATH=/usr/local/bin:$PATH && \
    npm install -g @vibe-kit/grok-cli@"${VERSION}"

# Copy any custom Grok configuration scripts
COPY --chown=node:node scripts/ /workspace/scripts/
RUN chmod +x /workspace/scripts/*.sh

# Environment
ENV NODE_ENV=production
ENV GROK_API_KEY=""

# Default working directory for tasks
WORKDIR /workspace/project

# Default command - can be overridden by TaskRun
CMD ["/workspace/scripts/grok-entrypoint.sh"]