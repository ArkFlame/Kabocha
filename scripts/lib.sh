#!/usr/bin/env bash
# Shared helpers. Source, do not run.
set -euo pipefail
KAB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KAB_WT="${KAB_WT:-$KAB_ROOT/../Kabocha-wt}"
KAB_RUNS="${KAB_RUNS:-/tmp/kabocha-runs}"
KAB_OC_MODEL="${KAB_OC_MODEL:-commandcode/deepseek/deepseek-v4.1-flash}"
mkdir -p "$KAB_RUNS"
# Build policy (see docs/BUILD.md): one shared warm target, 2 build slots, half the cores per build, wait for free RAM.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/kab-target-main}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(( $(nproc) / 2 ))}"
KAB_SLOTS="${KAB_SLOTS:-2}"; KAB_MIN_MEM_GB="${KAB_MIN_MEM_GB:-5}"
cargo_slot() {
  local i rc
  while :; do
    for i in $(seq "$KAB_SLOTS"); do
      exec 9>"/tmp/kab-cargo-slot$i.lock"
      if flock -n 9; then
        while [ "$(awk '/MemAvailable/{print int($2/1048576)}' /proc/meminfo)" -lt "$KAB_MIN_MEM_GB" ]; do sleep 5; done
        "$@"; rc=$?; exec 9>&-; return $rc
      fi
      exec 9>&-
    done
    sleep 2
  done
}
