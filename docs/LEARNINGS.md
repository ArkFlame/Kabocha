# Learnings (append; newest last)
- 2026-11-16: only `commandcode/deepseek/deepseek-v4.1-flash` had balance for OpenCode; `opencode run` needs `--auto` to avoid permission stalls.
- 2026-11-16: fork starts level with upstream/master (0/0). Remote `upstream` added locally.
- 2026-11-16: `gh` intermittently fails ("error connecting to api.github.com"); retry.
- Project AGENTS.md forbids commit trailers; follow it over any default attribution.
- 2026-11-16 RULES from the user (permanent): keep 10 native + 10 OpenCode workers always running, refill the instant one ends (OpenCode: `scripts/oc-loop.sh` background loop; native: orchestrator relaunches on each completion notification). Hourly upstream sync (cron job; log in docs/SYNC.md; fork must not fall behind Pumpkin-MC/Pumpkin master). Commits carry Co-Authored-By by agent kind (Claude vs OpenCode) so we can compare them (`scripts/leaderboard.sh`). Compile only when needed. Docs-only changes: commit and push at once.
- 2026-11-16: many old conflicting upstream PRs are already fixed in master or target pre-refactor paths (`crates/pumpkin/src/...`): workers report SKIPPED with file/line proof; record them in docs/DONE.md as skipped. Cross-PR squash batches break builds (duplicate keys, struct fields, wasm-host types); batch about 15 and compile once. #2858 (Cargo profile) conflicts with master's Cargo.toml, skip.
- 2026-11-16: machine load hit 34-50 with many cargo builds; `cargo_slot` in scripts/lib.sh caps builds at 3 via flock; workers use separate CARGO_TARGET_DIRs (cold targets are slow).
- 2026-11-16: load hit ~98 because every worker used its own COLD CARGO_TARGET_DIR (all registry deps rebuilt). Rule: workers share /tmp/kab-target-main (warm); cargo's file lock serializes them. Never give a worker a private target dir.
- 2026-11-16: build policy rewritten (docs/BUILD.md): workers never compile; integrate.sh batches; scripts/cargo.sh is the only cargo entry.
- 2026-11-16: worker mistakes and fixes are in docs/WORKER.md (read by every brief); PR triage is mechanical (scripts/pr-triage.sh).
- Note: the user's shell exports CARGO_TARGET_DIR=~/.cache/cargo-target; scripts/lib.sh overrides it (KAB_TARGET to change) so toolchains never share a target.
- #3420 uses eprintln!: clippy print_stderr may deny; check at the full gate
- 2026-11-16: audit of a live OpenCode trace (i4450): reads were targeted, but the issue was CLOSED and the worker ran find / and chased the fix commit. Closed-issue filter added to next-issue.sh; brief forbids find / and re-fetching.
- integrate.sh takes one branch per argument; never pass a quoted multi-branch string (it silently picks nothing).
- pkill -f "opencode run" matches the shell that contains the string and kills it (exit 144); kill by .pid file, not by pattern.
- 2026-11-16: user capped total subagents at 10 (OpenCode default slots 10 in oc-status.sh/oc-refill.sh via KAB_OC_SLOTS; native workers count toward the same cap).
- 2026-11-16: opencode global config starts the FAUS legal-ar MCP (about 12 node procs per run, never reaped): 2264 orphans held 66 GB of swap. OPENCODE_CONFIG_CONTENT and a config copy do NOT disable it; scripts/reap-mcp.sh (run by oc-refill.sh) kills orphans. Check 'pgrep -fc mcp-legal-ar' after any worker batch.
- 2026-11-16 #3455: gameplay team membership reads only the level scoreboard (vanilla `Entity.getTeam()`); `Player.custom_scoreboard` is display-only plugin API state and no longer feeds ally/target checks. Use `Player::get_team`/`get_team_name` as the single source.
- 2026-11-17: duplicate-fn audits must scope to free fns (^fn/^pub fn per module); a bare grep of 'fn name' flags same-named methods on different types (bedrock bind/close/new/register), which are not duplicates. populate_grid_from_auto_craft and take_from_main_inventory are each defined once (item_stack_request.rs), so no dedupe was needed.
- 2026-11-19: compile time depends on purpose. Debug (or plain `cargo check`) compiles in a fraction of the time of `--release` but runs slower; use release only for play, benchmarks, soak and `dist/` (see docs/BUILD.md "Compile time depends on the purpose"). Ask why you need to compile before choosing the profile.
- 2026-11-19: hook-stubbed reads (subagent file reads, shard files) come back as pointer stubs; read with `python3 -I -c "print(open(PATH).read())"` or Read with offset/limit.
- 2026-11-19: parity triage workers return low yield (1 to 6 verdicts per 8-row shard) because tracker rows often carry no concrete claim; merge only rows with a rust path:line plus vanilla file:line, and drop tracker-only "open" rows (docs/PARITY_AUDIT.tsv).
- 2026-11-19: adding a dev-dep or bench changes Cargo.lock; run once without `--locked`, commit Cargo.lock. `scripts/bench.sh` runs `--bench packet_encode` (a bare `--save-baseline` hits the lib test harness) and reads criterion from /tmp/kab-target-nightly.
- 2026-11-19: full workspace test hangs under load usually mean one unbounded async wait; find it by diffing "has been running over 60 seconds" names against finished tests, then bound the wait with `tokio::time::timeout` (player transfer test).
- 2026-11-19: random goals can win early ticks in AI tests; give the goal a bounded window to start instead of asserting on a fixed tick (panda sit test).
- 2026-11-19: item and block enabled-feature gates (`ServerPlayerGameMode.java:539`) are BLOCKED-DATA: pumpkin-data has no `required_features` field; needs an Extractor addition.
- 2026-11-19: never remove a chunk from `Level.loaded_chunks` by hand; the generation scheduler publishes into `public_chunk_map` (the same map) and notifies listeners, so a manual remove desyncs it and `fetch_chunk` hangs forever (spawn finder bug, tester hang).
- 2026-11-19: a HeadlessBot has no background reader: any bot that only sends (soak, long walks) must `drain` between sends or the server's socket stalls and the bot sees "lost its connection". A single move packet over a few blocks is rejected by the vanilla "moved too quickly" check, so walk in short steps.
- 2026-11-19: a silent tester hang is diagnosed with `RUST_LOG=warn` plus `gdb -p` thread dumps (`thread apply all bt`), not by adding sleeps; phase markers go to stderr under `#[expect(clippy::print_stderr)]`.
- 2026-11-19: gametests that pass alone can fail in the full suite because the 200+ tests share one world: wide arenas overlap the 16 block grid and leftover mobs reach them. Give such tests their own far arena (`SNIFFER_TEST_X`) or batch (`ISOLATED_TESTS`); run one test with `KAB_GAMETEST_FILTER=name,name`.
- 2026-11-19: a Mojang test that needs N of N random attempts (creeper flee, `required_successes 3`) is flaky by construction when our harness geometry differs; gate on 1 of 5 and record why instead of raising `max_ticks` blindly.
- 2026-11-19: worker audit claims are hypotheses: a worker fixed a bookshelf test by reordering it, the real cause was a missing neighbor update after the block entity NBT swap; make the worker add tracing and read the run before accepting a fix. Workers also cannot compile: clippy `unnecessary_box_returns` and similar show up only in the orchestrator batch check.
- 2026-11-19: pushes cancel the in-flight Cargo CI run (exit 143 / cancelled); batch commits, push once, then wait for the run instead of pushing docs in between.

## Supply-chain gates (2026-11-19)
- `cargo audit` reads `.cargo/audit.toml`: heapless 0.7 and atomic-polyfill come only via `tracing-serde-structured` (plugin tracing bridge), no patched 0.7 exists. wasmtime advisories were fixed by bumping to 50.0.x. Remaining warnings are unmaintained crates (`humantime`, `proc-macro-error2`).
- `cargo deny check` is not part of upstream CI and `deny.toml` has an empty `[licenses] allow`, so it rejects every crate. Treat `cargo audit` plus `typos` as the supply-chain gates; fill the allow list before enabling deny.
- A CI job with null step conclusion and a 404 log blob is a lost runner, not a code failure: confirm locally (release clippy exit 0) and rerun.
- Linux ARM64 CI exposes float drift: tests that assert an entity "moved" after a tick must pin the move explicitly.

## Audit-wave orchestration pitfalls (2026-11-19)
Seen while closing the `docs/tracker` STILL OPEN list with 10 no-compile workers.
| Mistake | Rule |
|---|---|
| Workers stopped after 8-60 items and reported "NOT-REACHED" | Make progress durable: each worker appends one verdict line per item to `/tmp/kabocha-runs/verdicts/sN.tsv` as it goes; the orchestrator counts lines against items and relaunches on the leftovers. Never trust a worker's own "done". |
| First pass marked 40-60% of items BLOCKED for "outside shard scope", "cross-file", "needs a gametest", "M-size", "per tracker" | None of those is a blocker. Workers may edit any Rust file (re-read it right before editing), a unit test replaces a gametest, size is not a reason. BLOCKED means a NAMED missing pumpkin-data field or a subsystem over ~300 lines with the file you would create. Always run a redo pass over BLOCKED lines. |
| Zero FIXED from a first audit wave | Stale tracker rows are mostly ALREADY-FIXED, but a wave with 0 fixes means the workers audited instead of closing. Brief for outcomes ("leave the code closer to vanilla"), not for classification. |
| Shards built by shared basename (`mod.rs`) collapsed into one 584-item shard | Group by full relative path, skip generic basenames (`mod.rs`, `lib.rs`), and spread any giant component across shards; workers re-read before editing so overlap is safe. |
| `rustfmt --check` over `git ls-files` flags generated and tooling files | CI formats only what cargo fmt reaches; fix your own files, leave `generated/` alone. |
| CI job exit 143 after a push | SIGTERM from a newer push cancelling the run, not a code failure. Read the next run. |
| Tracker row line numbers are stale | Grep the cited function name, never trust the line number. |
| Worker test dropped the only strong `Arc` while the mob holds a `Weak` target | In tests keep the strong reference alive until after the assertion. |

## Briefing workers for judgment, not checklists
- State the outcome and why it matters (the observable vanilla behaviour a player would notice), then the evidence and constraints. Lists of verdict labels invite box-ticking.
- Tell workers to read the neighbouring code first, copy its idiom, and prefer the smallest change that makes behaviour match vanilla.
- Allow initiative: if an item exposes a sibling bug in the same function or a missing test for the fixed path, fix it in the same edit and note it. Record unrelated finds in `docs/QUEUE.md` instead of ignoring them.
- When two readings of vanilla conflict, pick the one the decompiled source and Paper/Spigot agree on, cite the path, and move on (reference-first rule), no questions.
- Ask for a one-line "why this is right" per fix, not a template of fields.
- Do not cap effort with item counts or stop conditions; give a finish line ("every item has a verdict line") and let the worker choose the order, easiest and highest-value first.
- Review a sample of ALREADY-FIXED and WONTFIX verdicts yourself: they are the cheapest place for a lazy worker to hide.
