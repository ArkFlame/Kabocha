#!/usr/bin/env bash
# integrate.sh <branch>...  -- cherry-pick finished worker branches onto the persistent worktree `int`, then ONE incremental check.
# `int` keeps its path, so incremental compilation + warm target make repeat runs cheap. Conflicts are skipped and reported.
source "$(dirname "$0")/lib.sh"; wt="$KAB_WT/int"
[ -d "$wt" ] || "$KAB_ROOT/scripts/wt.sh" new int >/dev/null
git -C "$wt" fetch -q . 2>/dev/null; git -C "$wt" reset -q --hard master
for b in "$@"; do
  for h in $(git -C "$KAB_ROOT" rev-list --reverse "master..$b"); do
    git -C "$wt" cherry-pick "$h" >/dev/null 2>&1 || { echo "CONFLICT $b $h"; git -C "$wt" cherry-pick --abort; break; }
  done; echo "picked $b"
done
cd "$wt" && CARGO_INCREMENTAL=1 "$KAB_ROOT/scripts/cargo.sh" check --workspace --locked 2>&1 | grep -E "^error|^\s+--> |Finished" | head -30
