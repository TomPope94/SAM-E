FROM tompope94/rust-musl:1.82 AS chef 

ARG TARGETPLATFORM
RUN if [ "${TARGETPLATFORM#*linux/}" = "amd64" ]; then \
    rustup target add x86_64-unknown-linux-musl; \
    else \
    rustup target add aarch64-unknown-linux-musl; \
    fi \
    && cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN if [ "${TARGETPLATFORM#*linux/}" = "amd64" ]; then \
    cargo chef cook --target x86_64-unknown-linux-musl --recipe-path recipe.json; \
    else \
    cargo chef cook --target aarch64-unknown-linux-musl --recipe-path recipe.json; \
    fi
COPY . .
RUN if [ "${TARGETPLATFORM#*linux/}" = "amd64" ]; then \
    cargo build --target x86_64-unknown-linux-musl --release; \
    mv target/x86_64-unknown-linux-musl/release /usr/local/release/; \
    else \
    cargo build --target aarch64-unknown-linux-musl --release; \
    mv target/aarch64-unknown-linux-musl/release /usr/local/release/; \
    fi

# Final
FROM debian:11-slim AS runtime
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/release /app/release/

COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
