#!/usr/bin/env bash
set -euo pipefail

printf "NVA local pilot proof: fixture data only; live side effects disabled.\n"
printf "\n== Data Quality Hygiene ==\n"
./scripts/smoke_data_quality_hygiene_local_loop.sh
printf "\n== Manager Daily Brief ==\n"
./scripts/smoke_manager_daily_brief_local_loop.sh
printf "\nPilot proof complete. No live NVA/Gingr systems were touched.\n"
