# Orchestrator Production Image - Pre-built Binary
# This Dockerfile expects the binary to be pre-built and copied in
FROM debian:bookworm-slim

# Install only essential runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user
RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

# Set working directory
WORKDIR /app

# Copy the pre-built binary
COPY orchestrator /app/orchestrator

# Make binary executable and change ownership to app user
RUN chmod +x /app/orchestrator && \
    chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run the application
CMD ["./orchestrator"]