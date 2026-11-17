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

## Nightly (enabled 2026-11-16, user-approved)
rustup + nightly (1.102) installed in ~/.cargo/bin with rustc-codegen-cranelift-preview and rust-src. scripts/lib.sh switches every script to it (PATH, RUSTUP_TOOLCHAIN=nightly, RUSTFLAGS `-Zthreads=6`, target /tmp/kab-target-nightly; `KAB_NIGHTLY=0` opts out). Measured: `cargo check -p pumpkin-util` 33 s cold with -Zthreads=6. Master needs rustc newer than system 1.97 (`map_or_default`), nightly satisfies it. For test/boot builds add `-Zcodegen-backend=cranelift` (`CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift` with `-Zcodegen-backend`) once verified; first nightly build of the workspace is cold (~10 min) then warm.

## Low-stack / mobile builds (#2861)
On aarch64 Termux/Android, rustc can SIGSEGV while compiling `cranelift-codegen`: the backtrace is in `rustc_parse` with "cycle encountered" and "recursed" frames, i.e. the parser overflows the process stack on cranelift's very large generated expressions. This is not codegen: `CARGO_BUILD_JOBS=1` and `RUST_MIN_STACK` (which only sizes spawned threads) do not help. Raise the shell stack limit for the build instead: `ulimit -s 65536` (or `ulimit -s unlimited` where the platform allows it), then `cargo run --release`.
