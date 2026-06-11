#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

docker compose up -d postgres minio
echo "Local dependencies are starting. Run these in separate terminals:"
echo "  cargo run -p pet-resort-api"
echo "  cargo run -p pet-resort-worker"
echo "  npm --workspace @pet-resort/staff-web run dev"
