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
- 1 explored-position list holds 21 entries: FIXED (crates/pumpkin-core/src/entity/passive/sniffer.rs:341-345 via prepend_capped, cap 20 total; test sniffer.rs explored_positions_keep_newest_within_cap; UNVERIFIED: not compiled)
- 2 dig loot spawn position floored block corner: RESOLVED no change (decompiled vanilla `dropSeed` spawns at floored `getHeadBlock()`; Pumpkin matches; doc premise invalid)
- 3 ambient-sound interval: FIXED (restored 120; decompiled `Animal.getAmbientSoundInterval()` overrides Mob's 80; sniffer.rs get_ambient_sound_interval)
- 4 digging game-event position: FIXED (decompiled emits at `Vec3.atCenterOf(getHeadBlock())`; sniffer.rs mob_tick now emits at head-block center with source entity)
- 5 end-to-end chain test (M): STILL OPEN (needs gametest in crates/pumpkin-gametest/, outside this unit's files)


## Result
Verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/` (now present, reversing the earlier "absent" note). Sizes: S first.

- Row 1 (explored-position cap): already FIXED in a prior pass (prepend_capped, cap 20); confirmed vanilla `storeExploredPosition` does `limit(20)` then `add(0, ...)`, matching. No further change.
- Row 2 (dig loot spawn position): RESOLVED, no change. Decompiled `Sniffer.dropSeed` spawns the seed at `BlockPos head = getHeadBlock()` and `new ItemEntity(level, head.getX(), head.getY(), head.getZ(), itemStack)`, i.e. the floored head block. Pumpkin `sniffer.rs:376` uses `get_head_block().0.to_f64()`, which matches. The doc's premise (continuous head/entity position, `get_head_position()`) is contradicted by the decompiled source; applying it would break parity.
- Row 3 (ambient-sound interval): FIXED. Decompiled `Mob.getAmbientSoundInterval()` returns 80, but `Animal.getAmbientSoundInterval()` overrides it to 120, and Sniffer does not override. `sniffer.rs get_ambient_sound_interval` had been changed from 120 to 80 on the wrong "Mob default" assumption; restored to 120 with a corrected comment.
- Row 4 (digging game-event position): FIXED. Decompiled `emitDiggingParticles` emits `gameEvent(GameEvent.ENTITY_ACTION, getHeadBlock(), Context.of(this))`; `LevelAccessor.gameEvent(Holder, BlockPos, Context)` uses `Vec3.atCenterOf(pos)`. `sniffer.rs mob_tick` now emits via `world.emit_game_event_with_context(EntityAction.name(), get_head_block().to_centered_f64(), ...)` with the sniffer as source, instead of `emit_own_game_event` at the entity position. No clean test seam (no injectable event sink), so no unit test added.
- Row 5 (sniffing -> walk -> dig end-to-end test, M): STILL OPEN. Needs a `pumpkin-gametest` scenario over dirt, which lives in the `crates/pumpkin-gametest/` crate, outside this unit's allowed files (ai.rs, sniffer.rs only).
