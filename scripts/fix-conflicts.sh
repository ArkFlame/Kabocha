#!/usr/bin/env bash
# fix-conflicts.sh <integrate.log> [max]  -- for every "CONFLICT kab/<w> <hash>" line, launch an OpenCode worker (in parallel,
# default 20) that rebases that branch onto master in its own worktree and resolves the conflicts, keeping both sides' intent.
source "$(dirname "$0")/lib.sh"; cd "$KAB_ROOT"; max="${2:-20}"
grep -E '^CONFLICT ' "$1" | awk '{print $2}' | sort -u | head -"$max" | while read -r b; do
  w=${b#kab/}; d="$KAB_WT/$w"; [ -d "$d" ] || { flock "$KAB_RUNS/wt.lock" scripts/wt.sh new "$w" >/dev/null 2>&1 || true; }
  f="$KAB_RUNS/brief-rebase-$w.md"
  cat > "$f" <<B
<role>You are a Rust engineer resolving git conflicts. Work alone, one pass.</role>
<context>Repo: Kabocha (Pumpkin fork). Your cwd is your own worktree on branch $b; never leave it. The branch has commits that no longer cherry-pick onto master.</context>
<task>
1. Run: git rebase master. For each conflict: read both sides, keep BOTH intents (master's refactors win for structure and names, port the branch's change onto them), remove all conflict markers, git add the files, git -c core.editor=true rebase --continue.
2. If the branch's change is already present on master after resolving (empty commit), run: git rebase --skip and report skipped.
3. Check: grep -rn '<<<<<<<\|>>>>>>>' crates must be empty. Keep the commit subject (it ends with (#N)); do not add a body.
</task>
<constraints>Do NOT run cargo or tests, no background jobs, no questions, about 25 tool calls max.</constraints>
<report>One line: STATUS: committed|skipped|failed | $b | commit <hash or none> | files <list> | compiled: no | note <one sentence></report>
B
  scripts/oc-agent.sh "$w" "$f" 9>&- 2>/dev/null | cut -c1-80 &
done
wait
