#!/usr/bin/env bash
# clean-merged.sh -- remove worktrees + branches of kab/* branches already merged into master. Skips running workers, the int
# worktree, and dirty worktrees (no --force). Also prunes dead worktree records and old run files of removed workers.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
running=$(scripts/oc-status.sh | awk '$2=="running"{print $1}'); n=0
for b in $(git branch --merged master --format='%(refname:short)' --list 'kab/*'); do
  w=${b#kab/}; [ "$w" = int ] && continue; echo "$running" | grep -qx "$w" && continue
  d="$KAB_WT/$w"; [ -d "$d" ] && { git worktree remove "$d" >/dev/null 2>&1 || continue; }
  git branch -d "$b" >/dev/null 2>&1 && { n=$((n+1)); rm -f "$KAB_RUNS/$w".{log,out,pid,exit} "$KAB_RUNS/brief-"*"${w#[a-z]}.md" 2>/dev/null; }
done
git worktree prune; echo "removed $n merged worktrees/branches; remaining worktrees: $(git worktree list | wc -l)"
