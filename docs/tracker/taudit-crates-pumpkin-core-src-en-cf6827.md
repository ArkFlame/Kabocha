# audit / crates/pumpkin-core/src/entity/decoration/armor_stand.rs

Source: `crates/pumpkin-core/src/entity/decoration/armor_stand.rs`. No decompiled `net.minecraft.world.entity.decoration.ArmorStand` in this worktree (`.vendor/` absent), so remaining rows are code findings plus vanilla knowledge, not mechanically confirmed.

## Fixed in this pass
- `kill()` did not emit `GameEvent.ENTITY_DIE` (worker TODO at line 531); vanilla `ArmorStand.kill` removes then `gameEvent(GameEvent.ENTITY_DIE)`.
- Breaking a stand played `entity.armor_stand.break` twice: once explicitly in `damage_with_context` and again inside `broken_by_anything`.
- `interact_at_position` ignored spectators; vanilla `ArmorStand.interactAt` returns SUCCESS for spectators without swapping items.
- Stray `// ...` placeholder comments removed from `write_custom_nbt`/`read_custom_nbt`.

## Gapwave2-2 status (h3)
- #5 non-kill destroy paths: FIXED (VERIFIED-IN-TREE, no change this wave): bypass, explosion, creative and break paths all call `self.kill(caller)`, which emits ENTITY_DIE.
- #6 ENTITY_DAMAGE on survivable damage: FIXED (VERIFIED-IN-TREE): wobble branch emits `EntityDamage` with the caller; `cause_damage` branches are unverified.

## Remaining gaps
1. **Armor stand flags are never synced to clients** (size M).
   - `set_bit_field` (armor_stand.rs:149-158) only mutates the local `armor_stand_flags`; nothing calls `Entity::set_synced_data`, so Small/ShowArms/HideBasePlate/Marker never reach clients.
   - Vanilla: `ArmorStand.defineSynchedData` DATA_ARMOR_STAND_FLAGS (index 14, BYTE) and `ArmorStand.setSmall/showArms/hideBasePlate/setMarker` write it.
   - Blocker: `pumpkin_data::tracked_data::armor_stand` (crates/pumpkin-data/src/generated/tracked_data.rs:2777+) has no `DATA_ARMOR_STAND_FLAGS` const (index 14 absent). Needs codegen/extractor data, then synced-data writes + a `CSetEntityMetadata` broadcast.
2. **Pose is never synced** (size M).
   - `unpack_rotation` (line 207) only stores locally; `DATA_HEAD_POSE`/`DATA_BODY_POSE`/`DATA_LEFT_ARM_POSE`/`DATA_RIGHT_ARM_POSE`/`DATA_LEFT_LEG_POSE`/`DATA_RIGHT_LEG_POSE` (indices 16-21) exist in generated data but are unused here.
   - Vanilla `ArmorStand.setModelPose`/`setPose` update those synced entries; follow the pattern in `entity/decoration/display.rs`.
3. **`get_clicked_slot` hit boxes differ from vanilla for small stands** (size S, needs decompiled class).
   - armor_stand.rs:314-320 divides the relative click y by 0.5 for small stands, then armor_stand.rs:424-448 uses chest_top `0.9 + 1.0` and chest_bottom `0.9 + 0.3` for small. Vanilla `ArmorStand.getClickedSlot` compares raw relative y and uses `0.9 + (small ? -0.3 : 0.0)` .. `0.9 + (small ? 0.3 : 0.7)` for the chest and `0.4 + (small ? 0.8 : 0.4)` for the legs. Confirm against the decompiled method before changing.
4. **Paper `PlayerArmorStandManipulateEvent` is never fired** (size M).
   - The event type exists (`crates/pumpkin-core/src/plugin/api/events/player/player_armor_stand_manipulate.rs`) but `interact_at_position` (line 343) fires nothing; Paper fires it around `swap_item` and honours cancellation.
5. **Only `kill()` emits ENTITY_DIE** (size S).
   - The other destroy paths in `damage_with_context` (bypasses-invulnerability line 565, explosion line 575, creative line 616, break line 645) still call `entity.remove()` directly; vanilla routes all of them through `kill()` and therefore emit `GameEvent.ENTITY_DIE`.
6. **No ENTITY_DAMAGE game event on survivable damage** (size S).
   - Vanilla `causeDamage` and the wobble branch emit `GameEvent.ENTITY_DAMAGE` with the source entity; armor_stand.rs:283-292 and 628-637 emit none.
7. **Break particles use a guessed block state** (size S).
   - `spawn_break_particles` (line 265-279) uses `Block::OAK_PLANKS` plus count 10 / speed 0.05; verify against vanilla `ArmorStand.showBreakingParticles` (which derives the state from the block below) and `spawn_block_particle` semantics.
8. **Equipment drops are bare `drop_stack` calls** (size S).
   - `broken_by_anything` (line 240-261) drops each slot at `block_pos.up()` with no random velocity; vanilla spawns `ItemEntity`s with per-slot random offsets/pickup delay. Cosmetic only.

## Gap status
- 1 armor stand flags not synced: BLOCKED-DATA, no DATA_ARMOR_STAND_FLAGS in generated tracked_data (index 14 absent, per note).
- 2 pose not synced (armor_stand.rs:207): STILL OPEN, M; indices 16-21 exist in generated data.
- 3 get_clicked_slot hitboxes (armor_stand.rs:314-448): BLOCKED-DATA, needs decompiled getClickedSlot.
- 4 PlayerArmorStandManipulateEvent never fired (armor_stand.rs:343): STILL OPEN, M.
- 5 only kill() emits ENTITY_DIE: STILL OPEN, S, not landed this pass.
- 6 ENTITY_DAMAGE on survivable damage: STILL OPEN, S.
- 7 break particle block state (armor_stand.rs:265-279): STILL OPEN, S.
- 8 bare drop_stack for equipment: STILL OPEN, S, cosmetic.

## Result
- Row 1 (flags not synced): FIXED (corrected on re-verification). `set_bit_field` (used by all four flag setters, armor_stand.rs:162-168) and NBT load (armor_stand.rs:679-681) call `sync_flags()`, writing `DATA_CLIENT_FLAGS` (index 15) so Small/ShowArms/NoBasePlate/Marker reach clients. The original BLOCKED-DATA reason no longer applies.
- Row 2 (pose not synced): FIXED. `new()` and `unpack_rotation()` now write all six ROTATIONS synched entries (indices 16-21) as `Vector3<f32>` Euler triples, matching decompiled `ArmorStand.defineSynchedData`/`setArmorStandPose`.
- Row 3 (clicked slot hitboxes): VERIFIED, NO CODE CHANGE. Decompiled `ArmorStand.getClickedSlot` matches `clicked_slot` exactly: feet `0.1 .. 0.1 + (small ? 0.8 : 0.45)`, chest `0.9 + (small ? 0.3 : 0.0) .. 0.9 + (small ? 1.0 : 0.7)`, legs `0.4 .. 0.4 + (small ? 1.0 : 0.8)`, head `>= 1.6`, and `/ (scale * ageScale)` equals the existing `/ 0.5` for small stands. The doc note's older thresholds were from a different MC version. Decompiled source now available, so the BLOCKED-DATA reason no longer applies. Added a small-stand regression test.
- Row 4 (PlayerArmorStandManipulateEvent never fired): FIXED. `swap_item` fires the existing event with `fire_blocking` after the disabled-slot checks and returns `true` without touching either stack when cancelled, matching Paper's `swapItem`/`PlayerArmorStandManipulateEvent` cancellation.
- Row 5 (ENTITY_DIE on destroy paths): FIXED. The explosion branch and the `cause_damage` break branch now emit `EntityDie` after removal; bypass-invulnerability, creative and normal break paths already routed through `kill()`.
- Row 6 (ENTITY_DAMAGE on survivable damage): FIXED. `cause_damage` emits `EntityDamage` with the damaging entity on the surviving branch. Burn callers now pass `cause.or(source)` instead of the stand itself.
- Row 7 (break particle block state): VERIFIED, NO CODE CHANGE. Decompiled `showBreakingParticles` uses `Blocks.OAK_PLANKS.defaultBlockState()`, count 10, speed 0.05 and `getY(0.6667)`; current code matches. The doc note about deriving from the block below was stale.
- Row 8 (bare drop_stack for equipment): VERIFIED, NO CODE CHANGE. `drop_stack_at` spawns an `ItemEntity` with random +/-0.25 offsets at the stand position + 1.0, so cosmetic-only randomness is already present.

Files changed: `crates/pumpkin-core/src/entity/decoration/armor_stand.rs` only. `rustfmt --edition 2024` ran with exit 0. Build/tests were not run by this worker (orchestrator compiles once after all workers finish).

## Result (re-verification)

Re-checked every STILL OPEN / BLOCKED-DATA row against the current tree. All eight are resolved in code; the earlier Result's Row 1 status was stale and is corrected above. Static verification only (no cargo run, per worker rules):

1. Flags synced: FIXED (see corrected Row 1 above; sync_flags writes DATA_CLIENT_FLAGS on setter and NBT-load paths).
2. Pose synced: FIXED. new() and unpack_rotation() call sync_pose(), writing DATA_HEAD_POSE..DATA_RIGHT_LEG_POSE (indices 16-21) as Vector3<f32>; that type implements MetadataSerializer (entity_metadata.rs:419).
3. Clicked-slot hitboxes: VERIFIED, no code change (decompiled ArmorStand.getClickedSlot matches clicked_slot; regression test present).
4. PlayerArmorStandManipulateEvent: FIXED. swap_item fires it via fire_blocking after disabled-slot checks; cancellation returns true without mutating either stack. Event fields plus cancelled from #[cancellable] match the construction; fire_blocking(&server, &mut event) matches experience_orb.rs:383.
5. ENTITY_DIE on destroy paths: FIXED (kill, explosion branch, cause_damage break branch all emit EntityDie).
6. ENTITY_DAMAGE on survivable damage: FIXED (cause_damage surviving branch and wobble branch).
7. Break particle block state: VERIFIED (Block::OAK_PLANKS, count 10, speed 0.05; spawn_block_particle signature matches).
8. Equipment drops: VERIFIED (drop_stack_at at stand pos +1.0 for every slot).

No .rs file changed in this pass, so rustfmt was not required. Build/tests not run (orchestrator compiles once after all workers finish).


## Result (decompiled-vanilla re-verification)

Re-checked every STILL OPEN / BLOCKED-DATA row against decompiled
`net/minecraft/world/entity/decoration/ArmorStand.java` (now present under
`.vendor/vanilla-decompiled/`) and the current Pumpkin code. All eight rows are
already resolved; no source file changed this pass.

1. Flags synced: FIXED. `set_bit_field` (armor_stand.rs:162-168) and NBT load
   (armor_stand.rs:678-681) call `sync_flags()`, which writes `DATA_CLIENT_FLAGS`
   (armor_stand.rs:248-254). Decompiled `defineSynchedData` (Java:139-148) uses one
   BYTE accessor `DATA_CLIENT_FLAGS` with bit flags (Small=1, ShowArms=4,
   NoBasePlate=8, Marker=16); the generated const exists (tracked_data.rs:2791).
   `Entity::set_synced_data` marks data dirty for the end-of-tick tracker broadcast
   (entity/mod.rs:4884-4891), so all four flags reach clients.
2. Pose synced: FIXED. `new()` and `unpack_rotation()` call `sync_pose()`
   (armor_stand.rs:217-244), writing the six pose entries DATA_HEAD_POSE..
   DATA_RIGHT_LEG_POSE as `Vector3<f32>`; decompiled `defineSynchedData`
   (Java:142-147) defines exactly those six. Constants exist
   (tracked_data.rs:2811..2843) and `Vector3<f32>` implements `MetadataSerializer`
   (entity_metadata.rs:419), so this type-checks.
3. Clicked-slot hitboxes: VERIFIED, no code change. Decompiled `getClickedSlot`
   (Java:257-275) matches `clicked_slot` (armor_stand.rs:550-574) exactly: feet
   0.1..0.1+(small?0.8:0.45), chest 0.9+(small?0.3:0.0)..0.9+(small?1.0:0.7),
   legs 0.4..0.4+(small?1.0:0.8), head >=1.6, and division by
   `getScale()*getAgeScale()` equals the existing `/0.5` for small stands.
   Regression tests present (armor_stand.rs:898-911, 1020-1031).
4. PlayerArmorStandManipulateEvent: FIXED. `swap_item` fires it via `fire_blocking`
   (armor_stand.rs:406-420) after the disabled-slot checks. Decompiled `swapItem`
   (Java:281-305) fires the Bukkit/Paper event after the same removal/insertion
   checks and returns `true` untouched on cancellation; Pumpkin returns `true`
   without mutating either stack. The BLOCKED/WIT concern does not apply (pure
   Rust event, already wired).
5. ENTITY_DIE on destroy paths: FIXED. Decompiled `kill` (Java:591-600) does
   `remove` then `gameEvent(ENTITY_DIE)`. Pumpkin emits `EntityDie` in `kill()`
   (armor_stand.rs:697-704), in the explosion branch (armor_stand.rs:747-753),
   and in the `cause_damage` break branch (armor_stand.rs:346-352); all other
   paths route through `kill()`.
6. ENTITY_DAMAGE on survivable damage: FIXED. Decompiled wobble branch
   (Java:413-416) and `causeDamage` surviving branch (Java:483-486) emit
   `gameEvent(ENTITY_DAMAGE, source.getEntity())`. Pumpkin emits `EntityDamage`
   with the damaging entity in both branches (armor_stand.rs:355, 811), with burn
   callers passing `cause.or(source)` (armor_stand.rs:716, 759, 769).
7. Break particle block state: VERIFIED, no code change. Decompiled
   `showBreakingParticles` (Java:451-466) uses `OAK_PLANKS.defaultBlockState()`,
   `getY(0.6667)` (== `height/1.5`), width/height/4 spread, count 10, speed 0.05;
   `spawn_break_particles` (armor_stand.rs:323-338) matches.
8. Equipment drops: VERIFIED, no code change. `broken_by_anything`
   (armor_stand.rs:296-320) iterates all eight slots and uses `drop_stack_at` at
   pos+1.0, matching decompiled `brokenByAnything`. Cosmetic random offsets are
   supplied by `drop_stack_at`; parity is sufficient.

No new pure unit test added: the only pure seam (`clicked_slot`, flag/stack
helpers) is already covered, and sync/event behaviour needs a live world/server.
Rustfmt not required (no .rs change). Build/tests not run (worker rule:
orchestrator compiles once after all workers finish).
