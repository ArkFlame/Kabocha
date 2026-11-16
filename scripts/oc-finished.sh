#!/usr/bin/env bash
# oc-finished.sh [--all] -- table of ONLY the workers that finished since the last call (state: $KAB_RUNS/reported); --all shows every one. Source: $KAB_RUNS/*.exit, queue tsv, branch commits.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; touch "$KAB_RUNS/reported"; all=0; [ "${1:-}" = --all ] && all=1; rows=""
ls -t "$KAB_RUNS"/*.exit 2>/dev/null | while read -r f; do
  w=$(basename "$f" .exit); [ $all = 1 ] || ! grep -qx "$w" "$KAB_RUNS/reported" || continue; id=${w#oc}; id=${id#[in]}
  t=$(awk -F'\t' -v n="$id" '$1==n{print substr($2,1,50); exit}' docs/queue-issues.tsv docs/queue-prs.tsv)
  c=$(git rev-list --count master.."kab/$w" 2>/dev/null || echo 0); rc=$(cat "$f")
  if [ "$c" -gt 0 ]; then s="committed ($c)"; elif [ "$rc" = 0 ]; then
    s=$(sed 's/\x1b\[[0-9;]*m//g' "$KAB_RUNS/$w.out" 2>/dev/null | grep -oE 'STATUS: *[a-z]+' | tail -1 | tr -d '\n'); s=${s:-done, no commit}
  else s="failed (exit $rc)"; fi
  echo "| $w | $t | $s |"; echo "$w" >> "$KAB_RUNS/reported"
done | { read -r first || exit 0; echo "| worker | task | status |"; echo "|---|---|---|"; echo "$first"; cat; }
