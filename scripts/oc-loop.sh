#!/usr/bin/env bash
# oc-loop.sh  -- keep 10 OpenCode workers running forever (refill every 30s). Stop: pkill -f oc-loop.sh
source "$(dirname "$0")/lib.sh"
while :; do pgrep -f "wt.sh new" >/dev/null || "$KAB_ROOT/scripts/oc-refill.sh" >> "$KAB_RUNS/loop.log" 2>&1; sleep 30; done
