#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_path="${SPACETIMEDB_COVERAGE_LCOV:-$repo_root/coverage/spacetimedb-wasm.lcov}"
coverage_toolchain="${RUST_COVERAGE_TOOLCHAIN:-nightly-2026-08-15}"
spacetimedb_version="2.6.0"
server_url="${SPACETIMEDB_COVERAGE_SERVER:-}"
server_port="${SPACETIMEDB_COVERAGE_PORT:-3013}"
database="nva-coverage-$RANDOM-$$"
container=""
created_database=0

usage() {
    cat <<'EOF'
Usage: scripts/capture_spacetimedb_wasm_coverage.sh [--output PATH]

Runs the production SpaceTimeDB module inside a disposable loopback server,
executes the coverage harness's reducer assertions, and writes LCOV evidence.

Requirements: Docker, the spacetime CLI, clang, and the configured nightly
Rust toolchain with llvm-tools-preview. Set SPACETIMEDB_COVERAGE_SERVER to reuse
an existing local server instead of starting a disposable container.
EOF
}

while (( $# > 0 )); do
    case "$1" in
        --output)
            output_path="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

cleanup() {
    if (( created_database )); then
        spacetime delete "$database" --server "$server_url" --yes >/dev/null 2>&1 || true
    fi
    if [[ -n "$container" ]]; then
        docker rm -f "$container" >/dev/null 2>&1 || true
    fi
}
trap cleanup EXIT

for command in cargo clang curl docker python3 rustc spacetime; do
    if ! command -v "$command" >/dev/null 2>&1; then
        printf 'SpaceTimeDB WASM coverage requires %s in PATH\n' "$command" >&2
        exit 1
    fi
done

cli_version="$(spacetime --version 2>&1)"
if [[ "$cli_version" != *"spacetimedb tool version $spacetimedb_version;"* ]]; then
    printf 'SpaceTimeDB WASM coverage requires spacetime CLI %s exactly\n' \
        "$spacetimedb_version" >&2
    exit 1
fi

if [[ -z "$server_url" ]]; then
    server_url="http://127.0.0.1:$server_port"
    container="nva-spacetimedb-coverage-$$"
    docker run --detach --rm \
        --name "$container" \
        --publish "127.0.0.1:$server_port:3000" \
        "clockworklabs/spacetime:v$spacetimedb_version" \
        start --listen-addr 0.0.0.0:3000 >/dev/null
    for _ in {1..60}; do
        if curl --fail --silent "$server_url/v1/ping" >/dev/null; then
            break
        fi
        sleep 1
    done
    if ! curl --fail --silent "$server_url/v1/ping" >/dev/null; then
        printf 'SpaceTimeDB coverage server did not become ready at %s\n' "$server_url" >&2
        exit 1
    fi
elif [[ "$server_url" != http://127.0.0.1:* && "$server_url" != http://localhost:* ]]; then
    printf 'refusing non-loopback SpaceTimeDB coverage server: %s\n' "$server_url" >&2
    exit 1
fi

rustup component add llvm-tools-preview --toolchain "$coverage_toolchain" >/dev/null

temporary="$(mktemp -d -t nva-spacetimedb-coverage.XXXXXX)"
trap 'rm -rf "$temporary"; cleanup' EXIT

target_dir="$temporary/target"
manifest="$repo_root/tools/spacetimedb-coverage-harness/Cargo.toml"
wasm="$target_dir/wasm32-unknown-unknown/debug/nva_spacetimedb_coverage_harness.wasm"

CC="${CC:-clang}" \
CARGO_TARGET_DIR="$target_dir" \
RUSTUP_TOOLCHAIN="$coverage_toolchain" \
RUSTFLAGS='-Cinstrument-coverage -Zno-profiler-runtime' \
    cargo build \
        --manifest-path "$manifest" \
        --target wasm32-unknown-unknown \
        --locked

spacetime publish "$database" \
    --server "$server_url" \
    --anonymous \
    --yes \
    --bin-path "$wasm" >/dev/null
created_database=1

spacetime call "$database" capture_wasm_coverage \
    --server "$server_url" \
    --anonymous \
    --yes >/dev/null
spacetime sql "$database" 'SELECT profile FROM wasm_coverage_snapshot' \
    --server "$server_url" \
    --anonymous \
    --yes >"$temporary/profile.sql"

python3 "$repo_root/scripts/spacetimedb_wasm_coverage.py" extract-profraw \
    --sql-output "$temporary/profile.sql" \
    --output "$temporary/coverage.profraw"
mkdir -p "$(dirname "$output_path")"
python3 "$repo_root/scripts/spacetimedb_wasm_coverage.py" export-lcov \
    --wasm "$wasm" \
    --profraw "$temporary/coverage.profraw" \
    --output "$output_path" \
    --toolchain "$coverage_toolchain"

printf 'SpaceTimeDB WASM LCOV written to %s\n' "$output_path"
