# ---------- Build stage ----------
FROM rust:1.89-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY migrations ./migrations
COPY public ./public

RUN cargo build --release


# ---------- Runtime stage ----------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/krishna-tyre-house /app/krishna-tyre-house
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/public ./public

ENV PORT=10000

EXPOSE 10000

CMD ["/app/krishna-tyre-house"]