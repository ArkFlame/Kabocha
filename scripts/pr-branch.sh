#!/usr/bin/env bash
# pr-branch.sh <upstream-pr-number> <name>  -- fetch upstream PR head into worktree kab/<name> (rebased onto master if clean).
source "$(dirname "$0")/lib.sh"; pr="$1"; name="$2"
git -C "$KAB_ROOT" fetch upstream "pull/$pr/head:refs/kab/pr-$pr" 
wt=$("$KAB_ROOT/scripts/wt.sh" new "$name" "refs/kab/pr-$pr")
git -C "$wt" rebase master || { echo "CONFLICT: hand to a worker with the brief 'resolve rebase conflicts of PR #$pr'" >&2; git -C "$wt" rebase --abort; exit 3; }
echo "$wt"
