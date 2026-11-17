# Audit: crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs

Category: audit. Coverage: AUDIT. Source: `FishingBobberEntity` (vanilla `FishingHook` / `FishingBobberEntity`).
Markers: no `TODO`/`FIXME` in the file; the two tracker "vanilla markers" are doc comments
("Matches vanilla FishingBobberEntity.use" L124, "Matches vanilla FishingBobberEntity.catchingFish" L547).

## Fixed

- L542 `discard()`: vanilla `FishingHook.updateOwnerInfo` clears `player.fishing` only when it is
  *this* hook. The port wrote `-1` unconditionally, so a stale hook's removal could orphan a newer
  bobber. Now a `compare_exchange` on `player.fishing_bobber`.

## Gapwave2-2 status (h3)

- #2 cancelled PlayerFishEvent keeps hook: FIXED 20b15d659 (reel_in returns Option, None on cancel; rod keeps the hook and skips the -1 store; UNVERIFIED, no test)
- #3 should_stop_fishing discard: STILL OPEN (trivial, harmless today, to align with #2 rework later)

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
- #3 should_stop_fishing discard: FIXED faf838f3b (discard moved into should_stop_fishing, process_tick no longer discards separately; UNVERIFIED, no test)
