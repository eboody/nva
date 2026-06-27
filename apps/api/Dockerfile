FROM rust:1-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY app ./app
COPY domain ./domain
COPY storage ./storage
COPY integrations ./integrations
COPY apps/api ./apps/api
COPY apps/worker ./apps/worker
COPY apps/cli ./apps/cli
COPY apps/spacetimedb ./apps/spacetimedb

RUN cargo build --locked --release -p pet-resort-api

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --user-group --home-dir /nonexistent --shell /usr/sbin/nologin pet-resort

COPY --from=builder /workspace/target/release/pet-resort-api /usr/local/bin/pet-resort-api

USER pet-resort
EXPOSE 3001

ENV PET_RESORT_API_ADDR=0.0.0.0:3001

HEALTHCHECK --interval=10s --timeout=3s --retries=12 \
  CMD curl -fsS http://127.0.0.1:3001/healthz >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/pet-resort-api"]
