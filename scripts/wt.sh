#!/usr/bin/env bash
# wt.sh new <name> [base=master] | warm [N] | rm <name> | ls   -- one worktree+branch (kab/<name>) per worker.
# `new` takes a pre-built pool worktree (instant rename + incremental checkout) and falls back to a cold `worktree add`.
# `warm N` tops the pool up to N spare detached worktrees (.pool-*), so worker start never waits on a full checkout.
source "$(dirname "$0")/lib.sh"
cmd="${1:-ls}"; name="${2:-}"; base="${3:-master}"
case "$cmd" in
  new)
    spare=$(ls -d "$KAB_WT"/.pool-* 2>/dev/null | head -1)
    if [ -n "$spare" ] && mkdir "$spare.take" 2>/dev/null; then   # atomic claim
      if git -C "$KAB_ROOT" worktree move "$spare" "$KAB_WT/$name" >&2 2>&1 \
         && git -C "$KAB_WT/$name" checkout -q -B "kab/$name" "$base" >&2 2>&1; then rmdir "$spare.take"; echo "$KAB_WT/$name"; exit 0; fi
      rmdir "$spare.take" 2>/dev/null
    fi
    git -C "$KAB_ROOT" worktree add -b "kab/$name" "$KAB_WT/$name" "$base" >&2; echo "$KAB_WT/$name" ;;
  warm)
    exec 6>"$KAB_RUNS/warm.lock"; flock -n 6 || exit 0
    n="${name:-12}"
    while [ "$(ls -d "$KAB_WT"/.pool-* 2>/dev/null | grep -vc '\.take$')" -lt "$n" ]; do
      git -C "$KAB_ROOT" worktree add -q --detach "$KAB_WT/.pool-$(date +%s%N)" master >/dev/null 2>&1 || break
    done ;;
  rm)  git -C "$KAB_ROOT" worktree remove --force "$KAB_WT/$name"; git -C "$KAB_ROOT" branch -D "kab/$name" ;;
  ls)  git -C "$KAB_ROOT" worktree list ;;
  *) echo "usage: wt.sh new|warm|rm|ls <name> [base]" >&2; exit 2 ;;
esac
