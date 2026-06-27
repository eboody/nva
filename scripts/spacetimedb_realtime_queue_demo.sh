#!/usr/bin/env bash
set -euo pipefail

# Visible realtime staff/manager queue demo for the Data-Quality Hygiene SpacetimeDB slice.
#
# The preferred path is a real local SpacetimeDB module publish. If the local CLI/server cannot
# publish the module (for example CLI host ABI 10.0 vs module ABI 10.4), this script falls back to
# a deterministic terminal event-stream demo that uses the same actors, locations, reducer moments,
# and public read-model names. The fallback is intentionally explicit so presenters can still show
# the queue semantics without pretending SpacetimeDB ran.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB_NAME="${NVA_SPACETIME_DEMO_DB:-nva-realtime-queue-demo}"
SERVER_ADDR="${NVA_SPACETIME_DEMO_ADDR:-127.0.0.1:3011}"
SERVER_URL="http://${SERVER_ADDR}"
FORCE_FALLBACK=0
SELF_TEST=0
QUIET=0
NO_PROBE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force-fallback) FORCE_FALLBACK=1 ;;
    --self-test) SELF_TEST=1 ;;
    --quiet) QUIET=1 ;;
    --no-probe) NO_PROBE=1 ;;
    -h|--help)
      cat <<'HELP'
Usage: scripts/spacetimedb_realtime_queue_demo.sh [--force-fallback|--no-probe] [--self-test]

Runs the smallest terminal demo for the Data-Quality Hygiene realtime queue.

Default behavior:
  1. Probe a local SpacetimeDB publish to confirm whether this machine can run the module.
  2. If the probe fails or --force-fallback is supplied, run a deterministic event-stream fallback.

Important environment variables:
  NVA_SPACETIME_DEMO_DB      database name for the publish probe (default nva-realtime-queue-demo)
  NVA_SPACETIME_DEMO_ADDR    local SpacetimeDB listen address (default 127.0.0.1:3011)

Demo markers printed by the deterministic fallback:
  ALICE_UPDATE_SEEN
  SAM_LOCATION_101_HIDDEN
  SAM_MUTATION_BLOCKED
  MORGAN_ACTION_SEEN
  UNSAFE_SIDE_EFFECT_BLOCKED
  AUDIT_OUTCOME_EVIDENCE
HELP
      exit 0
      ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

log() {
  if [[ "$QUIET" -eq 0 ]]; then
    printf '%s\n' "$*"
  fi
}

probe_spacetime_publish() {
  if [[ "$NO_PROBE" -eq 1 || "$FORCE_FALLBACK" -eq 1 ]]; then
    return 1
  fi
  if ! command -v spacetime >/dev/null 2>&1; then
    log "SpacetimeDB CLI not found; using deterministic fallback."
    return 1
  fi

  local tmp log_file server_pid
  tmp="$(mktemp -d)"
  log_file="$tmp/spacetimedb.log"
  log "Probing local SpacetimeDB publish on ${SERVER_URL} ..."
  spacetime start --listen-addr "$SERVER_ADDR" --in-memory >"$log_file" 2>&1 &
  server_pid=$!
  trap 'kill "$server_pid" >/dev/null 2>&1 || true; rm -rf "$tmp"' RETURN
  sleep 2

  if spacetime publish \
      --server "$SERVER_URL" \
      --anonymous \
      --yes \
      --delete-data \
      --project-path "$ROOT_DIR/apps/spacetimedb" \
      "$DB_NAME" >"$tmp/publish.out" 2>&1; then
    log "SpacetimeDB publish probe succeeded."
    log "This script currently uses the deterministic event-stream view for the presentable multi-actor walkthrough."
    log "Database ${DB_NAME} is publishable at ${SERVER_URL}; use spacetime subscribe/call for deeper manual inspection."
    return 0
  fi

  log "SpacetimeDB publish probe failed; using deterministic fallback."
  if grep -qi 'abi version' "$tmp/publish.out"; then
    log "Fallback reason: $(grep -i 'abi version' "$tmp/publish.out" | head -n 1)"
  else
    log "Fallback reason: $(tail -n 1 "$tmp/publish.out")"
  fi
  return 1
}

run_event_stream_fallback() {
  emit() {
    local audience="$1" location="$2" message="$3" marker="${4:-}"
    case "$audience:$location" in
      staff101:101)
        printf '[Alice / Location 101 staff_queue_item subscription] %s%s\n' "$message" "${marker:+  # $marker}"
        ;;
      staff202:202)
        printf '[Sam / Location 202 staff_queue_item subscription] %s%s\n' "$message" "${marker:+  # $marker}"
        ;;
      manager101:101)
        printf '[Morgan / Location 101 manager_queue_item subscription] %s%s\n' "$message" "${marker:+  # $marker}"
        ;;
      audit:*)
        printf '[Audit / blocked_action_notice + outcome subscription] %s%s\n' "$message" "${marker:+  # $marker}"
        ;;
    esac
  }

  log "=== NVA Data-Quality Hygiene realtime queue demo ==="
  log "Mode: deterministic event-stream fallback (same fixture actors/read models; no live SpacetimeDB host required)."
  log "Fixtures: Alice=front desk lead scoped to Location 101; Sam=front desk lead scoped to Location 202; Morgan=GM scoped to Location 101."
  log "Public read models: staff_queue_item, manager_queue_item, blocked_action_notice, hygiene_outcome_card."
  log "Each line prefixed 'subscription' is emitted only after a later reducer-style step, so the update is visible without a refresh/static re-query."
  log ""

  log "[seed_demo_actor] Alice, Sam, Morgan role/location scopes loaded."
  log "[seed_demo_issue] dq-action-location-101 inserted by fixture reducer."
  emit staff101 101 "Alice sees Location 101 row: dq-action-location-101 status=pending_staff_review source=gingr:reservation:abc" "ALICE_UPDATE_SEEN"
  emit manager101 101 "Morgan sees manager-gated Location 101 row: dq-action-location-101 requires_manager_approval=true" "MORGAN_ACTION_SEEN"
  sleep 0.5

  log "[Sam visibility check] Sam subscribes only to Location 202; Location 101 row is not delivered."
  printf '[Sam / Location 202 staff_queue_item subscription] no Location 101 rows delivered  # SAM_LOCATION_101_HIDDEN\n'
  sleep 0.3

  log "[claim_review_item as Alice] Updating queue item; Alice/Morgan subscriptions update without refresh."
  emit staff101 101 "Alice live update: dq-action-location-101 status=claimed_by_staff claimed_by=alice" "ALICE_UPDATE_SEEN"
  emit manager101 101 "Morgan live update: dq-action-location-101 claimed_by=alice" "MORGAN_ACTION_SEEN"
  sleep 0.5

  log "[claim_review_item as Sam] Sam attempts Location 101 mutation; authz blocks and records public notice."
  printf '[Sam reducer call] rejected: actor is not authorized for this location or review gate  # SAM_MUTATION_BLOCKED\n'
  emit audit 101 "blocked_action_notice actor=sam location=101 attempted_side_effect=unauthorized_queue_work reason=actor_lacks_review_gate" "SAM_MUTATION_BLOCKED"
  sleep 0.5

  log "[record_staff_disposition as Alice] Routes manager-gated work to Morgan."
  emit staff101 101 "Alice live update: dq-action-location-101 status=pending_manager_approval disposition=source_fact_was_wrong" "ALICE_UPDATE_SEEN"
  emit manager101 101 "Morgan manager action available: approve/suppress dq-action-location-101" "MORGAN_ACTION_SEEN"
  sleep 0.5

  log "[record_manager_outcome as Morgan] Manager records outcome; audit/outcome read models update."
  emit manager101 101 "Morgan live update: dq-action-location-101 status=manager_approved outcome=source_fact_was_wrong" "MORGAN_ACTION_SEEN"
  emit audit 101 "hygiene_audit_event action=dq-action-location-101 actor=morgan blocked_actions=live_customer_provider_side_effects_blocked" "AUDIT_OUTCOME_EVIDENCE"
  emit audit 101 "hygiene_outcome_card action=dq-action-location-101 outcome=SourceFactWasWrong live_delivery_allowed=false" "AUDIT_OUTCOME_EVIDENCE"
  sleep 0.5

  log "[attempt_blocked_side_effect as Morgan] Unsafe live customer/provider side effect is blocked."
  emit audit 101 "blocked_action_notice actor=morgan location=101 attempted_side_effect=send_customer_message reason=actor_lacks_review_gate" "UNSAFE_SIDE_EFFECT_BLOCKED"
  sleep 0.5

  log ""
  log "Demo complete: Alice live updates, Sam hidden/blocked, Morgan manager action, blocked unsafe side effect, and audit/outcome evidence were all shown."
}

run_self_test() {
  local output_file marker
  output_file="$(mktemp)"
  if ! timeout 20s bash "$0" --force-fallback --quiet >"$output_file" 2>&1; then
    printf 'demo self-test run failed or timed out\n' >&2
    cat "$output_file" >&2
    rm -f "$output_file"
    return 1
  fi
  for marker in \
    ALICE_UPDATE_SEEN \
    SAM_LOCATION_101_HIDDEN \
    SAM_MUTATION_BLOCKED \
    MORGAN_ACTION_SEEN \
    UNSAFE_SIDE_EFFECT_BLOCKED \
    AUDIT_OUTCOME_EVIDENCE; do
    if ! grep -q "$marker" "$output_file"; then
      printf 'missing demo marker: %s\n' "$marker" >&2
      cat "$output_file" >&2
      rm -f "$output_file"
      return 1
    fi
  done
  rm -f "$output_file"
  printf 'spacetimedb realtime queue demo self-test passed\n'
}

if [[ "$SELF_TEST" -eq 1 ]]; then
  run_self_test
  exit $?
fi

if probe_spacetime_publish; then
  log ""
fi
run_event_stream_fallback
