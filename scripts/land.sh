#!/usr/bin/env bash
# land.sh <name> [--no-push]  -- rebase kab/<name> on master, verify, fast-forward master, push, drop worktree.
source "$(dirname "$0")/lib.sh"; name="$1"; push=1; [ "${2:-}" = "--no-push" ] && push=0
wt="$KAB_WT/$name"
git -C "$wt" status --porcelain | grep -q . && { echo "worktree dirty: commit first" >&2; exit 1; }
git -C "$wt" rebase master
( cd "$wt" && KAB_DIR="$wt" "$KAB_ROOT/scripts/check.sh" -f )
git -C "$KAB_ROOT" merge --ff-only "kab/$name"
[ $push = 1 ] && git -C "$KAB_ROOT" push origin master
"$KAB_ROOT/scripts/wt.sh" rm "$name"
