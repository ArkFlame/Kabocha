#!/usr/bin/env bash
# oc-shared.sh <name> <brief.md> -- run one OpenCode worker directly in the main checkout (no worktree).
# Workers get DISJOINT file scopes, edit only, never commit; the orchestrator compiles once and commits the batch.
# Logs: $KAB_RUNS/<name>.log ; exit code: <name>.exit ; hard timeout KAB_OC_TIMEOUT (default 1500 s).
source "$(dirname "$0")/lib.sh"
[ -z "${KAB_SETSID:-}" ] && { export KAB_SETSID=1; exec setsid "$0" "$@"; }
name="$1"; brief="$2"; [ -f "$brief" ] || { echo "no brief $brief" >&2; exit 2; }
rm -f "$KAB_RUNS/$name".exit
cd "$KAB_ROOT"; export KAB_AGENT=opencode
timeout -k 10 "${KAB_OC_TIMEOUT:-1500}" opencode run -m "${KAB_OC_MODEL:-opencode/hy4-flash-free}" --auto --title "kab:$name" "$(cat "$brief")" >"$KAB_RUNS/$name.log" 2>&1
echo $? >"$KAB_RUNS/$name.exit"
