# entities / Sniffer (RE-REVIEW)

Source: `crates/pumpkin-core/src/entity/passive/sniffer.rs`, `sniffer/ai.rs`.
Prior state: implemented in d5dc0dd2d (#4535), NBT persistence added in 8b5939e69.
No decompiled `Sniffer` class in this worktree (`.vendor/` empty in the parent repo, nothing under `../`), so rows are code findings plus vanilla knowledge, **not mechanically confirmed**.

## Fixed in this pass
1. **Explored positions kept 21 entries, vanilla cap is 20.**
   `on_digging_complete` did `take(MAX_EXPLORED_POSITIONS)` then `insert(0, ..)`, so the memory grew to 21 (test baked it in). Now `take(MAX_EXPLORED_POSITIONS - 1)`; test asserts 20 and index 19. Vanilla `Sniffer.storeExploredPosition` prepends then trims to 20. Commit c01aaaf14.
2. **Dig loot spawned at a floored block corner.**
   `drop_seed` used `get_head_block().0.to_f64()` (integer x/y/z), so the seed popped out of the block corner. Vanilla `Sniffer.dropSeed` spawns at `Vec3.atBottomCenterOf(this.getHeadBlock())`. Now block centre at head y; test updated. Commit 463f4cb28.

## Remaining gaps
1. **Digging game-event position** (S).
   - `sniffer.rs:629` emits `minecraft:entity_action` at `get_head_block().to_centered_f64()`.
   - Vanilla class: `Sniffer.tick` — emits `GameEvent.ENTITY_ACTION` at `this.position()` (entity feet), so the sculk sensor hears the sniffer body, not its head block.
   - Suggested fix: pass `entity.pos.load()` and update nothing else. Confirm with the decompiled class first; low gameplay impact.
2. **`canDig` missing the leash check** (S).
   - `sniffer.rs:273-282` checks panicking, tempting, baby, water, ground, vehicle, and `can_dig_at`, but not `is_leashed()`; `can_sniff` (`sniffer.rs:262-271`) does include it.
   - Vanilla class: `Sniffer.canDig` — the vanilla condition list should be compared line by line; if `!this.isLeashed()` is present, a leashed sniffer can dig here.
   - Suggested fix: add `&& !entity.is_leashed()` to `can_dig`.
3. **Ambient-sound interval not confirmed** (S).
   - `sniffer.rs:615-617` hardcodes `120`; Mob default is 80.
   - Vanilla class: `Sniffer.getAmbientSoundInterval` — confirm the decompiled value (or that it is not overridden).
4. **`DropSeedAtTick` is persisted but vanilla saves only `State`** (S).
   - `sniffer.rs:643-648` writes/reads `DropSeedAtTick`; vanilla `Sniffer.addAdditionalSaveData`/`readAdditionalSaveData` only store `State`, so a reloaded vanilla sniffer drops no seed until it digs again. Deliberate deviation (keeps an in-flight dig visual working); document or drop it.
5. **No end-to-end sniff -> walk -> dig test** (M).
   - `Searching` (`ai.rs:342-396`) and `Sniffing` (`ai.rs:398-444`) handoff is only exercised per-behaviour; add a `pumpkin-gametest` over dirt covering Scenting -> Sniffing -> walk -> Digging -> seed drop.
6. **`can_mate` does not call the base check** (S).
   - `sniffer.rs:421-426` only checks the two states; vanilla `Sniffer.canMate` also calls `super.canMate(other)` (ageable/animal check). Verify whether the breeding behaviour already enforces the same precondition; if not, route through `Animal::can_mate`.
