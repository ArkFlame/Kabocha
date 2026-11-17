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
- 1 open-water flag missing: STILL OPEN, M (fishing_bobber.rs catching_fish/tick; loot predicate wiring).
- 2 cancelled PlayerFishEvent still removes hook (item/items/fishing_rod.rs:96-106): STILL OPEN, S.
- 3 should_stop_fishing does not discard: STILL OPEN, trivial, harmless today.
- 4 rod durability path (fishing_rod.rs:98): STILL OPEN, verify.
- 5 random f64 vs f32: STILL OPEN, cosmetic, accepted by note.
- L542 discard compare-exchange: FIXED at fishing_bobber.rs:554 (verified).


## Result

Verified each STILL OPEN row against decompiled vanilla at
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
