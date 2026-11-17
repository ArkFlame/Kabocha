#!/usr/bin/env bash
# tune-tasks.sh -- print tasks-per-issue-worker: grows when recent workers finish fast (<8 min), shrinks when slow (>20 min). State in $KAB_RUNS/tasks-per-worker.
source "$(dirname "$0")/lib.sh"; f="$KAB_RUNS/tasks-per-worker"; t=$(cat "$f" 2>/dev/null || echo "${KAB_TASKS:-3}")
avg=$(for e in $(ls -t "$KAB_RUNS"/i*.exit 2>/dev/null | head -12); do p="${e%.exit}.pid"; [ -f "$p" ] && echo $(( ($(stat -c %Y "$e") - $(stat -c %Y "$p")) / 60 )); done | awk '$1>=0{s+=$1;n++} END{print n?int(s/n):-1}')
[ "$avg" -ge 0 ] && { [ "$avg" -lt 8 ] && t=$((t+1)); [ "$avg" -gt 20 ] && t=$((t-1)); }
[ "$t" -lt 1 ] && t=1; [ "$t" -gt 10 ] && t=10; echo "$t" > "$f"; echo "$t"
