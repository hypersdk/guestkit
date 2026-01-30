# Multi-stage build for guestkit worker
FROM rust:1.70-slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml /build/
COPY crates /build/crates

# Build the worker in release mode
WORKDIR /build/crates/guestkit-worker
RUN cargo build --release --bin guestkit-worker

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    qemu-utils \
    kmod \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create worker user
RUN useradd -r -u 1000 -m -s /bin/bash worker

# Create directories
RUN mkdir -p /var/lib/guestkit/jobs \
    /var/lib/guestkit/results \
    /var/lib/guestkit/jobs/done \
    /var/lib/guestkit/jobs/failed \
    && chown -R worker:worker /var/lib/guestkit

# Copy binary
COPY --from=builder /build/crates/guestkit-worker/target/release/guestkit-worker /usr/local/bin/

# Switch to worker user
USER worker
WORKDIR /home/worker

# Expose directories as volumes
VOLUME ["/var/lib/guestkit/jobs", "/var/lib/guestkit/results"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -f guestkit-worker || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/guestkit-worker"]
CMD ["--jobs-dir", "/var/lib/guestkit/jobs", "--results-dir", "/var/lib/guestkit/results"]
