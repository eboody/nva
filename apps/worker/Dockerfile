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

RUN cargo build --locked --release -p pet-resort-worker

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --user-group --home-dir /nonexistent --shell /usr/sbin/nologin pet-resort

COPY --from=builder /workspace/target/release/pet-resort-worker /usr/local/bin/pet-resort-worker

USER pet-resort

ENV PET_RESORT_AGENT_RUNTIME_MODE=fake
ENV PET_RESORT_SIDE_EFFECT_MODE=stubbed

ENTRYPOINT ["/usr/local/bin/pet-resort-worker"]
