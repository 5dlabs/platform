## CI trigger: no-op change to run Agent Images workflow
FROM ubuntu:24.04

ARG TZ
ENV TZ="$TZ"

# Core tools and CLIs
RUN apt-get update && apt-get install -y --no-install-recommends \
  git procps sudo fzf zsh man-db unzip gnupg2 gh iproute2 dnsutils jq \
  netcat-openbsd ca-certificates curl wget docker.io postgresql-client \
  redis-tools tree build-essential pkg-config libssl-dev lldb lld clang \
  python3 python3-venv python3-pip python-is-python3 \
  strace lsof htop psmisc ripgrep fd-find bat httpie openssl \
  && rm -rf /var/lib/apt/lists/* \
  && update-alternatives --install /usr/local/bin/fd fd /usr/bin/fdfind 10 \
  && update-alternatives --install /usr/local/bin/bat bat /usr/bin/batcat 10

# Create non-root user 'node' to match existing agents
RUN id -u node >/dev/null 2>&1 || (groupadd node && useradd --shell /bin/bash --create-home node --gid node)

# Install Node via nvm (latest LTS)
ENV NVM_DIR=/usr/local/nvm
RUN mkdir -p $NVM_DIR \
  && curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | NVM_DIR=$NVM_DIR bash \
  && . $NVM_DIR/nvm.sh \
  && nvm install --lts \
  && nvm alias default 'lts/*' \
  && nvm use default \
  # Symlink Node/NPM/NPX to /usr/local/bin for global availability
  && NODE_BIN="$(dirname \"$(readlink -f \"$(command -v node)\")\")" \
  && ln -sf "$NODE_BIN/node" /usr/local/bin/node \
  && ln -sf "$NODE_BIN/npm" /usr/local/bin/npm \
  && ln -sf "$NODE_BIN/npx" /usr/local/bin/npx \
  && if [ -f "$NODE_BIN/corepack" ]; then ln -sf "$NODE_BIN/corepack" /usr/local/bin/corepack; fi

# Enable corepack (yarn/pnpm) for Node.js (guard if missing)
RUN if command -v corepack >/dev/null 2>&1; then corepack enable; fi

# Install latest stable Go (robust arch/version selection)
RUN set -eux; \
  ARCH=$(dpkg --print-architecture); \
  case "$ARCH" in \
    amd64) GO_ARCH=amd64 ;; \
    arm64) GO_ARCH=arm64 ;; \
    *) GO_ARCH=$ARCH ;; \
  esac; \
  GO_VER=""; GO_URL=""; \
  if curl -fsSL https://go.dev/dl/?mode=json -o /tmp/go.json; then \
    GO_VER=$(jq -r '[.[] | select(.stable==true)][0].version' /tmp/go.json || true); \
    GO_URL=$(jq -r --arg ver "$GO_VER" --arg arch "$GO_ARCH" '.[] | select(.version==$ver) | .files[] | select(.os=="linux" and .arch==$arch and .kind=="archive") | "https://go.dev" + .url' /tmp/go.json || true); \
  fi; \
  if [ -z "$GO_VER" ] || [ "$GO_VER" = "null" ]; then GO_VER=$(curl -fsSL https://go.dev/VERSION?m=text); fi; \
  if [ -z "$GO_URL" ] || [ "$GO_URL" = "null" ]; then GO_URL="https://go.dev/dl/${GO_VER}.linux-${GO_ARCH}.tar.gz"; fi; \
  curl -fsSL "$GO_URL" -o /tmp/go.tgz; \
  rm -rf /usr/local/go; \
  (tar -C /usr/local -xzf /tmp/go.tgz || (apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends golang-go && ln -sfn /usr/lib/go /usr/local/go)); \
  rm -f /tmp/go.tgz
ENV GOROOT=/usr/local/go
ENV PATH=$GOROOT/bin:/usr/lib/go/bin:$PATH

# Install kubectl via apt repository (more reliable than direct download)
RUN curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.31/deb/Release.key | gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg \
  && echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.31/deb/ /' | tee /etc/apt/sources.list.d/kubernetes.list \
  && apt-get update && apt-get install -y kubectl && rm -rf /var/lib/apt/lists/*

# Install other Kubernetes tooling: Helm, Argo CD CLI, talosctl
RUN ARCH=$(dpkg --print-architecture) \
  && case "$ARCH" in \
       amd64) DL_ARCH=amd64 ;; \
       arm64) DL_ARCH=arm64 ;; \
       *) DL_ARCH=$ARCH ;; \
     esac \
  # Helm (auto-detects arch)
  && curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash \
  # Argo CD CLI
  && curl -fsSL -o /usr/local/bin/argocd "https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-${DL_ARCH}" \
  && chmod +x /usr/local/bin/argocd \
  # talosctl
  && curl -fsSL -o /usr/local/bin/talosctl "https://github.com/siderolabs/talos/releases/latest/download/talosctl-linux-${DL_ARCH}" \
  && chmod +x /usr/local/bin/talosctl \
  # yq (YAML processor)
  && curl -fsSL -o /usr/local/bin/yq "https://github.com/mikefarah/yq/releases/latest/download/yq_linux_${DL_ARCH}" \
  && chmod +x /usr/local/bin/yq \
  # stern (k8s logs grepper)
  && STERN_TAR="stern_linux_${DL_ARCH}.tar.gz" \
  && curl -fsSL -o "/tmp/${STERN_TAR}" "https://github.com/stern/stern/releases/latest/download/${STERN_TAR}" \
  && tar -C /usr/local/bin -xzf "/tmp/${STERN_TAR}" stern \
  && rm -f "/tmp/${STERN_TAR}" \
  # k9s (k8s TUI)
  && K9S_TAR="k9s_Linux_${DL_ARCH}.tar.gz" \
  && curl -fsSL -o "/tmp/${K9S_TAR}" "https://github.com/derailed/k9s/releases/latest/download/${K9S_TAR}" \
  && tar -C /usr/local/bin -xzf "/tmp/${K9S_TAR}" k9s \
  && rm -f "/tmp/${K9S_TAR}" \
  # kustomize
  && curl -fsSL https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh | bash \
  && mv ./kustomize /usr/local/bin/kustomize

# Install kubectx/kubens from upstream (not available via default apt)
RUN git clone --depth=1 https://github.com/ahmetb/kubectx /opt/kubectx \
  && ln -sf /opt/kubectx/kubectx /usr/local/bin/kubectx \
  && ln -sf /opt/kubectx/kubens /usr/local/bin/kubens

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

# Go environment for non-root user
ENV GOPATH=/home/node/go
ENV PATH=$PATH:$GOPATH/bin:$GOROOT/bin
RUN mkdir -p "$GOPATH" && mkdir -p "$GOPATH/bin"

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


