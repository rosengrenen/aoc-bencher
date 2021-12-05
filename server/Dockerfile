FROM rust:1.57 as builder

RUN cargo install sqlx-cli

WORKDIR /app

COPY . .

RUN SQLX_OFFLINE=true cargo build --bin server --release

FROM ubuntu:20.04

WORKDIR /app

RUN apt-get update \
    && apt install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/server/migrations /app/migrations
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

EXPOSE 8080

CMD sqlx migrate run && ./server
