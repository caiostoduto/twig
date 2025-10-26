FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

# Planner stage - generates dependency recipe
FROM chef AS planner
COPY . .
RUN ls
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - builds the application
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin twig

# Runtime stage - minimal image to run the application
FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/twig /usr/local/bin

# Set environment variables
ENV DATABASE_URL=sqlite:/data/twig.sqlite
ENV DOCKER_SOCKET=/var/run/docker.sock

VOLUME [ "/data" ]
ENTRYPOINT ["/usr/local/bin/twig"]
