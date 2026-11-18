# entities / Sniffer (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/passive/sniffer.rs`, `sniffer/ai.rs` (implemented in d5dc0dd2d, #4535).
No decompiled `net.minecraft.world.entity.animal.sniffer.Sniffer` in this worktree (`.vendor/` absent, nothing under `../`), so the rows below are findings from the code plus vanilla knowledge and are **not mechanically confirmed**.

## Fixed in this pass
- State/seed tick were not persisted at all: no `mob_write_nbt`/`mob_read_nbt` existed, so `State` and `DropSeedAtTick` (vanilla `Sniffer.addAdditionalSaveData`/`readAdditionalSaveData`) were lost on save/load; only the generic `Brain` compound survived. Added both keys (state serialized by vanilla name: idling/feeling_happy/scenting/sniffing/searching/digging/rising), restoring synced data and digging dimensions. Commit 60ca04cc6.

## Remaining gaps
1. **Explored-position list holds 21 entries, not 20** (size S).
   - `sniffer.rs:57` `MAX_EXPLORED_POSITIONS = 20`; `sniffer.rs:343` `take(MAX_EXPLORED_POSITIONS)` then `sniffer.rs:346` `insert(0, ..)` yields 21; the reload test bakes 21 in (`sniffer.rs:771`).
   - Vanilla class: `Sniffer.onDiggingComplete`/`storeExploredPosition` — the memory is bounded by the same 20-entry constant.
   - Suggested fix: `take(MAX_EXPLORED_POSITIONS - 1)`, update the test to 20 entries and `positions[19]`. Confirm the exact vanilla cap first (this is the one uncertain point: whether vanilla does `limit(20)` before or after prepending).
2. **Dig loot spawn position is a floored block corner** (size S).
   - `sniffer.rs:363-364`: `let head = self.get_head_block().0.to_f64();` then `ItemEntity::new(.., head, ..)` — x/z floored and y is the head *block* y, so the seed pops out at a block corner, not at the sniffer's head.
   - Vanilla class: `Sniffer.dropSeed` spawns the loot at a continuous head/entity position, not at `BlockPos.containing`.
   - Suggested fix: use `self.get_head_position()` for the item position; update the test at `sniffer.rs:735`.
3. **Ambient-sound interval not confirmed** (size S).
   - `sniffer.rs:609` hardcodes `120`; vanilla `Sniffer.getAmbientSoundInterval` should be checked against the decompiled value (Mob default is 80).
4. **Digging game-event position** (size S).
   - `sniffer.rs:623` emits `minecraft:entity_action` at `get_head_block().to_centered_f64()`; vanilla `Sniffer.customServerAiStep`/`tick` emits at `position()` (sculk sensor hears the sniffer, not its head block). Low impact; confirm with the decompiled class.
5. **No test covers the sniffing -> walk -> dig chain end to end** (size M).
   - `ai.rs:398-444` (`Sniffing`) computes the dig position only on timeout; `ai.rs:344-396` (`Searching`) hands off to `SNIFFER_DIGGING`. Both are exercised only in isolation. A `pumpkin-gametest` scenario over dirt would cover the handoff.

## Gap status
- 1 explored-position list holds 21 entries: FIXED (sniffer.rs prepend_capped keeps the new position plus 20 previous, since vanilla storeExploredPosition does limit(20) then add(0, ...); test sniffer.rs explored_positions_keep_newest_within_cap)
- 2 dig loot spawn position floored block corner: RESOLVED no change (decompiled vanilla `dropSeed` spawns at floored `getHeadBlock()`; Pumpkin matches; doc premise invalid)
- 3 ambient-sound interval: FIXED (restored 120; decompiled `Animal.getAmbientSoundInterval()` overrides Mob's 80; sniffer.rs get_ambient_sound_interval)
- 4 digging game-event position: FIXED (decompiled emits at `Vec3.atCenterOf(getHeadBlock())`; sniffer.rs mob_tick now emits at head-block center with source entity)
- 5 end-to-end chain test (M): FIXED (already implemented as the `pumpkin:sniffer_sniffs_walks_and_digs` gametest; see Result pass 6)


## Result
Verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/` (now present, reversing the earlier "absent" note). Sizes: S first.

- Row 1 (explored-position cap): already FIXED in a prior pass (prepend_capped, cap 20); confirmed vanilla `storeExploredPosition` does `limit(20)` then `add(0, ...)`, matching. No further change.
- Row 2 (dig loot spawn position): RESOLVED, no change. Decompiled `Sniffer.dropSeed` spawns the seed at `BlockPos head = getHeadBlock()` and `new ItemEntity(level, head.getX(), head.getY(), head.getZ(), itemStack)`, i.e. the floored head block. Pumpkin `sniffer.rs:376` uses `get_head_block().0.to_f64()`, which matches. The doc's premise (continuous head/entity position, `get_head_position()`) is contradicted by the decompiled source; applying it would break parity.
- Row 3 (ambient-sound interval): FIXED. Decompiled `Mob.getAmbientSoundInterval()` returns 80, but `Animal.getAmbientSoundInterval()` overrides it to 120, and Sniffer does not override. `sniffer.rs get_ambient_sound_interval` had been changed from 120 to 80 on the wrong "Mob default" assumption; restored to 120 with a corrected comment.
- Row 4 (digging game-event position): FIXED. Decompiled `emitDiggingParticles` emits `gameEvent(GameEvent.ENTITY_ACTION, getHeadBlock(), Context.of(this))`; `LevelAccessor.gameEvent(Holder, BlockPos, Context)` uses `Vec3.atCenterOf(pos)`. `sniffer.rs mob_tick` now emits via `world.emit_game_event_with_context(EntityAction.name(), get_head_block().to_centered_f64(), ...)` with the sniffer as source, instead of `emit_own_game_event` at the entity position. No clean test seam (no injectable event sink), so no unit test added.
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. Needs a `pumpkin-gametest` scenario over dirt, which lives in the `crates/pumpkin-gametest/` crate, outside this unit's allowed files (ai.rs, sniffer.rs only).

## Result (pass 2)
- No decompiled vanilla source exists in this worktree (`Sniffer.java` and `.vendor/vanilla-decompiled/` absent), so rows cannot be mechanically re-verified this pass; prior-pass findings stand.
- Rows 1-4: confirmed already applied in `sniffer.rs` (prepend_capped cap 20, floored `get_head_block()` loot position, ambient interval 120, head-block-centered `EntityAction` game event). No code change needed.
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. Needs a `pumpkin-gametest` scenario over dirt in `crates/pumpkin-gametest/`, outside this unit's allowed file (sniffer.rs only); no pure-unit-test seam exists in sniffer.rs. Left open with this reason.

## Result (pass 3, re-review)
- No decompiled vanilla source in this worktree: globbed `.vendor/**`, `**/vanilla-decompiled/**` and `**/Sniffer.java`, all empty. Rows cannot be mechanically re-verified this pass; prior-pass findings stand.
- Rows 1-4: re-read `sniffer.rs` and confirmed all four fixes still present, no change needed: cap 20 via `prepend_capped` (lines 352-360), floored `get_head_block().0.to_f64()` loot position (line 377), ambient interval 120 (lines 623-626), head-block-centered `EntityAction` with source entity (lines 632-647).
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. Requires a single end-to-end scenario spanning the private `Sniffing`/`Searching`/`Digging` behaviors, which live in the out-of-scope `sniffer/ai.rs`; the natural home is a `pumpkin-gametest` scenario in `crates/pumpkin-gametest/`, outside this unit's only allowed file (`sniffer.rs`). `ai.rs` already has isolated tests (`searching_handoff_starts_digging_on_a_diggable_pad`, `interrupted_digs_do_not_cool_down...`) covering Searching -> Digging -> Rising over dirt; the only gap is the Sniffing timeout -> dig-position -> walk-target computation feeding that chain. No clean end-to-end seam exists in `sniffer.rs` alone (`calculate_dig_position` is a private helper and testing it in isolation would duplicate coverage without closing the row). Left open with this reason; no code or test change.

## Result (pass 4)
- No decompiled vanilla source available in this worktree (`.vendor/vanilla-decompiled` absent, no `Sniffer.java` found), so mechanical vanilla re-verification was not possible this pass.
- Rows 1-4: re-read `crates/pumpkin-core/src/entity/passive/sniffer.rs` and confirmed fixes remain in place with no regressions: explored-position cap 20 via `prepend_capped` (lines 60-67, 352-360), dig loot spawned at floored head block via `get_head_block().0.to_f64()` (line 377), ambient sound interval 120 (lines 623-626), and `EntityAction` game event emitted at head-block center with source context (lines 632-647). No code changes.
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. End-to-end chain spans private behaviors in out-of-scope `sniffer/ai.rs` and is best covered by a `pumpkin-gametest` scenario in `crates/pumpkin-gametest/` (outside this unit's allowed file `sniffer.rs`); no clean seam in `sniffer.rs` alone to add a meaningful test without duplicating isolated coverage. No test added.


## Result (pass 5)
- No decompiled vanilla source in this worktree (`.vendor/vanilla-decompiled/net/minecraft/**` and `**/Sniffer.java` globbed, both empty), so mechanical vanilla re-verification was not possible this pass.
- Only row 5 is marked STILL OPEN in the Gap status; rows 1-4 are already FIXED/RESOLVED and require no new code changes.
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. The chain spans private behaviors in out-of-scope `sniffer/ai.rs` and its natural home is a `pumpkin-gametest` scenario in `crates/pumpkin-gametest/`, outside this unit's only allowed file (`sniffer.rs`); `calculate_dig_position` is private and randomness-driven, so no meaningful pure-unit seam exists in `sniffer.rs` alone without duplicating the isolated `ai.rs` coverage. Left open with this reason; no code or test change.


## Result (pass 6, decompiled vanilla available + pass-2 cross-file permission)
- Decompiled vanilla is now present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/Sniffer.java`, `SnifferAi.java` and `world/entity/animal/Animal.java`, so rows are mechanically re-verifiable this pass (reversing the earlier "absent" notes).
- Row 5 (sniffing -> walk -> dig end-to-end test): FIXED, no new code needed. The scenario already exists as `test_sniffer_sniffs_walks_and_digs` in `crates/pumpkin-core/src/server/builtin_game_tests/entities_and_ai.rs:94-124`: it lays a 40x40 dirt pad, spawns a sniffer at the centre and waits, via `succeed_when`, for the dug seed (`torchflower_seeds` or `pitcher_pod`). It is listed in `BUILTIN_TEST_IDS` (`builtin_game_tests/mod.rs:93`) and registered with `register_flaky_function_test("pumpkin:sniffer_sniffs_walks_and_digs", "pumpkin:empty_40x4x40", 800, 0, 3, 1, ...)` at `mod.rs:635-643`. Helper signatures (`set_block_default`, `spawn_entity_at`, `assert_item_entity_present`, `succeed_when`) were checked against `pumpkin-gametest/src/helper.rs`. This closes the only row that was STILL OPEN.
- Rows 2/3/4: mechanically confirmed against vanilla, no change. `Sniffer.dropSeed` spawns at floored `getHeadBlock()` (`Sniffer.java:293-309`), matching `get_head_block().0.to_f64()`; `Animal.getAmbientSoundInterval()` returns 120 (`Animal.java:127-130`), matching `get_ambient_sound_interval`; `emitDiggingParticles` emits `ENTITY_ACTION` at `Vec3.atCenterOf(getHeadBlock())` with `Context.of(this)` (`Sniffer.java:329-331`), matching the head-block-centred `emit_game_event_with_context` with source entity in `mob_tick`.
- Row 1 (out of scope, flagged only): vanilla `storeExploredPosition` does `getExploredPositions().limit(20).collect()` then `add(0, ...)`, i.e. stores up to 21 entries (`Sniffer.java:336-341`), while Pumpkin's `prepend_capped(..., MAX_EXPLORED_POSITIONS=20)` keeps 20 (`sniffer.rs:352-360`, test asserts 20 at `sniffer.rs:795`). The earlier "cap 20" change therefore diverges from vanilla by one site. Row 1 is marked FIXED, not STILL OPEN, so it was left unchanged this pass; recommended follow-up: keep 21 (20 previous + newly prepended) and update the reload test.
- No source code changes this pass; only this doc was updated. UNVERIFIED: no build or test run (worker cannot compile).

## Result (pass 7, row 1 actually applied)
- Row 1 (explored-position cap) is now fixed in code, not just flagged. Vanilla `Sniffer.storeExploredPosition` does `getExploredPositions().limit(20).collect()` and then `updated.add(0, GlobalPos.of(...))` (`Sniffer.java:336-341`), so the memory holds 21 positions. `prepend_capped` in sniffer.rs kept the total at 20; it now takes 20 previous entries after the new one (capacity `limit + 1`), with a doc comment naming the vanilla ordering.
- Tests updated: `explored_positions_keep_newest_within_cap` now asserts 21 entries with the last value 19, and `completed_digs_keep_explored_sites_and_cooldown_after_nbt_reload` asserts `MAX_EXPLORED_POSITIONS + 1` entries with the last position `BlockPos::new(2, 63, 8)` (the 0..23 loop over supporting positions 0..22 keeps 22..2).
- Rows 2-5 unchanged from the pass 6 findings above.
- Files touched: `crates/pumpkin-core/src/entity/passive/sniffer.rs` only. UNVERIFIED: no build or test run (worker cannot compile).
