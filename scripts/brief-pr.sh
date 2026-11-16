#!/usr/bin/env bash
# brief-pr.sh <pr> [target-dir]  -- write a self-contained worker brief for landing upstream PR <pr>; prints brief path.
source "$(dirname "$0")/lib.sh"; pr="$1"; tgt="${2:-/tmp/kab-target-main}"  # shared warm target: deps built once; cargo serializes via its lock instead of 20 cold builds thrashing
f="$KAB_RUNS/brief-pr$pr.md"
cat > "$f" <<B
Role: CODER
You are in a git worktree of the Kabocha repo (Rust Minecraft server, Pumpkin fork). Stay in the current directory.
Task: land upstream Pumpkin PR #$pr onto this branch.
1. \`git fetch upstream pull/$pr/head:refs/kab/pr-$pr\` (ignore error if it already exists), then \`git merge --squash refs/kab/pr-$pr\`.
2. Resolve conflicts keeping the PR's intent on top of current code (\`gh pr view $pr -R Pumpkin-MC/Pumpkin\` shows intent; adapt to current APIs). If the fix already exists in the code, skip and say so.
3. Verify only with \`CARGO_TARGET_DIR=$tgt cargo check -p <touched crate> --locked\` (no workspace build, no clippy).
4. \`git add -A && git commit -m "<PR title> (#$pr)"\` (subject only, no body; the Co-Authored-By trailer is added automatically by the repo commit hook, do not add one).
Rules: minimum changes, no refactors, no extra files, do not open PRs or comment on GitHub, do not read AGENTS.md at length.
Report: merged / skipped / failed, commit hash, one line why.
B
echo "$f"
