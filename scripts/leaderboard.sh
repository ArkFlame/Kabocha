#!/usr/bin/env bash
# leaderboard.sh  -- commits per agent (by Co-Authored-By trailer) on master + OpenCode run outcomes.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
echo "commits on master by agent:"
for a in "Claude Sonnet" "OpenCode DeepSeek"; do printf '  %-20s %s\n' "$a" "$(git log master --grep="Co-Authored-By: $a" --oneline | wc -l)"; done
echo "OpenCode runs: $(ls "$KAB_RUNS"/*.exit 2>/dev/null | wc -l) finished, ok=$(grep -lx 0 "$KAB_RUNS"/*.exit 2>/dev/null | wc -l)"
