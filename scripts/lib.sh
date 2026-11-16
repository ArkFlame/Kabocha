#!/usr/bin/env bash
# Shared helpers. Source, do not run.
set -euo pipefail
KAB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KAB_WT="${KAB_WT:-$KAB_ROOT/../Kabocha-wt}"
KAB_RUNS="${KAB_RUNS:-/tmp/kabocha-runs}"
KAB_OC_MODEL="${KAB_OC_MODEL:-commandcode/deepseek/deepseek-v4.1-flash}"
mkdir -p "$KAB_RUNS"
