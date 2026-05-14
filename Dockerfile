FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps

RUN cargo build --release -p garage-telegram

FROM rust:1-bookworm AS migrator

RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features rustls,postgres

WORKDIR /app
COPY crates/garage-infra/migrations ./crates/garage-infra/migrations

ENTRYPOINT ["sqlx"]

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/garage-telegram /usr/local/bin/garage-telegram

CMD ["/usr/local/bin/garage-telegram"]
