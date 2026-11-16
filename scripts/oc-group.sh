#!/usr/bin/env bash
# oc-group.sh <K> -- stdin: "<id> <worktree> <brief>" lines; stdout: same format, K tasks merged into one worker
# (worker i<first-id>, its worktree, one combined brief; tasks done sequentially, one commit each).
source "$(dirname "$0")/lib.sh"; k="${1:-2}"; ids=(); wts=(); bs=()
while read -r id wt b; do ids+=("$id"); wts+=("$wt"); bs+=("$b"); done
for ((i=0;i<${#ids[@]};i+=k)); do
  out="$KAB_RUNS/brief-group-${ids[i]}.md"; : > "$out"; n=0
  for ((j=i;j<i+k&&j<${#ids[@]};j++)); do n=$((n+1)); { echo "=== TASK $n of $k (issue #${ids[j]}) ==="; cat "${bs[j]}"; echo; } >> "$out"; done
  echo "You have $n tasks in one worktree: do them strictly in order, one commit per task (subject ends with its (#N)), then ONE report line per task. The per-task tool cap applies to each task separately." | cat - "$out" > "$out.tmp" && mv "$out.tmp" "$out"
  echo "${ids[i]} ${wts[i]} $out"
done
