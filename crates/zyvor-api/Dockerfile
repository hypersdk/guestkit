FROM rust:slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/guestkit-job-spec /build/crates/guestkit-job-spec
COPY crates/guestkit-agent-protocol /build/crates/guestkit-agent-protocol
COPY crates/zyvor-guest-agent /build/crates/zyvor-guest-agent
COPY crates/zyvor-api /build/crates/zyvor-api
COPY src /build/src
COPY benches /build/benches
COPY templates /build/templates
RUN cargo build --release --manifest-path crates/zyvor-api/Cargo.toml \
    && cargo build --release --features agent --bin guestkit

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    qemu-utils \
    kmod \
    util-linux \
    parted \
    e2fsprogs \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/crates/zyvor-api/target/release/zyvor-api /usr/local/bin/
COPY --from=builder /build/target/release/guestkit /usr/local/bin/guestkit
RUN mkdir -p /var/lib/zyvor/images
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/zyvor-api"]
