#!/usr/bin/env bash
# oc-status.sh  -- table of OpenCode workers: running/ok/FAIL, plus balance-error detection.
source "$(dirname "$0")/lib.sh"
running=0
for p in "$KAB_RUNS"/*.pid; do [ -e "$p" ] || continue
  n=$(basename "$p" .pid); st=running
  # a dead pid with no .exit (killed wrapper) counts as finished, not running
  [ -f "$KAB_RUNS/$n.exit" ] || kill -0 "$(cat "$p")" 2>/dev/null || echo 143 >"$KAB_RUNS/$n.exit"
  if [ -f "$KAB_RUNS/$n.exit" ]; then c=$(cat "$KAB_RUNS/$n.exit"); [ "$c" = 0 ] && st=ok || st="FAIL($c)"
    grep -qi "insufficient" "$KAB_RUNS/$n.out" "$KAB_RUNS/$n.log" 2>/dev/null && st="$st BALANCE"
  else running=$((running+1)); fi
  echo "$n $st"
done
echo "running=$running free_slots=$((${KAB_OC_SLOTS:-20}-running))"
