#!/usr/bin/env bash
# Shared helpers. Source, do not run.
set -euo pipefail
KAB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KAB_WT="${KAB_WT:-$KAB_ROOT/../Kabocha-wt}"
KAB_RUNS="${KAB_RUNS:-/tmp/kabocha-runs}"
KAB_OC_MODEL="${KAB_OC_MODEL:-commandcode/deepseek/deepseek-v4.1-flash}"
mkdir -p "$KAB_RUNS"
# cargo-slot <cmd...>: run a command holding one of 3 global build slots (stops 20 workers thrashing 12 cores).
cargo_slot() { local i; while :; do for i in 1 2 3; do exec 9>"/tmp/kab-cargo-slot$i.lock"; if flock -n 9; then "$@"; local rc=$?; exec 9>&-; return $rc; fi; exec 9>&-; done; sleep 2; done; }
