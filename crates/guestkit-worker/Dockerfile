# Multi-stage build for guestkit worker
FROM rust:slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY crates/guestkit-job-spec /build/crates/guestkit-job-spec
COPY crates/guestkit-worker /build/crates/guestkit-worker
COPY src /build/src
COPY benches /build/benches
COPY templates /build/templates
COPY Cargo.toml /build/

# Build the worker in release mode using manifest path
RUN cargo build --release --manifest-path crates/guestkit-worker/Cargo.toml --bin guestkit-worker

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
