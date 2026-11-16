---
name: kabocha-opencode
description: Launch and track OpenCode DeepSeek background workers.
---
# OpenCode workers
- Model: `commandcode/deepseek/deepseek-v4.1-flash` (verified working 2026-11-16). opencode-go, opencode/, open-design providers returned insufficient balance; re-test with a "pong" prompt before relying on them.
- Start: write brief to a file, `scripts/oc-agent.sh <name> <brief.md>`; status: `scripts/oc-status.sh`; results `/tmp/kabocha-runs/<name>.{out,log,exit}`.
- Max 10 concurrent. Flag `--auto` approves permissions: only run inside the worker worktree. Timeout default 1800s (`KAB_OC_TIMEOUT`).
- Briefs must be self-contained (worker has no chat context) and must say: no PRs, no gh comments, no files outside scope, run the acceptance commands, commit with conventional message.
- BALANCE in status -> route work to native workers and note in docs/LEARNINGS.md.
