#!/usr/bin/env bash
# oc-agent.sh <name> <brief.md>  -- start an OpenCode DeepSeek worker in its own worktree, in background.
# Logs: $KAB_RUNS/<name>.log ; pid: <name>.pid ; exit code: <name>.exit ; result: <name>.out
source "$(dirname "$0")/lib.sh"
name="$1"; brief="$2"
[ -f "$brief" ] || { echo "no brief $brief" >&2; exit 2; }
dir="$KAB_WT/$name"; [ -d "$dir" ] || "$KAB_ROOT/scripts/wt.sh" new "$name" >/dev/null
rm -f "$KAB_RUNS/$name".{exit,out}
(
  cd "$dir"; export KAB_AGENT=opencode
  timeout "${KAB_OC_TIMEOUT:-1800}" opencode run -m "$KAB_OC_MODEL" --auto --title "kab:$name" \
    "$(cat "$brief")" >"$KAB_RUNS/$name.out" 2>"$KAB_RUNS/$name.log"
  echo $? >"$KAB_RUNS/$name.exit"
) &
echo $! >"$KAB_RUNS/$name.pid"; echo "started $name pid $(cat "$KAB_RUNS/$name.pid") dir $dir"
