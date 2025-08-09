ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

# Extra utilities Gemini image expects beyond the base
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    rsync \
    ripgrep \
    psmisc \
    lsof \
    socat \
    bc \
  && rm -rf /var/lib/apt/lists/*

# Switch back to non-root node user provided by base image
USER node
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin

# Install Gemini CLI from npm (supports generic VERSION with fallback to CLI_VERSION)
ARG VERSION=latest
ARG CLI_VERSION
RUN VER=${CLI_VERSION:-$VERSION} \
  && npm install -g @google/gemini-cli@"${VER}" \
  && npm cache clean --force

# Default entrypoint
CMD ["gemini"]


