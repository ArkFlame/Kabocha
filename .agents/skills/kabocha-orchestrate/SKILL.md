---
name: kabocha-orchestrate
description: Run the Kabocha loop as orchestrator: queue, brief, launch 20 workers, verify, land. Use at session start and whenever a worker finishes.
---
# Orchestrate
1. `scripts/sync-upstream.sh`; read docs/QUEUE.md, docs/LEARNINGS.md. Seed queue with `scripts/upstream-queue.sh prs|issues` if empty.
2. Slots: 10 native (`general`, Role: CODER) + 10 OpenCode (`scripts/oc-agent.sh`). `scripts/oc-status.sh` shows free OpenCode slots.
3. Pick items by priority (docs/GOAL.md). Never give two workers overlapping files; one worktree each (`scripts/wt.sh new`). For upstream PRs start from `scripts/pr-branch.sh <n> <name>`.
4. Brief = Role; Task; Scope files; Evidence; Acceptance commands; Forbidden. Include code snippets and the wiring table row (AGENTS.md).
5. Worker ends -> run `scripts/check.sh` in its worktree -> read diff once -> `scripts/land.sh <name>` -> update QUEUE.md -> launch next item immediately.
6. Fail twice -> rewrite brief or do it yourself. Record any new pitfall in docs/LEARNINGS.md.
Native workers go in foreground parallel batches in one message; OpenCode run in background.
- Mandatory: workers read docs/WORKER.md (no cargo, no polling, commit before reporting, stage only own files, report once). Triage first with `scripts/pr-triage.sh <pr>` (EMPTY/CLEAN/CONFLICT/CLOSED); only CONFLICT goes to a worker, CLEAN goes to `scripts/batch-merge.sh`.
