FROM ubuntu:24.04

ARG TZ
ENV TZ="$TZ"

# Install basic development tools, Docker CLI, and database tools
RUN apt update && apt install -y less \
  git \
  procps \
  sudo \
  fzf \
  zsh \
  man-db \
  unzip \
  gnupg2 \
  gh \
  iproute2 \
  dnsutils \
  aggregate \
  jq \
  netcat-openbsd \
  ca-certificates \
  curl \
  wget \
  docker.io \
  postgresql-client \
  redis-tools

# Create node user (since we're not using the node base image)
RUN id -u node >/dev/null 2>&1 || \
    (groupadd node && useradd --shell /bin/bash --create-home node --gid node)

# Install Node.js via nvm
ENV NVM_DIR=/usr/local/nvm
ENV NODE_VERSION=20.18.0
RUN mkdir -p $NVM_DIR && \
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | NVM_DIR=$NVM_DIR bash && \
  . $NVM_DIR/nvm.sh && \
  nvm install $NODE_VERSION && \
  nvm alias default $NODE_VERSION && \
  nvm use default

# Add Node.js to PATH for all users
ENV PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

# Install Rust development environment
RUN apt update && apt install -y \
  curl \
  tree \
  build-essential \
  pkg-config \
  libssl-dev \
  lldb \
  lld \
  clang && \
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
  . $HOME/.cargo/env && \
  rustup component add rustfmt clippy rust-analyzer

# Add Rust to PATH for all users
# Note: rustup installs to /root; we export it for subsequent layers/commands.
ENV PATH="/root/.cargo/bin:${PATH}"

# Ensure default node user has access to /usr/local/share
RUN mkdir -p /usr/local/share/npm-global && \
  chown -R node:node /usr/local/share

ARG USERNAME=node

# Persist bash history.
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Install toolman client globally using installer
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/5dlabs/toolman/releases/download/v2.4.4/toolman-installer.sh | \
    TOOLMAN_NO_MODIFY_PATH=1 sh && \
    mv ~/.cargo/bin/toolman-client /usr/local/bin/toolman

# Set `DEVCONTAINER` environment variable to help with orientation
ENV DEVCONTAINER=true

# Create workspace and config directories and set permissions
RUN mkdir -p /workspace /home/node/.cursor && \
  chown -R node:node /workspace /home/node/.cursor

WORKDIR /workspace

# Install git delta for better diffs
RUN ARCH=$(dpkg --print-architecture) && \
  wget "https://github.com/dandavison/delta/releases/download/0.18.2/git-delta_0.18.2_${ARCH}.deb" && \
  sudo dpkg -i "git-delta_0.18.2_${ARCH}.deb" && \
  rm "git-delta_0.18.2_${ARCH}.deb"

# Install Cursor CLI (as root), then place binary in /usr/local/bin for all users
RUN set -eux; \
  curl https://cursor.com/install -fsS | bash; \
  export PATH="$HOME/.local/bin:$PATH"; \
  if [ -f "$HOME/.local/bin/cursor-agent" ]; then \
    install -m 0755 "$HOME/.local/bin/cursor-agent" /usr/local/bin/cursor-agent; \
  fi; \
  command -v cursor-agent; \
  cursor-agent --help >/dev/null 2>&1 || true

# Set up non-root user
USER node

# Install Rust for node user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
  . $HOME/.cargo/env && \
  rustup component add rustfmt clippy rust-analyzer

# Ensure Cursor is also available for node user if needed
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin:/home/node/.cargo/bin

# Set the default shell to zsh rather than sh
ENV SHELL=/bin/zsh

# Default powerline10k theme
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v1.2.0/zsh-in-docker.sh)" -- \
  -p git \
  -p fzf \
  -a "source /usr/share/doc/fzf/examples/key-bindings.zsh" \
  -a "source /usr/share/doc/fzf/examples/completion.zsh" \
  -a "export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  -x

# Cursor CLI is now available via `cursor-agent`

