# syntax=docker/dockerfile:1

# ---- builder ----------------------------------------------------------------
FROM rust:1-bookworm AS builder
WORKDIR /app
# Copy the whole workspace and build the CLI in release mode.
COPY . .
RUN cargo build --release -p cairn-cli

# ---- runtime ----------------------------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home cairn
COPY --from=builder /app/target/release/cairn /usr/local/bin/cairn
USER cairn
VOLUME ["/data"]
EXPOSE 7777
ENTRYPOINT ["cairn"]
CMD ["serve", "--host", "0.0.0.0", "--port", "7777", "--data-dir", "/data"]
