# Kabocha vs PumpkinMC: PR and issue disposition

Generated 2026-11-21 after a live audit of Pumpkin-MC/Pumpkin. Companion to docs/LEDGER.tsv, docs/OUTCOMES.tsv and docs/GAPS_FIXED.md (all generated; do not hand-edit).

## Method

1. gh pr list / gh issue list with --state open on Pumpkin-MC/Pumpkin gives the live open set (355 PRs, 407 issues).
2. Cross-reference against docs/LEDGER.tsv (all historical PRs) and docs/OUTCOMES.tsv (all historical issues).
3. Anything open upstream but absent from those files is a never-reviewed gap; triaged with scripts/pr-triage.sh (EMPTY / CLEAN / CONFLICT) or by code inspection for issues.
4. Kabocha master is fully synced with upstream/master (0 behind), so open upstream PRs are NOT in master and must be integrated explicitly.

## Live gap identified 2026-11-21: 33 PRs + 13 issues (numbers 4603-4665)

Status legend: LANDED = merged and pushed; IN FLIGHT = worker assigned; PENDING = queued; ALREADY-FIXED = same effect present on master with file:line proof.

### Pull requests

| PR | Title | Status |
|---|---|---|
| [#4605](https://github.com/Pumpkin-MC/Pumpkin/pull/4605) | fix(plugin): include players in get_nearby_entities | ALREADY-FIXED (#4294; v0_1 entity.rs:428, v0_2 entity.rs:388) |
| [#4606](https://github.com/Pumpkin-MC/Pumpkin/pull/4606) | feat(plugin): add v0.2 guest SDK | ALREADY-FIXED (plugin-api-v0_2) |
| [#4609](https://github.com/Pumpkin-MC/Pumpkin/pull/4609) | feat(wit): Redesign event to be a resource instead of a variant | ALREADY-FIXED (v0.2 event.wit:2318) |
| [#4611](https://github.com/Pumpkin-MC/Pumpkin/pull/4611) | fix(entity): let boats pick up nearby mobs | LANDED (4a9ab77b9) |
| [#4612](https://github.com/Pumpkin-MC/Pumpkin/pull/4612) | fix(util): always enable serde_json preserve_order | LANDED |
| [#4613](https://github.com/Pumpkin-MC/Pumpkin/pull/4613) | fix(server): save whitelist on and off to pumpkin.toml | LANDED (1f84bbc37) |
| [#4615](https://github.com/Pumpkin-MC/Pumpkin/pull/4615) | fix: keep text colors in bedrock translated system messages | LANDED |
| [#4619](https://github.com/Pumpkin-MC/Pumpkin/pull/4619) | perf(world): batch dependency GC after level changes | LANDED (e8bcc51e1) |
| [#4620](https://github.com/Pumpkin-MC/Pumpkin/pull/4620) | perf(world): send chunk level changes instead of the whole level map | LANDED (fdcb8bd87) |
| [#4622](https://github.com/Pumpkin-MC/Pumpkin/pull/4622) | fix(entity): align boat physics and minecart behavior with vanilla | ALREADY-FIXED (boat/minecart APIs) |
| [#4625](https://github.com/Pumpkin-MC/Pumpkin/pull/4625) | fix: show players on the locator bar | ALREADY-FIXED (commit 2b1bc82b4) |
| [#4626](https://github.com/Pumpkin-MC/Pumpkin/pull/4626) | fix(block): prevent opening a blocked shulker box | ALREADY-FIXED (shulker_box.rs:151) |
| [#4627](https://github.com/Pumpkin-MC/Pumpkin/pull/4627) | perf(world): compress chunks with zrip instead of ruzstd | LANDED (b908e11a6) |
| [#4628](https://github.com/Pumpkin-MC/Pumpkin/pull/4628) | feat(entity): vanilla parity for Breeze | BLOCKED (stale base, needs rewrite + codegen) |
| [#4630](https://github.com/Pumpkin-MC/Pumpkin/pull/4630) | feat(entity): vanilla parity for Creaking | LANDED (8413bfd89, partial: gaze/comparator) |
| [#4632](https://github.com/Pumpkin-MC/Pumpkin/pull/4632) | fix(world): preserve imported structure data when saving chunks | LANDED (b3fe50c73) |
| [#4634](https://github.com/Pumpkin-MC/Pumpkin/pull/4634) | Support deferred login admission | LANDED (cdcbdaca9) |
| [#4635](https://github.com/Pumpkin-MC/Pumpkin/pull/4635) | fix(entity): close remaining panda, dolphin, mooshroom and bee parity gaps | LANDED (bb4624ae5, partial: panda only) |
| [#4636](https://github.com/Pumpkin-MC/Pumpkin/pull/4636) | feat(entity): full vanilla parity for shulker | BLOCKED (stale-base redesign, needs build) |
| [#4638](https://github.com/Pumpkin-MC/Pumpkin/pull/4638) | perf(world): batch packets per tick and send movement from the entity tracker | LANDED (18ed0b039, partial: stages 1-2) |
| [#4639](https://github.com/Pumpkin-MC/Pumpkin/pull/4639) | fix(entity): despawn ineligible mobs on peaceful | ALREADY-FIXED core; remainder BLOCKED-DATA (codegen) |
| [#4641](https://github.com/Pumpkin-MC/Pumpkin/pull/4641) | Avoid double deletion of guest-owned resources | LANDED (f033a0d53) |
| [#4642](https://github.com/Pumpkin-MC/Pumpkin/pull/4642) | Expose transport latency through the v0.2 plugin API | LANDED (b9b41f8e4) |
| [#4643](https://github.com/Pumpkin-MC/Pumpkin/pull/4643) | fix(minecart) | BLOCKED (core fixes present, remainder needs generated APIs) |
| [#4644](https://github.com/Pumpkin-MC/Pumpkin/pull/4644) | fix(respawn_anchor): notify clients and neighbors when breaking anchor on explosion | LANDED |
| [#4647](https://github.com/Pumpkin-MC/Pumpkin/pull/4647) | fix(entity): bring leads closer to vanilla parity | LANDED (6ca8d115a, partial: leash targets) |
| [#4648](https://github.com/Pumpkin-MC/Pumpkin/pull/4648) | fix(entity): stop resistance from overflowing past level five | LANDED with issue #4649 (e7fea647f) |
| [#4650](https://github.com/Pumpkin-MC/Pumpkin/pull/4650) | fix(block): preserve drop flags when breaking multi-part blocks | LANDED |
| [#4652](https://github.com/Pumpkin-MC/Pumpkin/pull/4652) | feat(entity): sniffer vanilla parity | ALREADY-FIXED (#4535) |
| [#4655](https://github.com/Pumpkin-MC/Pumpkin/pull/4655) | feat(entity): implement happy ghast vanilla parity | ALREADY-FIXED (#4206) |
| [#4658](https://github.com/Pumpkin-MC/Pumpkin/pull/4658) | fix(plugin): separate PlayerCommandPreprocessEvent from PlayerCommandSendEvent | LANDED (9e5b74ca2, partial: event split) |
| [#4661](https://github.com/Pumpkin-MC/Pumpkin/pull/4661) | feat(entity): vanilla parity for armadillo | ALREADY-FIXED (armadillo.rs inline) |
| [#4663](https://github.com/Pumpkin-MC/Pumpkin/pull/4663) | feat(entity): port copper golem behavior and item transport | ALREADY-FIXED |

### Issues

| Issue | Title | Status |
|---|---|---|
| [#4603](https://github.com/Pumpkin-MC/Pumpkin/issues/4603) | TRACKER: Parity Integration Test | LANDED (64df933d7, 2 proof tests) |
| [#4623](https://github.com/Pumpkin-MC/Pumpkin/issues/4623) | Player Locator Bar does not show other players with locator_bar enabled | ALREADY-FIXED |
| [#4629](https://github.com/Pumpkin-MC/Pumpkin/issues/4629) | Cannot place a regular rail on top of a hopper | LANDED (e8a812cbb, proof test) |
| [#4640](https://github.com/Pumpkin-MC/Pumpkin/issues/4640) | wardem cannot attack any entity | LANDED (6c953b6e9, proof test) |
| [#4646](https://github.com/Pumpkin-MC/Pumpkin/issues/4646) | Bed despawn without dropping anything | ALREADY-FIXED (PR #4650 tests) |
| [#4649](https://github.com/Pumpkin-MC/Pumpkin/issues/4649) | Resistance 255 overflow crash on mob hit | LANDED (e7fea647f, proof test) |
| [#4651](https://github.com/Pumpkin-MC/Pumpkin/issues/4651) | 1.8.9 client Unable to locate sign via Velocity/ViaRewind | SKIPPED (ViaRewind proxy artifact) |
| [#4653](https://github.com/Pumpkin-MC/Pumpkin/issues/4653) | Firework rocket flight_duration ignored with Elytra | ALREADY-FIXED (firework_rocket.rs:37-42) |
| [#4654](https://github.com/Pumpkin-MC/Pumpkin/issues/4654) | Taming a horse grants unrelated advancements | BLOCKED-DATA (needs EntityCondition variant fields + codegen) |
| [#4657](https://github.com/Pumpkin-MC/Pumpkin/issues/4657) | Nether Portal generates at wrong height | BLOCKED (needs seeded-world runtime repro) |
| [#4662](https://github.com/Pumpkin-MC/Pumpkin/issues/4662) | Explosion Player Knockback | ALREADY-FIXED (b45b906cc) |
| [#4664](https://github.com/Pumpkin-MC/Pumpkin/issues/4664) | Burnable blocks catch fire when fire_spread_radius_around_player is 0 | LANDED (86e47eb98, proof test) |
| [#4665](https://github.com/Pumpkin-MC/Pumpkin/issues/4665) | WASM host v0.1 team color / play-sound / hide-player / gui.set-item bugs | LANDED (b11ccc8e8, byte-layout proof tests) |

## Already-counted upstream work (historical)

docs/GAPS_FIXED.md records the completed sweep: 365 PRs (286 fixed on master, 79 skipped) and 417 issues (231 fixed, 186 skipped), 0 open in that ledger. The table above is the new set published after that sweep; it is appended to the ledger as each item is resolved.

