---
name: kabocha-land
description: Verify and merge a finished worker branch into Kabocha master.
---
# Land
1. Worktree committed (conventional commit with scope, subject only, no body).
2. Read diff once: added lines vs vanilla/dup/magic numbers; removed lines (where did behaviour go?); other callers of changed shared fns.
3. `scripts/land.sh <name>` (rebase, check, ff-merge, push, cleanup). Conflict -> send worker the rebase task.
4. Update docs/QUEUE.md. Master must stay green; if `check.sh` fails on master, fix first, nothing else lands.
Never open PRs or post GitHub comments (upstream or fork).
