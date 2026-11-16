#!/usr/bin/env bash
# wt.sh new <name> [base=master] | rm <name> | ls   -- one worktree+branch (kab/<name>) per worker
source "$(dirname "$0")/lib.sh"
cmd="${1:-ls}"; name="${2:-}"; base="${3:-master}"
case "$cmd" in
  new) git -C "$KAB_ROOT" worktree add -b "kab/$name" "$KAB_WT/$name" "$base" >&2; echo "$KAB_WT/$name" ;;
  rm)  git -C "$KAB_ROOT" worktree remove --force "$KAB_WT/$name"; git -C "$KAB_ROOT" branch -D "kab/$name" ;;
  ls)  git -C "$KAB_ROOT" worktree list ;;
  *) echo "usage: wt.sh new|rm|ls <name> [base]" >&2; exit 2 ;;
esac
