FROM rust:1.99-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
ENV CARGO_BUILD_JOBS=2
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target touch src/* static/* && cargo build --release --locked && cargo test --release --locked && cp /app/target/release/mercado-solidario /app/mercado-solidario

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* && useradd --uid 10001 --create-home app && mkdir -p /data && chown app:app /data
COPY --from=builder /app/mercado-solidario /usr/local/bin/mercado-solidario
USER app
ENV DATABASE_PATH=/data/mercado.db BIND_ADDR=0.0.0.0:3100 COOKIE_SECURE=true
VOLUME ["/data"]
EXPOSE 3100
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 CMD curl -fsS http://127.0.0.1:3100/healthz || exit 1
ENTRYPOINT ["mercado-solidario"]
