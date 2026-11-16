#!/usr/bin/env bash
# pr-triage.sh <pr>...  -- mechanical triage, no agent needed. Prints "<pr> EMPTY|CLEAN|CONFLICT|CLOSED".
# EMPTY = master already contains the PR's effect (merge result tree == master tree). CLEAN = applies as is (batch-merge it).
# CONFLICT = needs a worker. CLOSED = no longer open upstream.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"
for pr in "$@"; do
  git fetch -q upstream "pull/$pr/head:refs/kab/pr-$pr" 2>/dev/null || { echo "$pr CLOSED"; continue; }
  if out=$(git merge-tree --write-tree master "refs/kab/pr-$pr" 2>/dev/null); then
    [ "$(echo "$out" | head -1)" = "$(git rev-parse master^{tree})" ] && echo "$pr EMPTY" || echo "$pr CLEAN"
  else echo "$pr CONFLICT"; fi
done
