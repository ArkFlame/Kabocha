# Build policy (why and how)
Findings (2026-11-16, 12 cores / 38 GB / disk 93% full):
- Each git worktree has its own path, so cargo rebuilds the workspace crates (pumpkin-core is the heavy one) per worktree; 20 workers meant 20 compiles. Private per-worker `CARGO_TARGET_DIR`s also rebuilt all registry deps (665 MB-1.3 GB each) and pushed load to ~98 and RAM to ~33 GB used.
- `~/.cargo/config.toml` sets `incremental = false` and a global target dir; no mold/lld/clang; sccache exists at `~/.cargo/bin/sccache` but path-keyed worktrees limit hits, so it is not enabled.
Rules (implemented in scripts/lib.sh):
1. **Workers do not compile.** They merge/resolve and commit. The orchestrator batches finished branches with `scripts/integrate.sh <branch>...` into the persistent worktree `int` and runs ONE check (incremental, warm). Failures are bisected by batch.
2. Anyone who must run cargo uses `scripts/cargo.sh`: shared `/tmp/kab-target-main`, 2 build slots (flock), `CARGO_BUILD_JOBS=nproc/2`, waits until MemAvailable >= 5 GB. Never a private target dir.
3. Prefer `cargo check -p <crate>`; no `--workspace`/clippy/tests until before landing a batch (`scripts/check.sh -f`).
4. `scripts/gc-targets.sh` removes private targets and prunes worktrees; run after landing.
5. Possible next wins: install mold + clang for linking (only matters for `cargo build/test`), nextest for tests, sccache for the registry-less workspace crates if worktree paths are normalized with `--remap-path-prefix`.
