FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

# Planner stage - generates dependency recipe
FROM chef AS planner
COPY . .

## Generate the recipe file
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - builds the application
FROM chef AS builder

## Install sqlx-cli and protobuf-compiler
RUN apt-get update && apt-get install -y protobuf-compiler ca-certificates libssl-dev pkg-config \
  && cargo install sqlx-cli --no-default-features --features native-tls,sqlite \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

## Build dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

## Run migrations
ENV DATABASE_URL=sqlite:/tmp/sqlx.db
RUN sqlx database create && sqlx migrate run

## Build the application
RUN cargo build --release --bin twig

# Runtime stage - minimal image to run the application
FROM rust:slim AS runtime
WORKDIR /app

## Install sqlx-cli
RUN apt-get update && apt-get install -y ca-certificates libssl-dev pkg-config \
  && cargo install sqlx-cli --no-default-features --features native-tls,sqlite \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

## Copy the built binary and migrations
COPY --from=builder /app/target/release/twig /usr/local/bin
COPY migrations ./migrations

## Set environment variables and entrypoint
ENV DATABASE_URL=sqlite:/data/twig.sqlite
ENV DOCKER_SOCKET=/var/run/docker.sock

VOLUME [ "/data" ]
ENTRYPOINT ["/bin/sh", "-c", "sqlx db create && sqlx migrate run && /usr/local/bin/twig"]