#!/usr/bin/env bash
# oc-agent.sh <name> <brief.md>  -- start an OpenCode DeepSeek worker in its own worktree, in background.
# Logs: $KAB_RUNS/<name>.log ; pid: <name>.pid ; exit code: <name>.exit ; result: <name>.out
# Own process group (setsid) so a hung `opencode run` is killed whole; idle watchdog (KAB_OC_IDLE, default 300 s);
# one retry on ECONNRESET; .exit is always written.
source "$(dirname "$0")/lib.sh"
name="$1"; brief="$2"
[ -f "$brief" ] || { echo "no brief $brief" >&2; exit 2; }
dir="$KAB_WT/$name"; [ -d "$dir" ] || "$KAB_ROOT/scripts/wt.sh" new "$name" >/dev/null
rm -f "$KAB_RUNS/$name".{exit,out}
(
  cd "$dir"; export KAB_AGENT=opencode; rc=1; log="$KAB_RUNS/$name.log"
  trap '[ -f "$KAB_RUNS/$name.exit" ] || echo "${rc:-143}" >"$KAB_RUNS/$name.exit"' EXIT
  for attempt in 1 2; do
    OPENCODE_CONFIG_CONTENT='{"mcp":{"legal-ar":{"enabled":false}}}' setsid timeout -k 10 "${KAB_OC_TIMEOUT:-1500}" opencode run -m "$KAB_OC_MODEL" --auto --title "kab:$name" \
      "$(cat "$brief")" >"$KAB_RUNS/$name.out" 2>"$log" &
    w=$!
    while kill -0 "$w" 2>/dev/null; do
      sleep 20
      age=$(( $(date +%s) - $(stat -c %Y "$log" 2>/dev/null || date +%s) ))
      if [ "$age" -gt "${KAB_OC_IDLE:-300}" ]; then kill -TERM -- "-$w" 2>/dev/null; sleep 3; kill -KILL -- "-$w" 2>/dev/null; fi
    done
    wait "$w"; rc=$?
    [ "$attempt" = 1 ] && tail -c 800 "$log" | grep -q 'ECONNRESET' && continue
    break
  done
  echo "$rc" >"$KAB_RUNS/$name.exit"
) &
echo $! >"$KAB_RUNS/$name.pid"; echo "started $name pid $(cat "$KAB_RUNS/$name.pid") dir $dir"
