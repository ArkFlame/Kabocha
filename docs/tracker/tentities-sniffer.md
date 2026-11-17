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
