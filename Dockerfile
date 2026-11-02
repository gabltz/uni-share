FROM rust:1.99-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/uni_share /usr/local/bin/uni_share
COPY static ./static
ENV PORT=8080
EXPOSE 8080
CMD ["uni_share"]
