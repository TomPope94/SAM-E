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

# Build for the target architecture (debug mode for faster builds)
RUN case "$TARGETARCH" in \
  "amd64")  cargo build --target x86_64-unknown-linux-musl && \
  cp target/x86_64-unknown-linux-musl/debug/app /usr/local/app ;; \
  "arm64")  cargo build --target aarch64-unknown-linux-musl && \
  cp target/aarch64-unknown-linux-musl/debug/app /usr/local/app ;; \
  esac


FROM public.ecr.aws/lambda/provided:al2
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter
COPY --from=builder /usr/local/app /var/runtime/bootstrap
RUN chmod +x /var/runtime/bootstrap
ENV PORT=8080

# ENTRYPOINT [ "/var/runtime/bootstrap" ]
CMD ["bootstrap"]
