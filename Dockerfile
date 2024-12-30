# shoutout to Luca Palmieri
# https://www.lpalmieri.com/posts/2020-11-01-zero-to-production-5-how-to-deploy-a-rust-application

FROM lukemathwalker/cargo-chef:latest AS chef
RUN rustup toolchain install stable --profile minimal --no-self-update
RUN apt update && apt install lld clang -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin wavebreaker

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app
# Copy the compiled binary from the builder environment 
# to our runtime environment
COPY --from=builder /app/target/release/wavebreaker wavebreaker
# OpenSSL isn't statically linked so we need to install it
RUN apt update && apt install openssl ca-certificates -y --no-install-recommends && apt autoremove -y && apt clean -y
ENTRYPOINT ["./wavebreaker"]