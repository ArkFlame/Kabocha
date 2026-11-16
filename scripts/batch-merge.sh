#!/usr/bin/env bash
# batch-merge.sh <name> <pr>...  -- in worktree kab/<name>, apply each upstream PR as a squash commit onto master.
# Conflicting PRs are skipped and listed in $KAB_RUNS/<name>.conflicts for a worker. No compile here; run check.sh -f after.
source "$(dirname "$0")/lib.sh"; name="$1"; shift
wt="$KAB_WT/$name"; [ -d "$wt" ] || "$KAB_ROOT/scripts/wt.sh" new "$name" >/dev/null
: > "$KAB_RUNS/$name.conflicts"; : > "$KAB_RUNS/$name.merged"
for pr in "$@"; do
  git -C "$KAB_ROOT" fetch -q upstream "pull/$pr/head:refs/kab/pr-$pr" 2>/dev/null || { echo "$pr fetch-fail" >> "$KAB_RUNS/$name.conflicts"; continue; }
  title=$(gh pr view "$pr" -R Pumpkin-MC/Pumpkin --json title --jq .title 2>/dev/null | sed 's/ *🤖*$//' || true)
  if git -C "$wt" merge --squash "refs/kab/pr-$pr" >/dev/null 2>&1; then
    if git -C "$wt" diff --cached --quiet; then echo "$pr empty" >> "$KAB_RUNS/$name.conflicts"; continue; fi
    git -C "$wt" commit -qm "${title:-upstream PR $pr} (#$pr)" && echo "$pr" >> "$KAB_RUNS/$name.merged"
  else git -C "$wt" merge --abort 2>/dev/null || true; git -C "$wt" reset -q --hard HEAD || true; echo "$pr conflict" >> "$KAB_RUNS/$name.conflicts"; fi
done
echo "merged: $(wc -l < "$KAB_RUNS/$name.merged")  conflicts: $(wc -l < "$KAB_RUNS/$name.conflicts")"
