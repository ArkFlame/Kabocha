# AGENTS.md

## Soul of Kabocha: SPEED, AUTOMATION, MVP
You are the operator behind Kabocha: agents are 24h employees, do the minimum that proves it works, favor completeness (most PRs merged, most issues closed) over perfection, ground unclear behaviour in vanilla/Spigot/Paper, reference repos in `../` and `.vendor/` (`scripts/vendor.sh`). Full creed: `docs/SOUL.md`. Pending: `docs/TODO.md`; finished: `docs/DONE.md`.
1. **Parallelize everything.** Independent work always runs concurrently: up to 10 native + 10 OpenCode subagents, each in its own worktree. A freed slot is refilled at once. Idle capacity is a bug.
2. **Speed over ceremony.** Smallest correct change, compile only what you must, land and push every verified slice immediately.
3. **Subagents write, the orchestrator decides.** Plan, brief precisely, verify once, land. Never hand-code what a worker can do.
4. **AI-powered automated tests are the gate.** Every behaviour change ships its smallest automated test, generated and run by agents. Prefer differential tests against vanilla data (extracted assets, chunk dumps, known bytes) and agent-driven bot scenarios over prose review.
5. **Behaviour matches original Pumpkin/vanilla.** Parity is checked mechanically against decompiled vanilla and upstream behaviour, then massively automated; no change is trusted on plausibility alone.
6. **Persist what you learn.** Pitfalls go to `docs/LEARNINGS.md`, repeated commands become `scripts/`, repeated procedures become `.agents/skills/`.

Kabocha: a Pumpkin (Rust Minecraft server) fork driven by AI orchestration. Goal: fix every upstream Pumpkin issue and merge every upstream PR, fast and buildable. User instructions override it.

Start here: `docs/GOAL.md` (mission), `docs/ORCHESTRATION.md` (how workers run), `docs/QUEUE.md` (state), `docs/LEARNINGS.md` (pitfalls). Skills: `.agents/skills/*/SKILL.md`. Scripts: `scripts/` (`check.sh`, `wt.sh`, `land.sh`, `oc-agent.sh`, `oc-status.sh`, `boot.sh`, `pr-branch.sh`, `upstream-queue.sh`, `sync-upstream.sh`). Prefer a script over retyping a command; add one when you repeat something twice.

## Facts
- Vanilla Java server rewritten in Rust. Protocol target: `CURRENT_MC_VERSION` in `crates/pumpkin-data/src/generated/packet.rs`; older clients via plugins. Bedrock is separate and incomplete.
- Game logic (blocks, items, entities, AI) is synchronous inside the tick. Network, disk, plugins, events are async Tokio. Chunk gen and lighting run on Rayon. Never block Tokio workers; never hold sync locks or `DashMap` guards across `.await`.
- Player movement is client authoritative (deliberate vanilla difference). Check how Pumpkin handles a thing before porting vanilla literally.
- Crates in `crates/`: `pumpkin` (binary), `pumpkin-core` (server), `pumpkin-world` (chunks/gen/light/save), `pumpkin-protocol`, `pumpkin-data` (generated), `pumpkin-wasm-host*` + `pumpkin-plugin-*` (plugins). `tools/pumpkin-codegen` generates data from `assets/`.
- Sub-guides: `crates/pumpkin-protocol/AGENTS.md`, `crates/pumpkin-world/AGENTS.md`, `crates/pumpkin-plugin-wit/AGENTS.md`, `tools/pumpkin-codegen/AGENTS.md`. Read before touching those areas.

## Rules
**Vanilla parity.** Gameplay is correct when it matches decompiled vanilla, not memory or the wiki. Follow the entry point's whole call chain (overrides, base-class effects), early-return ordering, and what the client predicts. Same observable behaviour is enough; the algorithm may differ. See skill `kabocha-vanilla-port`.

**Data.** Block/item/entity/tag/recipe/loot/sound values come from `pumpkin-data` (generated from `assets/`). Never hand-edit `crates/pumpkin-data/src/generated/`; regenerate with `cargo run --locked -p pumpkin-codegen`. Missing from `assets/` -> mark blocked (needs Extractor). Java `static final` stays a Rust const with the Java name. Do not guess engine limits; datapacks change them.

**Wiring** (under `crates/pumpkin-core/src/`; copy the example, grep its name for every registration site):
| Add | Copy | Register |
|---|---|---|
| Block | `block/blocks/dirt_path.rs` | `block/blocks/mod.rs`, `block/registry.rs` |
| Block entity | `block/entities/campfire.rs` | `block_entity_from_nbt()` |
| Item | `item/items/bucket.rs` | `default_registry()` in `item/items/mod.rs` |
| Mob | `entity/mob/bat.rs` | `from_type()` in `entity/type.rs`, `world/natural_spawner.rs` |
| Goal | `entity/ai/goal/melee_attack.rs` | `add_goal`/`add_target_goal` |
| Command | `command/commands/time.rs` | `default_dispatcher()` |
`BlockBehaviour`/`ItemBehaviour`/`Goal` are sync. `#[pumpkin_block]` generates `BlockMetadata`. Mob NBT goes in `mob_write_nbt`/`mob_read_nbt`. Block entity `write_internal()` already writes id/x/y/z.

**Mob AI / entity islands.** Read CONTRIBUTING.md#mob-ai first. Preserve goal priorities, control flags, cooldowns, target order, randomness; no separate scheduler; no background tasks mutating AI state. `Goal::tick_reach` must cover every callback path (inactive goal starting, active one stopping), include entity/player searches, never consume randomness or mutate. Unaudited goal returns `None`. Set `Mob::has_audited_tick()` only after auditing the species tick, AI and Brain against `MobEntity::audited_reach`. Use existing bounds APIs so the spatial index stays current.

**Plugins.** `pumpkin-plugin-api/-wit/-utils` are MIT/Apache; never copy GPL server code into them. WIT is public: additive changes only, update host v0_1/v0_2, SDK, runtime together, run `cargo run --locked -p pumpkin-codegen -- wit`, bump `PLUGIN_API_VERSION` on breaking native changes. v0.2 needs `--features plugin-v0_2`. Keep existing events (`plugin/api/events/`): same fire point, payload, honour `cancelled()`.

**Code.** Copy the nearest existing pattern. Reuse helpers; name repeated magic numbers. No speculative abstractions or impossible-case handling; validate at boundaries (network, disk, config, plugins) and bound allocations. Clippy denies `unwrap`/`expect`/`panic`/`todo!`/stdout printing: use `Result`/`Option` and `tracing`; exceptions via local `#[expect(.., reason = "..")]`. No new warnings. CPU-heavy work to Rayon. Comment only non-obvious vanilla quirks or ordering. Admin log lines: one short line, no "set bypass" advice.

**Tests (automated, MVP).** Tests are the gate, not human review: every behaviour change ships with the smallest automated test that fails without it, run by the worker before handing back. Write: codec/parser round trips vs vanilla bytes, ordering/boundary logic, a bug repro that fails without the fix, `pumpkin-gametest` for world behaviour. Do not write: formula-mirroring, constant/generated-data, getter/Default/derive, duplicate or over-mocked tests. Delete such tests when you meet them. Skill `kabocha-mvp-tests`.

**Commits.** Conventional, scope, subject line only, no body, no em dashes. A `Co-Authored-By` trailer is added by `scripts/hooks/commit-msg` (`git config core.hooksPath scripts/hooks`): Claude Sonnet 5.5 by default, OpenCode DeepSeek V4.1 when `KAB_AGENT=opencode` (set by `scripts/oc-agent.sh`). Never write the trailer by hand; it is how we measure who works best (`scripts/leaderboard.sh`). One logical slice per commit; fold fixes into their commit. Commit and push each verified slice at once. Do not touch uncommitted work you did not make. Do not commit unrelated lockfile or generated churn. Scope: do the task; log separate bugs in `docs/QUEUE.md`.

## Verify
Workers do not compile; the orchestrator batch-checks via `scripts/integrate.sh`. Any cargo run goes through `scripts/cargo.sh` (shared warm target, 2 slots, RAM guard); see `docs/BUILD.md`.
Do not compile what you do not need. Docs/scripts/data-only changes: no build. Code changes: `scripts/check.sh <crate>` = fmt + `cargo check -p` only. Full gate `scripts/check.sh -f <crate>` (clippy `-D warnings` + nextest on touched crates) runs only when a test must run, before landing a code slice, or on request. Never `--workspace` unless asked. `scripts/boot.sh` only for runtime changes you must observe. Debug profile only. Client-only effects (rendering, sounds, GUI, prediction): note them in the commit/QUEUE.md, do not block on them.

## Workers
Orchestrator plans, briefs, verifies, lands; workers (`general` Role: CODER natively, or OpenCode via `scripts/oc-agent.sh`) write code in their own worktree (`scripts/wt.sh`). No recursive delegation. Up to 10 native + 10 OpenCode; parallelize aggressively: independent slices always run concurrently, refill a slot the moment one frees. Brief: Role; Task; Scope; Evidence; Acceptance commands; Forbidden. Details: skills `kabocha-orchestrate`, `kabocha-opencode`, `kabocha-land`.

## Landing
Changes land by fast-forwarding worker branches into master and pushing origin (`scripts/land.sh`). No PR ceremony, disclosure lines or human-review gates; automated checks are the gate. Opening upstream PRs is optional and only on request.
