#!/usr/bin/env bash
# brief-pr.sh <pr>  -- write a self-contained, structured worker brief for landing upstream PR <pr>; prints brief path.
source "$(dirname "$0")/lib.sh"; pr="$1"
title=$(awk -F'\t' -v n="$pr" '$1==n{print $6}' "$KAB_ROOT/docs/queue-prs.tsv"); files=$(git -C "$KAB_ROOT" diff --name-only "master...refs/kab/pr-$pr" 2>/dev/null | head -12 | tr '\n' ' ')
f="$KAB_RUNS/brief-pr$pr.md"
cat > "$f" <<B
<role>You are a senior Rust engineer porting one upstream patch into a fork. You work alone, in one pass, with no chat context beyond this brief.</role>

<context>
Repo: Kabocha, a fork of the Pumpkin Minecraft server (Rust). Your current directory is your own git worktree; never leave it.
Upstream PR #$pr: "$title". Files it touches: $files
Code has been refactored since the PR was written: old paths like crates/pumpkin/src/... now live in crates/pumpkin-core/src/... Port the PR's INTENT to the current layout.
Read first (short): docs/WORKER.md. Do not read AGENTS.md.
</context>

<task>
1. \`git fetch upstream pull/$pr/head:refs/kab/pr-$pr\` (ok if it exists); \`git merge --squash refs/kab/pr-$pr\`.
2. If it conflicts: read the PR (\`gh pr view $pr -R Pumpkin-MC/Pumpkin\`, \`git diff master...refs/kab/pr-$pr\`), then resolve so the intent applies on top of current code. Prefer current names/APIs over the PR's.
3. If the intent is already in the code: \`git reset --hard\`, and report SKIPPED with file:line proof.
4. Search for every constructor/caller of anything you changed (struct field, signature) and update them in the same commit.
5. Review `git diff --cached`: delete stray fragments (lone `}`, `>`, orphan `#[test]`).
6. Commit: \`git add <only files you changed>\` then \`git commit -m "$title (#$pr)"\` (subject only, no body; the author trailer is added automatically).
</task>

<constraints>
- Do NOT run cargo, tests, clippy, or any build. Do not sleep, poll, or start background jobs. The orchestrator compiles in batches.
- Minimum change: no refactors, no new files unless the PR adds them, no formatting-only edits.
- No conflict markers may remain: \`grep -rn '<<<<<<<\|>>>>>>>' crates\` must be empty.
- Do not open PRs or comment on GitHub. Do not touch other worktrees.
</constraints>

<output>Reply with exactly one line, nothing else:
STATUS: merged|skipped|failed | PR #$pr | commit <hash or none> | files <list> | note <one short sentence>
Example: STATUS: merged | PR #1234 | commit ab12cd3 | files crates/pumpkin-core/src/entity/item.rs | note moved fix to pumpkin-core layout
</output>
B
echo "$f"
