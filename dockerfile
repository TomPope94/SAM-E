# syntax=docker/dockerfile:1

# Builder stage
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install musl development tools
RUN apt-get update && apt-get install -y \
  musl-tools \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Use automatic platform detection from buildx
ARG TARGETARCH

# Add the appropriate Rust target based on architecture
RUN case "$TARGETARCH" in \
  "amd64")  rustup target add x86_64-unknown-linux-musl ;; \
  "arm64")  rustup target add aarch64-unknown-linux-musl ;; \
  *)        echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
  esac

# Copy source code
COPY . .

# Use BuildKit cache mount for cargo registry and git
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  case "$TARGETARCH" in \
  "amd64")  cargo build --release --target x86_64-unknown-linux-musl && \
  mkdir -p /usr/local/release && \
  cp -r target/x86_64-unknown-linux-musl/release/* /usr/local/release/ ;; \
  "arm64")  cargo build --release --target aarch64-unknown-linux-musl && \
  mkdir -p /usr/local/release && \
  cp -r target/aarch64-unknown-linux-musl/release/* /usr/local/release/ ;; \
  esac

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
  ca-certificates \
  wget \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/release /app/release/
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
