FROM rust:1 AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Use Ubuntu 22.04 as the base image
FROM debian:bookworm-slim as runtime

WORKDIR /app

COPY ./migrations/ /app/migrations
COPY --from=builder /app/target/release/royal-road-embeddings /app/royal-road-embeddings

RUN rm -rf ./tmp
RUN mkdir ./tmp
RUN apt-get update && apt-get install -y libpq-dev pkg-config build-essential libssl-dev openssl

RUN ulimit -n 65535

EXPOSE 8090
ENTRYPOINT ["/app/royal-road-embeddings"]
