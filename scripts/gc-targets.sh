#!/usr/bin/env bash
# gc-targets.sh  -- delete private per-worker cargo targets and merged worktrees (disk is ~93% full). Keeps /tmp/kab-target-main.
rm -rf /tmp/kab-target-pr* /tmp/kab-target-n* /tmp/kab-target-c1 /tmp/kab-target-oc* 2>/dev/null
cd "$(dirname "$0")/.." && git worktree prune
df -h /tmp | tail -1
