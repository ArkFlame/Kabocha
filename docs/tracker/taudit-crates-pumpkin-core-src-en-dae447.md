# Audit: crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs

Category: audit. Coverage: AUDIT. Source: `FishingBobberEntity` (vanilla `FishingHook` / `FishingBobberEntity`).
Markers: no `TODO`/`FIXME` in the file; the two tracker "vanilla markers" are doc comments
("Matches vanilla FishingBobberEntity.use" L124, "Matches vanilla FishingBobberEntity.catchingFish" L547).

## Fixed

- L542 `discard()`: vanilla `FishingHook.updateOwnerInfo` clears `player.fishing` only when it is
  *this* hook. The port wrote `-1` unconditionally, so a stale hook's removal could orphan a newer
  bobber. Now a `compare_exchange` on `player.fishing_bobber`.

## Open gaps

1. **Open-water flag missing (medium).** `catching_fish` (L549+) and `tick` never track vanilla
   `FishingHook.openWater` (`calculateOpenWater`: 5x5 area at the hook is water down to the floor
   and air above). Vanilla exposes it through `isOpenWaterFishing()` and the
   `minecraft:gameplay/fishing/treasure` loot condition
   (`type_specific.fishing_hook.in_open_water`), so treasure is only rolled in open water.
   Consequence: the 5% treasure branch can never be gated correctly; loot generation ignores it.
   Fix: compute the flag when entering `HookState::Bobbing`, store it on the entity, and wire it
   into the loot `this_entity` predicate support in `crate::world::loot`.

2. **Cancelled `PlayerFishEvent` keeps the hook in Bukkit (small).**
   `reel_in` returns 0 on a cancelled `CAUGHT_FISH`/`CAUGHT_ENTITY` (L205-214, L139-150), which
   matches CraftBukkit returning early from `FishingHook.retrieve` *before* `discard()`. But the
   caller `item/items/fishing_rod.rs:96-106` removes the hook and clears `fishing_bobber`
   unconditionally, so a cancelled event still destroys the hook. Fix: have `reel_in` return
   `Option<i32>` (`None` = keep hook) and let the rod skip the removal.

3. **`should_stop_fishing` does not discard (trivial).** Vanilla `FishingHook.shouldStopFishing`
   calls `this.discard()` before returning `true`; here only `process_tick` discards, and the
   `retrieve` path relies on the caller. Harmless today, but worth aligning when (2) is reworked.

4. **Rod durability application (small).** Vanilla `FishingRodItem.use` calls
   `stack.hurtAndBreak(retrieve(...), player, slot)`; the port calls
   `player.damage_item_in_slot(&slot, damage)` (`fishing_rod.rs:98`). Verify that path applies
   Unbreaking and triggers the item-break sound/`ItemBreakEvent` for the rod.

5. **Random precision (cosmetic, no fix planned).** `bob_velocity` random (L409) and the tease
   chance/distance rolls (L663-685) use `rand::random::<f64>()`, vanilla uses
   `RandomSource.nextFloat()` (f32). Distribution difference only.

## Verified as matching vanilla

- Throw origin/velocity (`throw_setup`), eye offset -0.1, yaw/pitch from launch movement (L84-99).
- `luck = max(1, luck)`, wait `100..=600` minus lure, bite `20..=40`, hook `20..=80` (L104-108, L651, L693).
- Synced random reseeded per tick with `uuid.min ^ gameTime` for the bite kick (L283-291) and the
  biting pull-down using two `nextFloat` products (L414-417).
- Fly -> Bobbing water damping `(0.3, 0.2, 0.3)` and early return; gravity `-0.03` while airborne;
  inertia `0.92` (L394-401, L449-451, L470-472).
- `bob_velocity` formula and `+signum*0.1` snap (L712-728).
- `retrieve` damage 3 (item entity) / 5 (other entity) / 1 (fish) / 2 (on ground), loot params
  (tool, origin, this entity, luck), item launch velocity and 10 tick pickup delay, xp 1..=6,
  `FISH_CAUGHT` stat and `FISHED_ITEM` advancement (L126-279).
- `setHookedEntity` stores `id + 1`, hooked entity teleport to `0.8 * height`, block hit stopping
  at the face, `FishingRodReelIn` status broadcast, off-hand rod accepted (L157-161, L377-391, L502-521, L525-539).

## Gap status
- 1 open-water flag missing: VERIFIED (matches vanilla). latched_open_water + is_open_water_fishing (fishing_bobber.rs), is_open_water (predicates.rs), loot in_open_water gate; .vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java:233-236,442-485.
- 2 cancelled PlayerFishEvent still removes hook: VERIFIED (matches vanilla). reel_in returns Option<i32>; fishing_rod.rs keeps hook on None; CraftBukkit FishingHook.retrieve early return before discard().
- 3 should_stop_fishing does not discard: VERIFIED (matches vanilla). Calls self.discard(player) before returning true; FishingHook.java:273-291.
- 4 rod durability path: VERIFIED (matches vanilla). damage_item_in_slot rolls Unbreaking and fires break events/sound; FishingRodItem.java:27.
- 5 random f64 vs f32: VERIFIED (matches vanilla). All rolls are f32 / next_f32; FishingHook.java:153,232,348,403-405,417.
- L542 discard compare-exchange: FIXED at fishing_bobber.rs:554 (verified).


## Result

Verified each originally open row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. Findings below supersede the "Gap status" list above.

- **Row 1 (open-water flag, M): LEFT OPEN.** The loot side of this gap is already resolved outside
  this unit: `crate::world/loot/predicates.rs` handles `fishing_hook.in_open_water` via
  `get_fishing_hook_in_open_water`/`is_open_water`, and the treasure table condition
  (`pumpkin-data generated/loot_table.rs` `LOOT_PREDICATE_562`, `in_open_water: true`) is therefore
  gated, contrary to the audit's "loot generation ignores it" claim. The only residual piece is a
  stored `openWater` field plus an `isOpenWaterFishing()` API for the `FishingHookPredicate`
  advancement predicate; that needs entity-field + predicate work in files outside this unit.
  Reason: cross-file (advancement/predicate) work outside the two permitted files.
- **Row 2 (cancelled PlayerFishEvent removes hook, S): VERIFIED CORRECT, no change needed.**
  `FishingBobberEntity::reel_in` returns `Option<i32>` (`None` on a cancelled
  CAUGHT_ENTITY/CAUGHT_FISH/IN_GROUND/REEL_IN event), and `fishing_rod.rs` keeps the hook and
  `player.fishing_bobber` on `None`, removing only on `Some`. This matches CraftBukkit
  `FishingHook.retrieve`, which returns 0 early on a cancelled event *before* the `discard()` at the
  end (FishingHook.java lines 518-520, 544-546, 573-575, 582-584 vs 588).
- **Row 3 (should_stop_fishing does not discard, trivial): VERIFIED CORRECT, no change needed.**
  `should_stop_fishing` calls `self.discard(player)` before returning `true`
  (fishing_bobber.rs:538), matching vanilla `FishingHook.shouldStopFishing` (FishingHook.java:289).
- **Row 4 (rod durability path, S): VERIFIED CORRECT, no change needed.**
  `player.damage_item_in_slot` -> `damage_item_stack_internal` -> `stack.damage_item` applies
  Unbreaking (`pumpkin-data/item_stack/mod.rs` lines 475-483) and fires `PlayerItemDamageEvent`,
  `PlayerItemBreakEvent`, the break sound and the equipment-break entity status
  (`entity/player.rs` lines 2159-2244). This covers the audit's two verification points for vanilla
  `stack.hurtAndBreak(dmg, player, slot)` (`FishingRodItem.java:27`).
- **Row 5 (random f64 vs f32): LEFT OPEN.** Cosmetic distribution-only difference, accepted by
  note. Reason: cosmetic, no fix planned.

**Tests:** no new pure unit-test seam exists for rows 2-4: the only pure helpers in these two files
(`bob_velocity`, `throw_setup`, `bobber_sound_pitch`) are either already covered or trivial
formula-mirrors that the project's test rules discourage, and the row-2/3/4 behaviour requires a
live `Player`/`World`. No `.rs` file was changed, so no rustfmt was required.


## Result (re-verification against decompiled vanilla, current checkout)

Every fixable row is already implemented in the working tree; no `.rs` file needed changes. This
supersedes the earlier `## Result` above, whose row 1 was stale.

- **Row 1 (open-water flag, M): FIXED in code.** `FishingBobberEntity` stores
  `open_water: AtomicBool` and exposes `is_open_water_fishing()` (fishing_bobber.rs:56,175). It
  latches the flag in `process_tick` through the pure `latched_open_water` helper
  (fishing_bobber.rs:481-491,932-944), matching `FishingHook.tick:233-237`
  (`openWater = openWater && outOfWaterTime < 10 && calculateOpenWater`, reset to true when no fish
  is near). `world/loot/predicates.rs` gates `fishing_hook.in_open_water` on that latched flag
  (predicates.rs:268-273,372-392) and implements the 5x5 / y=-1..=2 `calculateOpenWater` scan
  (predicates.rs:365-370,414-449). Pure unit tests exist for both helpers (fishing_bobber.rs:1065+,
  predicates.rs:1034+). No cross-file work remains.
- **Row 2 (cancelled PlayerFishEvent, S): FIXED.** `reel_in -> Option<i32>` returns `None` on a
  cancelled CAUGHT_ENTITY/CAUGHT_FISH/IN_GROUND/REEL_IN event; `fishing_rod.rs:136-158` keeps the
  hook and `player.fishing_bobber` on `None` and removes only on `Some`. Matches CraftBukkit
  `FishingHook.retrieve`, which returns 0 on a cancelled event before the `discard()` at line 588
  (FishingHook.java:518-520, 544-546, 573-575, 582-584).
- **Row 3 (should_stop_fishing discard, trivial): FIXED.** Calls `self.discard(player)` before
  returning true (fishing_bobber.rs:605-622), matching `FishingHook.shouldStopFishing`
  (FishingHook.java:273-291).
- **Row 4 (rod durability, S): CORRECT.** `fishing_rod.rs:146` uses `damage_item_in_slot` ->
  `damage_item_stack_internal` (player.rs:2289-2383), which rolls Unbreaking via
  `compute_durability_damage_applied`, fires `PlayerItemDamageEvent`/`PlayerItemBreakEvent` and
  broadcasts/syncs the break. This covers vanilla `stack.hurtAndBreak(dmg, player, slot)`.
- **Row 5 (random f64 vs f32): LEFT OPEN.** Cosmetic distribution-only difference, accepted by
  note. Reason: cosmetic, no fix planned.

No `.rs` file was changed, so `rustfmt` was not required. No compile was run (workers do not
compile); verification was by reading the code against `.vendor/vanilla-decompiled` sources only.


## Result (post-restart independent re-verification)

The server restarted mid-unit. Re-verified every originally open row against the actual current
checkout (not the two prior `## Result` blocks) and decompiled vanilla. All fixable rows are
already implemented correctly; no `.rs` file needed changes this pass.

- **Row 1 (open-water flag, M): CORRECT in code.** `FishingBobberEntity` stores
  `open_water: AtomicBool` and exposes `is_open_water_fishing()`. `process_tick` latches the flag
  via the pure `latched_open_water` helper (`nibble<=0 && hooked<=0 -> true`, else
  `current && out_of_water_time<10 && area_open`), reading `out_of_water_time` from the state at
  tick start before its update - exactly vanilla `FishingHook.tick:233-237` (latch at 236, counter
  updated at 240/251). `predicates.rs` `is_open_water` (5x5, y=-1..=2) reproduces vanilla
  `calculateOpenWater` layer rules (FishingHook.java:442-466): uniform layer, air-not-on-bottom,
  water-not-above-air; the `is_water_source` gate matches `getOpenWaterTypeForBlock`'s
  `fluidState.isSource()` requirement (475-485). The loot `in_open_water` predicate reads the
  latched flag. Pure tests already exist for both helpers (`latched_open_water`,
  `is_open_water`), so no new test was added (duplicates are discouraged).
- **Row 2 (cancelled PlayerFishEvent, S): CORRECT.** `reel_in -> Option<i32>`; `fire_fish_event`
  returns `None` on cancel; `fishing_rod.rs:132-154` keeps the hook and `player.fishing_bobber` on
  `None` and removes only on `Some`. Matches CraftBukkit `FishingHook.retrieve` early return
  before `discard()`.
- **Row 3 (should_stop_fishing discard, trivial): CORRECT.** Calls `self.discard(player)` before
  returning `true` (fishing_bobber.rs:620), matching vanilla `shouldStopFishing`.
- **Row 4 (rod durability, S): CORRECT.** `fishing_rod.rs:142` uses `damage_item_in_slot` ->
  `damage_item_stack_internal` (player.rs:2289-2412), which rolls Unbreaking via
  `compute_durability_damage_applied`, fires `PlayerItemDamageEvent`/`PlayerItemBreakEvent`, plays
  the break sound, broadcasts the equipment-break entity status and awards the BROKEN stat. This
  covers vanilla `stack.hurtAndBreak(dmg, player, slot)`.
- **Row 5 (random f64 vs f32): LEFT OPEN.** Cosmetic distribution-only difference, accepted by
  note. Reason: cosmetic, no fix planned.

No `.rs` file changed, so no rustfmt. No compile run (workers do not compile); verification by
reading code against `.vendor/vanilla-decompiled` only.

Status: PASS fixed=0 open=1 files=none


## Result (fresh pass, independent verification of current checkout)

Re-verified every originally open row against decompiled vanilla FishingHook.java and CraftBukkit FishingHook. All fixable rows are already implemented in the working tree; no .rs file needed changes.

- **Row 1 (open-water flag, M): CORRECT.** FishingBobberEntity stores open_water: AtomicBool and exposes is_open_water_fishing(). process_tick latches via the pure latched_open_water helper, reading out_of_water_time from the state at tick start before its update, matching vanilla FishingHook.tick 233-251 with MAX_OUT_OF_WATER_TIME=10. predicates.rs is_open_water reproduces calculateOpenWater layer rules (FishingHook.java 442-485) and gates the loot in_open_water predicate (predicates.rs 268-273). Pure tests exist for both helpers (fishing_bobber.rs 1065+, predicates.rs 1042+). No new test added (duplicates discouraged).
- **Row 2 (cancelled PlayerFishEvent, S): CORRECT.** reel_in returns Option<i32>; fire_fish_event returns None on cancel (fishing_bobber.rs 381-385); fishing_rod.rs 138-154 keeps the hook and player.fishing_bobber on None and removes only on Some. Matches CraftBukkit retrieve early return before discard.
- **Row 3 (should_stop_fishing discard, trivial): CORRECT.** calls self.discard(player) before returning true (fishing_bobber.rs 620), matching FishingHook.java 289.
- **Row 4 (rod durability, S): CORRECT.** fishing_rod.rs 142 uses damage_item_in_slot -> damage_item_stack_internal, which rolls Unbreaking and fires the item damage/break events and sound. Covers vanilla stack.hurtAndBreak.
- **Row 5 (random f64 vs f32): LEFT OPEN.** Cosmetic distribution-only difference, accepted by note. Reason: cosmetic, no fix planned.

No .rs file changed, so no rustfmt. No compile run (workers do not compile); verification by reading code against .vendor/vanilla-decompiled only. Re-checked the diff against the worker compile-pitfalls table: no changes made, so none apply.


## Result (second pass, full cross-file scope granted)

Re-verified every originally open row against decompiled vanilla FishingHook.java / CraftBukkit FishingHook
with permission to edit any needed source file. All fixable rows are already implemented and correct
in the current checkout; no .rs file needed changes this pass.

- **Row 1 (open-water flag, M): CORRECT.** `FishingBobberEntity` stores `open_water: AtomicBool` and
  exposes `is_open_water_fishing()` (fishing_bobber.rs:56,175). `process_tick` latches via the pure
  `latched_open_water` helper (fishing_bobber.rs:932-944), reading `out_of_water_time` from the state
  at tick start, matching vanilla `FishingHook.tick:233-251` (MAX_OUT_OF_WATER_TIME=10). The fishing
  loot params set `this_entity: Some(self)` (fishing_bobber.rs:239), so the predicate reads the latched
  flag. `predicates.rs` `is_open_water` (416-453) reproduces vanilla `calculateOpenWater`
  (FishingHook.java:442-485) exactly: per-layer uniform scan, ABOVE_WATER with INVALID previous ->
  false, INSIDE_WATER with ABOVE_WATER previous -> false, source-only water gate (365-374 vs 475-481).
  The loot `fishing_hook.in_open_water` condition is wired at predicates.rs:268-273. Pure unit tests
  exist for both helpers (fishing_bobber.rs:1065+, predicates.rs:1039+). No new test added (duplicates
  are discouraged).
- **Row 2 (cancelled PlayerFishEvent, S): CORRECT.** `reel_in -> Option<i32>` and `fire_fish_event`
  return `None` on cancel; `fishing_rod.rs:132-154` keeps the hook and `player.fishing_bobber` on
  `None` and removes only on `Some`. Matches CraftBukkit `retrieve` early return before `discard()`.
- **Row 3 (should_stop_fishing discard, trivial): CORRECT.** Calls `self.discard(player)` before
  returning true (fishing_bobber.rs:620), matching FishingHook.java:289.
- **Row 4 (rod durability, S): CORRECT.** `fishing_rod.rs:142` uses `damage_item_in_slot`, which rolls
  Unbreaking and fires the damage/break events and sound. Covers vanilla `stack.hurtAndBreak`.
- **Row 5 (random f64 vs f32): LEFT OPEN.** Cosmetic distribution-only difference, accepted by note.
  Reason: cosmetic, no fix planned.

No .rs file changed, so no rustfmt needed. No compile run (workers do not compile); verification by
reading code against .vendor/vanilla-decompiled only. Diff re-checked against the worker compile-pitfalls
table: no changes made, so none apply.

Status: PASS fixed=0 open=1 files=none


## Result (third pass: all remaining rows resolved)

Independent verification of the current checkout against decompiled vanilla FishingHook.java
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java`) and CraftBukkit
FishingHook. No .rs file needed changes this pass.

- **Row 5 (random f64 vs f32): VERIFIED (matches vanilla).** Every random roll in fishing_bobber.rs
  now uses f32 precision, matching vanilla `RandomSource.nextFloat()` / `Mth.nextFloat(..., float)`:
  bob_velocity caller passes `rand::random::<f32>()` (fishing_bobber.rs:482) vs FishingHook.java:232
  `force * this.random.nextFloat() * 0.2`; fish approach `rand::rng().triangle(0.0, 9.188) as f32`
  (:685) vs FishingHook.java:348 `(float) this.random.triangle(0.0, 9.188)`; tease roll
  `rand::random::<f32>() < tease_chance` (:770) vs :403 `this.random.nextFloat() < teaseChance`;
  tease angle/distance (:771-772) `f64::from(rand::random::<f32>() * 360.0)` /
  `f64::from(25.0f32 + rand::random::<f32>() * 35.0)` vs :404-405
  `Mth.nextFloat(random, 0.0F, 360.0F)` / `Mth.nextFloat(random, 25.0F, 60.0F)`; lure angle
  `rand::random::<f32>() * 360.0` (:789) vs :417 `Mth.nextFloat(random, minLureAngle,
  maxLureAngle)` (defaults 0/360); synced bite kick `synced.next_f32()` (:756) vs :153
  `Mth.nextFloat(syncronizedRandom, 0.6F, 1.0F)`. The earlier f64 rolls were converted to f32 in
  a prior pass; no `random::<f64>()` remains in the file. No fix required.

Rows 1-4 were already FIXED/CORRECT in prior passes (open-water flag, cancelled-event hook
retention, should_stop_fishing discard, rod durability). Every originally open row is now resolved:
none remains open, and none needs generated data or a missing subsystem, so BLOCKED-DATA does not
apply.

No .rs file changed, so rustfmt was not required. No compile run (workers do not compile);
verification by reading code against .vendor/vanilla-decompiled only. Diff re-checked against the
worker compile-pitfalls table: no changes made, so none apply.

Status: PASS fixed=0 open=0 files=none


## Result (third pass: independent close-out, final)

Independently re-verified each originally open row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java` and the current
checkout of the four unit files. No `.rs` change was needed: every row is already implemented and
matches vanilla, so no BLOCKED-DATA (missing generated field or subsystem) applies.

- **Row 1 (open-water flag, M): VERIFIED (matches vanilla).** `FishingBobberEntity` stores
  `open_water: AtomicBool` and exposes `is_open_water_fishing()` (fishing_bobber.rs:56,175).
  `process_tick` latches via the pure `latched_open_water` helper (fishing_bobber.rs:489-499,940-952),
  reading `out_of_water_time` from the state at tick start BEFORE its update, matching
  `FishingHook.tick` lines 233-237 (`nibble<=0 && timeUntilHooked<=0 -> openWater=true`, else
  `openWater && outOfWaterTime<10 && calculateOpenWater`) with MAX_OUT_OF_WATER_TIME=10 (vanilla
  line 59, update at 240/251). The fishing loot params pass `this_entity: Some(self)`
  (fishing_bobber.rs:239); `predicates.rs` gates `fishing_hook.in_open_water` on the latched flag
  (predicates.rs:268-273,377-397) and `is_open_water` reproduces `calculateOpenWater` layer rules
  (predicates.rs:419-454 vs FishingHook.java:442-485): uniform 5x5 layer, ABOVE_WATER/Air with
  INVALID previous -> false, INSIDE_WATER/Water with ABOVE_WATER previous -> false, source-only
  water gate (predicates.rs:365-374 vs FishingHook.java:475-481). Pure unit tests exist for both
  helpers (fishing_bobber.rs:1073+, predicates.rs:1043+), so no duplicate test was added.
- **Row 2 (cancelled PlayerFishEvent removes hook, S): VERIFIED (matches vanilla).** `reel_in`
  returns `Option<i32>` (`None` on cancelled CAUGHT_ENTITY/CAUGHT_FISH/IN_GROUND/REEL_IN via
  `fire_fish_event` at fishing_bobber.rs:361-394); `fishing_rod.rs:132-154` sets `keep_hook=true` on
  `None`, skips the removal and the `player.fishing_bobber.store(-1)`. This matches CraftBukkit
  `FishingHook.retrieve`, which returns early on a cancelled event before the terminal `discard()`.
- **Row 3 (should_stop_fishing does not discard, trivial): VERIFIED (matches vanilla).**
  `should_stop_fishing` calls `self.discard(player)` before returning true (fishing_bobber.rs:628),
  matching `FishingHook.shouldStopFishing` (FishingHook.java:273-291). `discard` also uses a
  compare_exchange so a stale hook cannot orphan a newer bobber.
- **Row 4 (rod durability path, S): VERIFIED (matches vanilla).** `fishing_rod.rs:142` calls
  `player.damage_item_in_slot(&equipment_slot, damage)`, which rolls Unbreaking via
  `compute_durability_damage_applied` and fires the item damage/break events and break sound. This
  covers vanilla `stack.hurtAndBreak(retrieve(...), player, slot)` (`FishingRodItem.java:27`).
- **Row 5 (random f64 vs f32): VERIFIED (matches vanilla).** No `random::<f64>()` remains: throw
  bob velocity uses `rand::random::<f32>()` (fishing_bobber.rs:482 vs FishingHook.java:232), fish
  approach `rand::rng().triangle(0.0,9.188) as f32` (:685 vs :348), tease/lure rolls and the synced
  bite kick all use f32 / `synced.next_f32()` (:505,659,662,695,756,770-772,789 vs
  FishingHook.java:153,326,330,357,403-405,417).

Every originally open row is now FIXED/VERIFIED with a vanilla citation; none remains open and none
needs generated data or a missing subsystem. No `.rs` file changed, so rustfmt was not required.
No compile run (workers do not compile); verification was by reading code against `.vendor/` only.
Diff re-checked against the worker compile-pitfalls table: no changes made, so none apply.

Status: PASS fixed=0 open=0 files=none


## Result (final pass, THIRD PASS close-out)

Independent verification of the actual current checkout of all unit files (fishing_bobber.rs,
fishing_rod.rs, world/loot/predicates.rs) against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java`. All five originally open rows
were already implemented correctly in code; no `.rs` change was needed, and no BLOCKED-DATA
condition (missing generated pumpkin-data field or whole subsystem) applies.

- **Row 1 (open-water flag, M): VERIFIED (matches vanilla).** `FishingBobberEntity` stores
  `open_water: AtomicBool` and exposes `is_open_water_fishing()` (fishing_bobber.rs:56,175).
  `process_tick` latches via pure `latched_open_water` (fishing_bobber.rs:490-499,940-952), reading
  `out_of_water_time` from the `HookState::Bobbing` value at tick start before that state is
  updated (:514/:518), exactly vanilla `FishingHook.tick:233-237` with MAX_OUT_OF_WATER_TIME=10
  (FishingHook.java:59,236,251). `predicates.rs` gates the loot `fishing_hook.in_open_water` condition
  on the latched flag (:268-273,377-397), and `is_open_water`/`is_open_water_at` reproduce
  `calculateOpenWater` layer rules (predicates.rs:365-374,419-454 vs FishingHook.java:442-485):
  uniform 5x5 layer, ABOVE_WATER/Air with INVALID previous -> false, INSIDE_WATER/Water with
  ABOVE_WATER previous -> false, source-only water gate. Pure tests exist for both helpers
  (fishing_bobber.rs:1073+, predicates.rs:1040+); no duplicate added.
- **Row 2 (cancelled PlayerFishEvent removes hook, S): VERIFIED (matches vanilla).** `reel_in`
  returns `Option<i32>`; `fire_fish_event` returns `None` on a cancelled event
  (fishing_bobber.rs:389-393), propagated by `?` (e.g. :194-200). `fishing_rod.rs:132-154` sets
  `keep_hook=true` on `None`, skipping `bobber_base...remove()` and the
  `player.fishing_bobber.store(-1)`. Matches CraftBukkit `FishingHook.retrieve`, which returns early
  on a cancelled event before the terminal `discard()`.
- **Row 3 (should_stop_fishing does not discard, trivial): VERIFIED (matches vanilla).**
  `should_stop_fishing` calls `self.discard(player)` before returning true (fishing_bobber.rs:628),
  matching FishingHook.java:273-290 (discard at :289). `discard` uses a compare_exchange so a stale
  hook cannot orphan a newer bobber (:633-643).
- **Row 4 (rod durability path, S): VERIFIED (matches vanilla).** `fishing_rod.rs:142` calls
  `player.damage_item_in_slot(&equipment_slot, damage)`, which rolls Unbreaking and fires the item
  damage/break events and break sound, covering vanilla
  `stack.hurtAndBreak(retrieve(...), player, slot)` (FishingRodItem.java:27).
- **Row 5 (random f64 vs f32): VERIFIED (matches vanilla).** No `random::<f64>()` remains; bob
  velocity uses `rand::random::<f32>()` (fishing_bobber.rs:482), tease/lure/bite rolls use f32 or
  `synced.next_f32()` (:505,659,662,695,770-772,789), matching `RandomSource.nextFloat()` /
  `Mth.nextFloat` (FishingHook.java:153,232,348,403-405,417).

Every originally open row is now FIXED/VERIFIED with a vanilla citation; none remains open and none needs
generated data or a missing subsystem, so BLOCKED-DATA does not apply. No `.rs` file changed, so
rustfmt was not required; no compile run (workers do not compile), verification by reading code
against `.vendor/` only. Diff re-checked against the worker compile-pitfalls table: no changes made,
so none apply.

Status: PASS fixed=0 open=0 files=none


## Result (third pass, final independent close-out)

Re-verified all five rows against the current checkout of fishing_bobber.rs, fishing_rod.rs and
world/loot/predicates.rs and decompiled vanilla FishingHook.java and FishingRodItem.java at
.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/ and .../world/item/.
All five are already implemented and match vanilla; no .rs change was needed and no BLOCKED-DATA
(missing generated pumpkin-data field or whole subsystem) applies.

- Row 1 (open-water flag, M): VERIFIED (matches vanilla). FishingBobberEntity stores
  open_water: AtomicBool and exposes is_open_water_fishing() (fishing_bobber.rs:56,175).
  process_tick latches via the pure latched_open_water helper (fishing_bobber.rs:489-499,940-952),
  reading out_of_water_time from the HookState::Bobbing value at tick start before its update,
  matching FishingHook.java:233-236 with MAX_OUT_OF_WATER_TIME=10 (line 59). predicates.rs
  is_open_water (419-453) reproduces calculateOpenWater (FishingHook.java:442-485): uniform 5x5
  layer, ABOVE_WATER/Air with INVALID previous -> false, INSIDE_WATER/Water with ABOVE_WATER
  previous -> false, source-only water gate (365-374 vs 475-481). The loot in_open_water
  condition reads the latched flag (predicates.rs:268-273,377-396). Pure tests exist for both
  helpers (fishing_bobber.rs:1073+, predicates.rs:1040+); no duplicate added.
- Row 2 (cancelled PlayerFishEvent removes hook, S): VERIFIED (matches vanilla). reel_in
  returns Option<i32>; fire_fish_event returns None on cancel (fishing_bobber.rs:389-393),
  propagated with ?. fishing_rod.rs:132-154 sets keep_hook=true on None and skips removal and
  the player.fishing_bobber.store(-1). Matches CraftBukkit FishingHook.retrieve, which returns
  early on a cancelled event before the terminal discard().
- Row 3 (should_stop_fishing does not discard, trivial): VERIFIED (matches vanilla). Calls
  self.discard(player) before returning true (fishing_bobber.rs:613-630), matching
  FishingHook.java:273-291 (discard at line 289). discard also uses a compare_exchange.
- Row 4 (rod durability path, S): VERIFIED (matches vanilla). fishing_rod.rs:142 calls
  player.damage_item_in_slot, which rolls Unbreaking and fires the item damage/break events and
  break sound, covering stack.hurtAndBreak(retrieve(...), player, slot) (FishingRodItem.java:27).
- Row 5 (random f64 vs f32): VERIFIED (matches vanilla). No random::<f64>() remains; all rolls
  use f32 / synced.next_f32() (fishing_bobber.rs:482,505,659,662,684-685,695,738,756,770-772,789),
  matching RandomSource.nextFloat() / Mth.nextFloat (FishingHook.java:153,232,348,403-405,417).

Every originally open row is now VERIFIED with a vanilla citation; none remains open, so BLOCKED-DATA
does not apply. No .rs file changed, so rustfmt was not required; no compile run (workers do not
compile), verification by reading code against .vendor/ only. Diff re-checked against the worker
compile-pitfalls table: no code changes, so none apply.

Status: PASS fixed=0 open=0 files=none
