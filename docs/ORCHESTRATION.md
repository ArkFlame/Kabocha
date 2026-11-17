# Orchestration
## Capacity
Compile sparingly: workers run `check.sh <crate>` while iterating and `-f` once before handing back; docs/data-only slices skip builds.
12 cores, 38 GB RAM. 20 workers max (10 native + 10 OpenCode) but cap concurrent cargo builds near 6; the rest do reading/editing. Debug profile only. Use one `CARGO_TARGET_DIR` per worker class (e.g. `/tmp/kab-target-native`, `/tmp/kab-target-oc`) to avoid worktree-per-target disk blowup; set it in the brief.
## Worker lifecycle
`scripts/wt.sh new <name>` -> brief -> worker -> commit in worktree -> `scripts/check.sh -f <crate>` -> diff review -> `scripts/land.sh <name>`. A freed slot is refilled at once from docs/QUEUE.md.
## Native workers
Agent tool, `subagent_type: general`, brief starts `Role: CODER`. Launch several in one message for parallelism; they report when done. Tell each its worktree path and that it must not leave it.
## OpenCode workers
`opencode run -m commandcode/deepseek/deepseek-v4.1-flash --auto "<brief>"` run inside the worktree (wrapped by `scripts/oc-agent.sh`, background, logs in `/tmp/kabocha-runs`). Other DeepSeek providers returned insufficient-balance on 2026-11-16 (opencode-go, opencode, open-design); retest with a "pong" prompt. DeepSeek workers are good at: mechanical ports, data wiring, test cleanup, conflict resolution of small PRs. Give native workers: protocol, AI reach audits, subtle vanilla parity, large PRs.
## Brief template
```
Role: CODER
Worktree: <path>   (stay inside)
Task: <one deliverable>
Scope: <files/crates>
Evidence: <upstream issue/PR link, vanilla class, failing output>
Design: <snippets, wiring rows, names to reuse>
Acceptance: <commands that must exit 0, e.g. scripts/check.sh pumpkin-core>
Finish: commit `type(scope): subject` (no body, no trailers); report commands + exits and anything skipped.
```
## Failure handling
Worker fail #1: resend exact error. Fail #2: rewrite brief or take it yourself. Master red: stop landing, fix master. Balance error: reroute to native.
## Upstream sync
`scripts/sync-upstream.sh`; upstream is not merged wholesale, items are landed individually through the queue. GitHub API via `gh` is flaky: `scripts/upstream-queue.sh` retries.
## Standing rules (user, 2026-11-16)
- Always 10 native + 10 OpenCode workers running. OpenCode: `setsid nohup scripts/oc-loop.sh &` (check `pgrep -f oc-loop.sh`). Native: relaunch one per completion notification, same turn.
- Hourly upstream sync: cron job (re-create at session start: "7 * * * *" -> scripts/sync-upstream.sh, merge upstream/master, append to docs/SYNC.md). Fork was 3 commits behind at 53 ahead on 2026-11-16.
- Author trailers: Claude workers -> `Co-Authored-By: Claude Sonnet 5.5`, OpenCode -> `🤖 Generated with opencode` + `Co-Authored-By: opencode <noreply@opencode.ai>`; via scripts/hooks/commit-msg (core.hooksPath). Compare with `scripts/leaderboard.sh`.
- Skipped-as-already-present PRs go to docs/DONE.md as `- #N (PR, skipped: already present) title`.
