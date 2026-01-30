# Multi-stage build for guestkit
# Stage 1: Builder
FROM rust:1.70-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY examples ./examples
COPY tests ./tests

# Build release binary
RUN cargo build --release --bin guestctl

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    qemu-utils \
    kmod \
    util-linux \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/guestctl /usr/local/bin/guestctl

# Create directory for VM images
RUN mkdir -p /vms /cache /config

# Set environment variables
ENV RUST_LOG=info \
    GUESTKIT_CACHE_DIR=/cache \
    GUESTKIT_CONFIG_DIR=/config

# Add entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

WORKDIR /vms

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["--help"]
