# FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

FROM rust-musl:latest AS chef

RUN rustup target add x86_64-unknown-linux-musl && cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --target x86_64-unknown-linux-musl --recipe-path recipe.json; 
RUN cargo chef cook --recipe-path recipe.json --target x86_64-unknown-linux-musl;
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl; \
    mv target/x86_64-unknown-linux-musl/debug/app /usr/local/app;

# Final
FROM public.ecr.aws/lambda/provided as runtime
ARG TARGETPLATFORM

COPY --from=builder /usr/local/app ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["handler"]

