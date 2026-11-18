# CI audit (ArkFlame/Kabocha vs Pumpkin-MC/Pumpkin)

Audited 2026-11-17/18 on master. Method: workflow YAML read, `gh run list/view` (read-only), `git diff upstream/master -- .github`, YAML and TOML parse, `uses:` pins checked with `gh api repos/<o>/<r>/tags`. No cargo was run (pipeline in progress). Local reproduction commands are listed, not executed.

Fork contains all of upstream master (0 commits behind), so every diff below is fork-local.

## Jobs

State: G = green on fork, R = red on fork (latest master run), S = skipped on fork, U = unverified.

| Workflow | Job | Trigger | Runner | What it checks | Secrets / permissions | Fork state | Local reproduction |
|---|---|---|---|---|---|---|---|
| rust.yml | prepare (Plan CI) | push master, PR, dispatch | ubuntu-latest | `ci_packages.py plan` (affected packages, full matrix on push) | contents read | G | `python3 .github/scripts/ci_packages.py --help` (plan needs cargo metadata) |
| rust.yml | format | has_code | ubuntu-latest | `cargo fmt --check` | none | R (3 diffs) | `scripts/cargo.sh fmt --check` |
| rust.yml | clippy | has_code | ubuntu-latest | `ci_packages.py run -- cargo clippy --all-targets --all-features` with `RUSTFLAGS=-Dwarnings` | none | R (unknown lint) | `scripts/cargo.sh clippy --workspace --locked --all-targets --all-features -- -D warnings` |
| rust.yml | clippy_release | push, run_full | ubuntu-latest | `cargo clippy --release --all-targets --all-features` | none | R (same lint) | `scripts/cargo.sh clippy --release --workspace --locked --all-targets --all-features -- -D warnings` |
| rust.yml | check_wit | WIT changed | ubuntu-slim | wit-bindgen + wasm-tools validate v0.1 and v0.2 | none | G (run 44224466341) | `wit-bindgen rust crates/pumpkin-plugin-wit/v0.2 --out-dir $TMP` then `wasm-tools validate` |
| rust.yml | test | has_code, format+clippy ok | ubuntu, windows, macOS (PR) / +ARM64 Linux/Windows (push) | `cargo nextest run --verbose --no-tests=pass` and `cargo test --doc` per affected package; installs `wasm32-wasip2` | none | U (never reached on fork: upstream lints red) | `scripts/cargo.sh nextest run --workspace --locked --no-tests=pass`; `scripts/cargo.sh test --workspace --locked --doc` |
| rust.yml | gametest | has_code | ubuntu-latest | `cargo run -p pumpkin-tester` (headless bots, Mojang GameTests) | none | U (never reached) | `scripts/cargo.sh run --locked -p pumpkin-tester` |
| rust.yml | test_android | run_full (push) | ubuntu-latest + KVM emulator | `cargo ndk -t arm64-v8a test --workspace` on API 35 emulator | KVM via udev rule | U (never reached) | needs Android NDK and emulator; not reproducible locally |
| rust.yml | build_pull_request | PR | Linux x64/ARM64, Windows x64/ARM64, macOS | `cargo build --verbose` affected packages | none | S (PR only) | `scripts/cargo.sh build --locked` |
| rust.yml | build_android_pull_request | PR | ubuntu-latest | `cargo ndk build --locked` affected | none | S (PR only) | needs NDK |
| rust.yml | build_release | push, run_full | 5 OS | `cargo build --release`, upload binaries | none | U (blocked by lints) | `scripts/cargo.sh build --release --locked` |
| rust.yml | build_android_release | push, run_full | ubuntu-latest | `cargo ndk build --release --locked` | none | U (blocked by lints) | needs NDK |
| rust.yml | draft_canary_release | push master | ubuntu-latest | tags `canary`, drafts prerelease (contents write) | GITHUB_TOKEN | S (blocked by lints) | left ungated, see changes |
| typos.yml | spelling | push, PR | ubuntu-latest | `crate-ci/typos@v1.52.2` with `typos.toml` | none | G (runs 44224466285, 44217060156) | `typos` (not installed here) |
| nix.yml | check-flake | push master (paths), PR | ubuntu-latest | `nix flake check`, `nix build .#default` | magic-nix-cache (FlakeHub, non-fatal) | R at 00:48 run (Rust compile errors, see code failures); latest 44224466320 in progress at audit time | `nix flake check` |
| ferrflow.yml | version (Versioning) | push master | ubuntu-latest | FerrFlow version and tag | id-token write | S (repo gate) | n/a |
| nightly.yml | check, build, build_android, draft_release | schedule 03:00 UTC, dispatch | 5 OS + Android | nightly binaries and `nightly` tag | contents write | S (repo gate) | n/a |
| docker.yml | docker-build-deploy, merge | workflow_run of Nightly/Release, push paths, dispatch | ubuntu, arm64 | multi-arch GHCR image | packages/id-token write | S (repo gate); older fork runs had no jobs at sha 8901989 (startup failure) | n/a |
| release.yml | build_release, build_android_release, build_arm32_release | tag push | 5 OS + Android + ARM32 | release binaries | none | U (blocked by lints) | `scripts/cargo.sh build --release --locked` |
| release.yml | sign_windows_release | tag push | windows | SignPath signing | `SIGNPATH_*` secrets | S (repo gate, was failing without secrets) | n/a |
| release.yml | publish_release | tag push | ubuntu-latest | GitHub release with checksums | contents write | S (repo gate) | n/a |
| sync-wit.yml | sync-wit | push master (wit paths), dispatch | ubuntu-slim | subtree push to `pumpkin-plugin-wit` mirror | `BOT_APP_CLIENT_ID` var, `BOT_APP_PRIVATE_KEY` secret | S (repo gate; was red with empty client-id, run 44207855376) | n/a |

Non-workflow config:
- `rust-toolchain.toml` channel `stable`. `rustfmt.toml` present. `typos.toml` present. `clippy.toml` absent. `deny.toml` absent, so **cargo deny is not run anywhere**.
- Dependabot: github-actions, docker, devcontainers, weekly. Parses.
- Issue templates: `bug_report.yml`, `feature_request.yml`. Not semantically reviewed.
- Local YAML parse (`yaml.safe_load`) of every `.github/**/*.yml`: pass. TOML parse of the three config files: pass.
- Every `uses:` pin resolves via the GitHub tags API: checkout@v7, setup-python@v7, upload-artifact@v7, download-artifact@v8, create-github-app-token@v3, rust-cache@v2, nttld/setup-ndk@v1, android-emulator-runner@v2, softprops/action-gh-release@v3, signpath@v3, nix-installer-action@v23, magic-nix-cache-action@v15, docker/* (v4/v6/v7), FerrFlow@v7, typos v1.52.2, bytecodealliance/actions@v1. `dtolnay/rust-toolchain@stable` is a branch. `taiki-e/install-action@nextest` and `actionlint` were not checked.
- Every `ci_packages.py` subcommand used (`plan`, `run -- cargo clippy|nextest run|test --doc|build|ndk build`) is supported by `parse_args`/`cargo_package_insert_index`. Push to master selects the full package set and full matrix.

## Fork-environment fixes applied

| File | Change | Reason |
|---|---|---|
| `.github/workflows/sync-wit.yml` | job `sync-wit` gets `if: github.repository == 'Pumpkin-MC/Pumpkin'` | Fork lacks `BOT_APP_CLIENT_ID` and `BOT_APP_PRIVATE_KEY`; run failed at token step |
| `.github/workflows/release.yml` | `sign_windows_release` and `publish_release` get the same gate | SignPath secrets and release publish are upstream-only; sign would fail on fork tags |
| `.github/scripts/ci_packages.py` | `.github/workflows/ferrflow.yml` added to `RUST_CI_IGNORED_FILES` | Fork-only workflow change would otherwise trigger a full Rust rebuild (unmapped path). Verified: `select_packages` returns no packages for it |

Checks kept intact: fmt, clippy (affected and release), check_wit, test (nextest and doctests), gametest, test_android, build_pull_request, build_android_pull_request, build_release, build_android_release, typos, nix. No check was deleted or loosened.

## Fork drift from upstream (`git diff upstream/master -- .github`)

| File | Drift | Verdict |
|---|---|---|
| rust.yml | `rustup target add wasm32-wasip2` in test job (v0.2 guest fixture) | Intentional, keep. Upstream lacks it, so the test job may fail upstream if the fixture is absent there |
| ci_packages.py | `pumpkin-plugin-api-v0_2` added to `WIT_CONSUMER_PACKAGES` | Intentional, keep. Package exists in workspace |
| docker.yml | ARM v7 platform, QEMU step, `enable=` on version tags | Intentional fork change, keep. Only runs upstream |
| nix.yml | `nix-installer-action` v22 to v23 | Pin bump, keep |
| release.yml | signpath v2 to v3; `build_arm32_release` job; publish needs it | Intentional, keep. Gated now |
| ferrflow.yml | new file, repo-gated | Fork-only, keep |

## Code-level CI failures still open (for the orchestrator)

1. **Lint unknown on stable (`clippy` and release clippy, run 44224466341).** `error: unknown lint: clippy::large_unopt_stack_frames` at `crates/pumpkin-data/src/generated/configured_features_generated.rs:6`. Source is `tools/pumpkin-codegen/src/configured_feature.rs:146`. Upstream uses `clippy::large_stack_frames` in both places. The fork renamed it. Fix: revert the name in codegen, then regenerate with `cargo run --locked -p pumpkin-codegen`. Do not edit the generated file by hand.
2. **`large_stack_frames` after that fix** (same run): `this function may allocate 921885 bytes on the stack` at `configured_features_generated.rs:9`. The allow list must include the real lint name. Upstream's allow list has it.
3. **rustfmt (run 44224466341, Check formatting):** `crates/pumpkin-core/src/data/datapack/mod.rs` at 1957 (`is_none_or` chain), 1965 (closing brace), 3744 (`assert_eq!` line too long). Reproduced read-only with `rustfmt --edition 2024 --check <file>`. Fix: `rustfmt --edition 2024 crates/pumpkin-core/src/data/datapack/mod.rs`.
4. **WIT v0.2 `event.wit`:** 836 lines removed vs upstream, including `player-*-event-data` records. AGENTS requires WIT changes to be additive, and existing events must keep payloads. Orchestrator must confirm this is an intended v0.2 rewrite. v0.1 has 4 removed lines to recheck. Not a CI failure, but a merge-quality blocker.
5. **Nix (run 44210767724, 00:48):** `pumpkin-core` lib had 122 compile errors (E0505, E0277, E0308, E0425...). That commit no longer matches HEAD. Latest run 44224466320 was still in progress; re-check its result.
6. **Tests and gametest never ran green on fork** (last 100 Cargo runs: 77 cancelled, 23 failure, 0 success). Their state is unknown until items 1-3 are fixed.

## Unverifiable without a build

- Test, nextest, doctest, gametest, Android, and release build results.
- Whether `cargo fmt` and clippy are clean after the lint rename.
- `typos` (not installed; `typos.toml` parses).
- `actionlint` (not installed; only PyYAML parse was done).
- `cargo deny`: no `deny.toml` and no job, so there is no check to run.
- Nix flake evaluation.
- Remaining `docker.yml` startup failure at sha 8901989 (current file has the repo gate and parses).

## Pumpkin merge quality checklist

| Item | Requirement source | Result | Evidence |
|---|---|---|---|
| Conventional commits | PR template, AGENTS | PASS | last 20 subjects match `type(scope): subject`; no em dashes; `scripts/hooks/commit-msg` present |
| Formatting | CI format job | FAIL | 3 rustfmt diffs (item 3) |
| Clippy `-D warnings` | CI clippy job | FAIL | unknown lint and large stack frame (items 1-2) |
| Tests (nextest, doctests, gametest) | CI test and gametest jobs | UNVERIFIED | never green on fork |
| No `unwrap`/`expect`/`panic` in non-test code | AGENTS (clippy denies) | UNVERIFIED | rough grep of `crates/` outside `generated`/`tests` gives about 3265 hits, but counts include test modules and inline `#[cfg(test)]`. Clippy deny is the real gate |
| Docs | CONTRIBUTING.md | PASS (exists) | content not judged |
| Plugin API stability | AGENTS WIT rules | FAIL (needs review) | v0.2 `event.wit` 836 lines removed vs upstream (item 4) |
| Spelling | CI typos job | PASS on fork | runs 44224466285 and 44217060156 succeeded |
| Workflow YAML validity | CI | PASS | PyYAML safe_load on all `.github/**/*.yml` |
| Action pins exist | CI | PASS | GitHub tags API (see above) |

## Orchestrator notes

- Item 1-2 fixed: codegen and generated file now allow both `clippy::large_stack_frames` and `clippy::large_unopt_stack_frames` (plus `unknown_lints`, `renamed_and_removed_lints`) so stable and nightly both pass.
- Item 4: the v0.2 `event.wit` rework is upstream PR #4405 (variant to resource), merged here on purpose; precedent is upstream's own open PR, not a fork rewrite.
