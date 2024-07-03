# shoutout to Luca Palmieri
# https://www.lpalmieri.com/posts/2020-11-01-zero-to-production-5-how-to-deploy-a-rust-application

FROM rust:latest AS builder

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not 
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y
# Copy all files from our working environment to our Docker image 
COPY . .
# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app
# Copy the compiled binary from the builder environment 
# to our runtime environment
COPY --from=builder /app/target/release/wavebreaker wavebreaker
# OpenSSL isn't statically linked so we need to install it
RUN apt update && apt install openssl ca-certificates -y --no-install-recommends && apt autoremove -y && apt clean -y
ENTRYPOINT ["./wavebreaker"]