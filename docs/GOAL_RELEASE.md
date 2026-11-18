# GOAL: release-ready Kabocha, verified and playable (limit 4000 chars)

Done means every item below is true and evidenced; stop only then.

1. Land the work. Commit all verified slices (conventional subject, no body, trailer by hook), merge upstream/master (`scripts/sync-upstream.sh`, fork must be 0 behind), resolve conflicts, push origin/master. git status clean, origin/master == master, only branch master, no extra worktrees (`scripts/clean-merged.sh`, `git worktree prune`).
2. Cleanup. No stray files in repo root, no index.lock, no scratch outputs, no unrelated lockfile/generated churn. Regenerate data only via `cargo run --locked -p pumpkin-codegen` (and `-- wit` when WIT changed); never hand-edit generated/.
3. Full compile and gates, all through `scripts/cargo.sh`, debug profile unless building release binaries: `cargo fmt --all --check`; `clippy --workspace --locked --all-targets --all-features -- -D warnings` exit 0; `test --workspace --locked --no-fail-fast` all pass (rerun a failure alone before calling it a bug); typos clean; Nix flake and release lints reproduce green locally where possible; `gh run list -R ArkFlame/Kabocha` green on the pushed head.
4. Deliver runtime binaries. Build the `pumpkin` binary in release for linux x86_64, copy to `dist/` with SHA256SUMS and the exact commit hash; verify it starts, loads the world, accepts a login and shuts down cleanly (`scripts/boot.sh`).
5. Gaps honesty. Regenerate with own-gaps.sh, parity.sh, gaps-fixed.sh, tracker-md.sh, ledger.sh. In docs/PARITY.tsv every row we truly fixed is marked done with a citation (path + vanilla file:line); rows not done stay open with a one-line reason (BLOCKED-DATA names the missing field or subsystem; client-only noted). No row is marked done on plausibility. TRACKER.md, DONE.md, TODO.md, README stats match the TSV.
6. Run it. Start the release server in the background with a fresh world, confirm the port listens and a status ping answers, then report to the user: public IP:port, LAN IP:port, game version, commit hash, and how to stop it. Keep it running.
7. Report. Final message: gates with real exit codes, counts of done / open / BLOCKED-DATA rows, what was skipped, known risks.

Rules: parallel native subagents up to 20, refill at once; workers never run cargo, the orchestrator batch-checks; Kabocha AGENTS.md rules apply (vanilla parity, data from pumpkin-data, no unwrap/panic, tests only where they fail without the fix).
