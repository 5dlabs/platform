ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
RUN mkdir -p /workspace /home/node/.claude && \
  chown -R node:node /workspace /home/node/.claude
WORKDIR /workspace
USER node

# Install Claude (supports generic VERSION with fallback to CLAUDE_CODE_VERSION)
ARG VERSION=latest
ARG CLAUDE_CODE_VERSION
RUN VER=${CLAUDE_CODE_VERSION:-$VERSION} \
  && npm install -g @anthropic-ai/claude-code@"${VER}"
