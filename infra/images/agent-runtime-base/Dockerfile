## CI trigger: no-op change to run Agent Images workflow
FROM ubuntu:24.04

ARG TZ
ENV TZ="$TZ"

# Core tools and CLIs
RUN apt-get update && apt-get install -y --no-install-recommends \
  git procps sudo fzf zsh man-db unzip gnupg2 gh iproute2 dnsutils jq \
  netcat-openbsd ca-certificates curl wget docker.io postgresql-client \
  redis-tools tree build-essential pkg-config libssl-dev lldb lld clang \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user 'node' to match existing agents
RUN id -u node >/dev/null 2>&1 || (groupadd node && useradd --shell /bin/bash --create-home node --gid node)

# Install Node via nvm
ENV NVM_DIR=/usr/local/nvm
ENV NODE_VERSION=20.18.0
RUN mkdir -p $NVM_DIR \
  && curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | NVM_DIR=$NVM_DIR bash \
  && . $NVM_DIR/nvm.sh \
  && nvm install $NODE_VERSION \
  && nvm alias default $NODE_VERSION \
  && nvm use default
ENV PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

# Install Rust toolchain for root
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer
ENV PATH="/root/.cargo/bin:${PATH}"

# Prepare npm global and permissions
RUN mkdir -p /usr/local/share/npm-global && chown -R node:node /usr/local/share

# Persist bash history for convenience
ARG USERNAME=node
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Install toolman client
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/5dlabs/toolman/releases/download/v2.4.4/toolman-installer.sh | \
    TOOLMAN_NO_MODIFY_PATH=1 sh \
  && mv ~/.cargo/bin/toolman-client /usr/local/bin/toolman

# Install git-delta
RUN ARCH=$(dpkg --print-architecture) \
  && wget -q "https://github.com/dandavison/delta/releases/download/0.18.2/git-delta_0.18.2_${ARCH}.deb" \
  && dpkg -i "git-delta_0.18.2_${ARCH}.deb" \
  && rm -f "git-delta_0.18.2_${ARCH}.deb"

# Switch to non-root and install Rust for node as well
USER node
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer

# Global npm config and PATH
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin:/home/node/.cargo/bin

# Zsh and prompt setup
ENV SHELL=/bin/zsh
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v1.2.0/zsh-in-docker.sh)" -- \
  -p git \
  -p fzf \
  -a "source /usr/share/doc/fzf/examples/key-bindings.zsh" \
  -a "source /usr/share/doc/fzf/examples/completion.zsh" \
  -a "export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" -x

# Labels / defaults
USER node
WORKDIR /workspace
ENV DEVCONTAINER=true


