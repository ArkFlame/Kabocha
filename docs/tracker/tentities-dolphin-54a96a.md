# Tracker: entities / Dolphin (RE-REVIEW 54a96a)

Audited `crates/pumpkin-core/src/entity/passive/dolphin.rs` (648 lines) and
`crates/pumpkin-core/src/entity/passive/dolphin/goals.rs` (857 lines) against vanilla
`Dolphin` + `Dolphin$DolphinSwimToTreasureGoal` / `$DolphinSwimWithPlayerGoal` /
`$DolphinJumpGoal` / `$PlayWithItemsGoal` / `$FollowPlayerRiddenEntityGoal`.
No `.vendor/` decompiled tree exists in this worktree, so class behaviour is from vanilla
knowledge and `crates/pumpkin-data/src/generated/`.

Trigger for the re-review: `0505dd44e` "fix(entity): give all living entities an air supply
(#3275)" is the only dolphin commit after the previous review (`0a35a87c3`).

## Reviewed change: `0505dd44e` air supply (no gap)

- `dolphin.rs:350-352` `base_tick` now delegates `living_entity.tick_air_supply(caller)`
  (signature gained `caller`); the generic `Mob::base_tick` default
  (`mob/mod.rs:2134-2138`) is now the same call, so the dolphin override (and
  `DrownedEntity`'s) is redundant but behaviourally identical (not a parity gap).
- `living.rs:3723-3763` uses `caller.get_max_air_supply()` / `increase_air_supply()`;
  dolphin overrides (`dolphin.rs:342-348`) return `TOTAL_AIR_SUPPLY` 4800 for both, matching
  vanilla `Dolphin.getMaxAirSupply()` 4800 and `increaseAirSupply()` returning the max
  (instant refill out of water). `new()` seeds `air_supply` 4800 (`dolphin.rs:62`).
- Drain path still correct for dolphin: `can_breathe_underwater` comes from the
  `minecraft:can_breathe_under_water` entity tag (`tag.rs:21020-21055`), which does **not**
  list `dolphin` (it lists axolotl/cod/guardian/squid/turtle/nautilus/...), so a submerged
  dolphin drains 1 air/tick and drowns, as vanilla (dolphins are not water-breathing; their
  4800 supply is the long breath).
- `post_tick` no-AI refill to 4800 (`dolphin.rs:356-357`) still matches vanilla
  `aiStep` `isNoAi() -> setAirSupply(getMaxAirSupply())`. Tests `dolphin.rs:588-645` cover
  nautilus breathing and no-AI.

## Fixed in this pass

- None. The only delta since the last audit is behaviour-neutral, and the remaining gaps below
  are either data-blocked or need decompiled verification.

## Remaining gaps

### 1. Dolphin is modelled as an ageable/breedable mob (M)
- File: `crates/pumpkin-core/src/entity/passive/dolphin.rs:148-156` (`impl AgeableMob`),
  `:153-155` (0.1 baby spawn chance), `:317-326` (`finalize_spawn` ->
  `finalize_ageable_spawn`), `:328-340` (`mob_init_data_tracker` writes
  `ageable_mob::DATA_BABY_ID` / `AGE_LOCKED`), `:412-428` (`mob_interact` age-up branch),
  `:158` (`impl AgeableWaterCreature`), and `entity/type.rs:754`
  (`check_surface_ageable_water_creature_spawn_rules`).
- Vanilla: `Dolphin extends WaterAnimal`, which is not `AgeableMob`; dolphins have no baby
  variant, no breeding, no age lock. Feeding a dolphin only sets `GotFish`; it never ages up.
  The dolphin synched-data list is `GOT_FISH`, `MOISTNESS_LEVEL` only, so the extra
  `DATA_BABY_ID`/`AGE_LOCKED` payload is sent at indices the vanilla client does not define.
- Fix: take air/XP/spawn-obstruction helpers directly from `water_animal` (dolphin already
  does for the last two, `dolphin.rs:277-283`) instead of `AgeableWaterCreature`; drop
  `impl AgeableMob`, `ageable_data`, the ageable tracker writes, the age-up branch and
  `impl AgeableWaterCreature`. `squid.rs` has the same mistake, so change both together (or
  introduce a non-ageable `WaterCreature` helper). Size M.

### 2. Dolphin food uses the `fishes` tag instead of `minecraft:dolphin_food` (S)
- File: `crates/pumpkin-core/src/entity/passive/dolphin.rs:414`
- Vanilla `Dolphin.mobInteract` gates on `ItemTags.DOLPHIN_FOOD_ITEMS`
  (`cod`, `salmon`, `tropical_fish`), not `ItemTags.FISHES`. Pumpkin accepts
  `minecraft:fishes` (raw/cooked cod, raw/cooked salmon, pufferfish, tropical fish), so
  pufferfish and cooked fish are eaten, consumed and set `GotFish` where vanilla falls through
  to `super.mobInteract`.
- Fix blocked on data: `grep 'DOLPHIN_FOOD|dolphin_food'` in
  `crates/pumpkin-data/src/generated/tag.rs` still has no match, so the vanilla item tag is
  not extracted into `assets/` yet. Needs the Extractor/codegen tag dump before a data-driven
  check is possible; do not hardcode item ids. Size S once the tag exists.

### 3. `DolphinJumpGoal::should_continue` clause grouping unverified (S)
- File: `crates/pumpkin-core/src/entity/passive/dolphin/goals.rs:427-435`
- Pumpkin ORs four clauses (`dy^2 >= 0.03 || pitch == 0 || |pitch| >= 10 || !is_in_water()`)
  and ANDs `!on_ground`. Vanilla `Dolphin$DolphinJumpGoal.canContinueToUse` groups the
  vertical-speed test with the pitch/water clauses (single `&&`/negated conjunction), so the OR
  form keeps the goal alive in cases vanilla ends it (e.g. zero vertical speed with level
  pitch). `tick()` (`:459-482`) and the `start()`/pitch writes match vanilla and should not
  change.
- Fix: needs the decompiled `Dolphin$DolphinJumpGoal` (none in this worktree) to choose
  between `&&` and the negated conjunction; the previous audit flagged the same uncertainty, so
  do not guess. Size S (one-line condition + test).

### 4. `PlayWithItemsGoal` is split into two goals at priorities 7 and 8 (S)
- File: `crates/pumpkin-core/src/entity/passive/dolphin/goals.rs:78-81`
  (`MoveToItemGoal` at 7, `PlayWithItemsGoal` at 8), implementations `:570-639`.
- Vanilla registers a single `PlayWithItemsGoal` at priority 7 that both navigates to a nearby
  item and drops the held item on stop with a `tickCount + random(100)` cooldown. Pumpkin keeps
  the cooldown only on `MoveToItemGoal`; `PlayWithItemsGoal` has no cooldown and drops in both
  `tick` and `stop`. Observable behaviour is close (one MOVE goal at a time), but priorities
  differ from vanilla when both qualify.
- Fix: merge into one goal at 7 with vanilla `canUse` (cooldown branch OR held item) and a
  single drop in `stop`, re-checking the drop velocity/thrower against vanilla
  `Dolphin` item throw. Size S.

## Checked, no gap
- Air supply after `0505dd44e` (above): 4800 max, instant refill, tag-driven drowning, no-AI
  refill.
- Moistness: `post_tick` resets to 2400 in water/rain, decrements otherwise, `DRY_OUT` at 0,
  on-ground random hop (0.2/0.5/0.2, random yaw, clears on-ground, marks velocity dirty) matches
  vanilla `aiStep`.
- Synched data (`GOT_FISH`, `MOISTNESS_LEVEL`) and NBT keys (`GotFish`, `Moistness`) match
  vanilla save/read.
- Goal priorities 0/0/1/2/4/4/5/5/6 (`dolphin.rs:103-109`, `goals.rs:43-92`), guardian flee
  (8.0, 1.0, 1.0 at 9) and `RevengeGoal` ignoring `GUARDIAN_LIKE` match vanilla.
- Treasure goal: `gotFish && air >= 100` start/continue, clears fish on stop when
  none/reached/stuck, `dolphin_located` structure search radius 50, 1.3 nav speed, entity
  status 38 retry every `getTickCount(80)`, 16/1 then 8/4 then 8/5 water-validated positions.
- Swim-with-player: 10-block swimmer search, `getTarget() != player` gate, 256 continue radius,
  grace 100t on start and 1/6 per tick, stop inside 6.25, 4.0 nav speed.
- Item play: 8-block pickup-delay-0/alive/in-water filter, play sound, 1.2 nav speed, drop at
  eye height -0.3 with 0.3 scale velocity, 40t pickup delay, thrower uuid, plugin event.
- Attributes from codegen: max health 10, attack damage 3, movement speed 1.2, follow range 16,
  water-creature spawn restriction, 4/chunk limit, XP 2.


## Gap status
- 1 dolphin modelled as ageable (M): CLOSED (NO GAP - vanilla `Dolphin extends AgeableWaterCreature`; see re-verify Result)
- 2 dolphin_food tag: CLOSED (NO GAP - vanilla uses ItemTags.FISHES, not dolphin_food; see re-verify Result)
- 3 DolphinJumpGoal clause grouping: CLOSED (NO GAP - OR grouping confirmed by decompile; see re-verify Result)
- 4 PlayWithItemsGoal split priorities 7/8: CLOSED (NO GAP - two goals confirmed by decompile; see re-verify Result)


## Result

Re-review against the decompiled tree now present at
`.vendor/vanilla-decompiled/net/minecraft/` (MC source for this branch). All four rows were
verified smallest-first; every one resolves to NO GAP, so no source file was changed.

- Row 1 (ageable, M): NO GAP. Decompiled `Dolphin.java:74` declares
  `public class Dolphin extends AgeableWaterCreature`, and `mobInteract` calls `canAgeUp()` /
  `ageUp()` at `:328-330`. Pumpkin's `impl AgeableMob` + `impl AgeableWaterCreature`
  (`dolphin.rs:148-158`) and age-up branch (`:417-423`) match vanilla. The prior audit assumed
  `Dolphin extends WaterAnimal` with no breeding; that is false for this MC version, so the
  proposed de-ageable refactor (and the cross-crate `entity/type.rs` edit) must NOT be applied.
- Row 2 (FISHES vs dolphin_food, S): NO GAP. Decompiled `Dolphin.java:323` gates on
  `itemStack.is(ItemTags.FISHES)`; there is no `DOLPHIN_FOOD_ITEMS` check. Pumpkin uses
  `tag::Item::MINECRAFT_FISHES` (`dolphin.rs:414`), which matches. The BLOCKED-DATA
  `dolphin_food` tag is not needed; do not hardcode ids.
- Row 3 (DolphinJumpGoal clause grouping, S): NO GAP. Decompiled
  `DolphinJumpGoal.java:56-60` is exactly
  `(!(yd*yd < 0.03) || getXRot()==0 || !(abs(getXRot()) < 10) || !isInWater()) && !onGround()`,
  i.e. the OR grouping Pumpkin uses. Keep the current `should_continue` condition.
- Row 4 (PlayWithItemsGoal split priorities 7/8, S): NO GAP. Decompiled `Dolphin.java:176-177`
  registers `MoveToItemGoal` at 7 and `PlayWithItemsGoal` at 8 as two separate goals, with the
  cooldown only on `MoveToItemGoal` (`:559-604`) and `PlayWithItemsGoal` dropping in both
  `stop()` and `tick()` (`:606-621`). Pumpkin's split matches; the proposed merge into one goal
  at 7 would diverge from vanilla.
- Verification source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java`
  and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java`.
- Files touched: none (docs only). rustfmt not applicable. No unit test added because there is no
  code seam to guard.


## Result (re-verify 54a96a)

Independent re-check of every STILL OPEN row against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/`. All four resolve to NO GAP, so no
source file (dolphin.rs / squid.rs) was changed and no test seam exists.

- Row 1 (ageable, M): NO GAP. `Dolphin.java:74` declares
  `public class Dolphin extends AgeableWaterCreature`, and `mobInteract` calls
  `canAgeUp()` / `ageUp()` at `:328-330`. Pumpkin's `impl AgeableMob`
  (`dolphin.rs:148`) and `impl AgeableWaterCreature` (`:158`) match. The prior
  "extends WaterAnimal / no breeding" premise was false for this MC version;
  do NOT apply the de-ageable refactor or the `entity/type.rs` edit.
- Row 2 (FISHES vs dolphin_food, S): NO GAP. `Dolphin.java:323` gates on
  `itemStack.is(ItemTags.FISHES)`; Pumpkin uses
  `item.has_tag(&tag::Item::MINECRAFT_FISHES)` (`dolphin.rs:414`), which matches.
  Vanilla does not check `dolphin_food` here; do not hardcode item ids.
- Row 3 (DolphinJumpGoal grouping, S): NO GAP. `DolphinJumpGoal.java:56-59` is
  `(!(yd*yd < 0.03) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround()`,
  equivalent to Pumpkin's `>= 0.03 || pitch==0 || abs(pitch)>=10 || !is_in_water()`
  AND `!on_ground` (`goals.rs:427-435`). Keep the current condition.
- Row 4 (PlayWithItemsGoal split 7/8, S): NO GAP. `Dolphin.java:176-177` registers
  `MoveToItemGoal` at 7 with cooldown only in `stop()` (`:578-581`) and
  `PlayWithItemsGoal` at 8 dropping in both `stop()` and `tick()` (`:613-620`).
  Pumpkin's split matches; the proposed merge would diverge from vanilla.
- The Gap status section above is superseded: rows 1-4 are CLOSED (NO GAP).
- Verification source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java`
  and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java`.
- Files touched: doc only. rustfmt not applicable. No unit test added: no code seam to guard.


## Result (CODER re-verify)

No row is marked STILL OPEN: the Gap status section lists all four as CLOSED, and the only
"STILL OPEN" string is prose inside the prior Result. Independently re-checked all four against
the decompiled tree, smallest-first; every one is NO GAP, and current Pumpkin source has not
drifted. No source file (dolphin.rs / squid.rs) was changed.

- Row 1 (ageable, M): NO GAP. `Dolphin.java:74` is `public class Dolphin extends AgeableWaterCreature`;
  `mobInteract` calls `canAgeUp()` / `ageUp()` at `:328-330`. Pumpkin keeps `impl AgeableMob`
  (`dolphin.rs:148`), `impl AgeableWaterCreature` (`:158`), `finalize_ageable_spawn` (`:325`),
  ageable tracker writes (`:331-334`) and the age-up branch (`:417-418`). Matches; do NOT
  de-ageable or touch `entity/type.rs`.
- Row 2 (FISHES vs dolphin_food, S): NO GAP. `Dolphin.java:323` is `itemStack.is(ItemTags.FISHES)`;
  Pumpkin uses `item.has_tag(&tag::Item::MINECRAFT_FISHES)` (`dolphin.rs:414`). No `dolphin_food`
  check in vanilla; do not hardcode ids.
- Row 3 (DolphinJumpGoal grouping, S): NO GAP. `DolphinJumpGoal.java:56-59` is exactly
  `(!(yd*yd < 0.03F) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround()`,
  the OR grouping Pumpkin uses. Keep the current `should_continue`.
- Row 4 (PlayWithItemsGoal split 7/8, S): NO GAP. `Dolphin.java:176-177` registers
  `MoveToItemGoal` at 7 and `PlayWithItemsGoal` at 8 as two goals; cooldown only in
  `MoveToItemGoal.stop()` (`:578-581`), `PlayWithItemsGoal` drops in both `stop()` and `tick()`
  (`:613-620`). Pumpkin's split matches; the proposed merge would diverge.
- Files touched: doc only. rustfmt not applicable. No unit test added: no code seam to guard.


## Result (CODER re-verify 2)

No row is marked STILL OPEN (the Gap status section lists 1-4 CLOSED/NO GAP), so per task scope there is nothing to fix. Independently re-verified all four against the decompiled tree at `.vendor/vanilla-decompiled/net/minecraft/`, smallest-first; every row is NO GAP and current Pumpkin source has not drifted. No source file (dolphin.rs / squid.rs) changed; no test seam exists.

- Row 2 (FISHES vs dolphin_food, S): NO GAP. `Dolphin.java:323` is `itemStack.is(ItemTags.FISHES)`; Pumpkin `dolphin.rs:414` is `item.has_tag(&tag::Item::MINECRAFT_FISHES)`. No `dolphin_food` check in vanilla; do not hardcode ids.
- Row 3 (DolphinJumpGoal grouping, S): NO GAP. `DolphinJumpGoal.java:56-59` is `(!(yd*yd < 0.03F) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround()`, the OR grouping Pumpkin uses (`goals.rs:427-435`). Keep the current condition.
- Row 4 (PlayWithItemsGoal split 7/8, S): NO GAP. `Dolphin.java:176-177` registers `MoveToItemGoal` at 7 and `PlayWithItemsGoal` at 8 as two separate goals. Pumpkin's split matches; a merge would diverge.
- Row 1 (ageable, M): NO GAP. `Dolphin.java:74` is `public class Dolphin extends AgeableWaterCreature`, with `canAgeUp()`/`ageUp()` at `:328-330`. Pumpkin `impl AgeableMob` (`dolphin.rs:148`), `impl AgeableWaterCreature` (`:158`) and age-up branch (`:417-423`) match. Do NOT de-ageable or touch `entity/type.rs`.
- Verification source: `Dolphin.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java`.
- Files touched: doc only. rustfmt not applicable. No unit test added: no code seam to guard.


## Result (THIRD PASS - terminal VERIFIED)

Independent verification of all four rows against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/`. No row is marked STILL OPEN (the Gap status
section lists 1-4 CLOSED). Every row is VERIFIED: current Pumpkin source matches the
decompiled vanilla source, so no source edit and no unit test seam exists. These statuses
are terminal.

- Row 1 (ageable, M): VERIFIED. `.vendor/.../animal/dolphin/Dolphin.java:74` is
  `public class Dolphin extends AgeableWaterCreature`; `mobInteract` calls
  `canAgeUp()`/`ageUp()` at `:328-330` and `finalizeSpawn` uses a 0.1F `AgeableMobGroupData`
  at `:103`. Pumpkin matches: `impl AgeableMob` (`dolphin.rs:148`),
  `impl AgeableWaterCreature` (`:158`), `finalize_ageable_spawn` (`:325`), ageable tracker
  writes (`:331-334`), age-up branch (`:417-418`). Do NOT de-ageable or touch `entity/type.rs`.
- Row 2 (FISHES vs dolphin_food, S): VERIFIED. `Dolphin.java:323` is
  `itemStack.is(ItemTags.FISHES)`; there is no dolphin_food check. Pumpkin `dolphin.rs:414`
  uses `item.has_tag(&tag::Item::MINECRAFT_FISHES)`. Matches; no generated field needed,
  do not hardcode ids.
- Row 3 (DolphinJumpGoal grouping, S): VERIFIED. `.vendor/.../ai/goal/DolphinJumpGoal.java:56-59`
  is `(!(yd*yd < 0.03F) || getXRot()==0 || !(abs(getXRot())<10) || !isInWater()) && !onGround()`,
  the OR grouping Pumpkin uses at `goals.rs:427-435`. Keep the current condition.
- Row 4 (PlayWithItemsGoal split 7/8, S): VERIFIED. `Dolphin.java:176-177` registers
  `MoveToItemGoal` at 7 and `PlayWithItemsGoal` at 8 as two separate goals; cooldown only in
  `MoveToItemGoal.stop()` (`:578-581`), `PlayWithItemsGoal` drops in `stop()` and `tick()`
  (`:613-620`). Pumpkin matches (`goals.rs:69-70`, cooldown at `:576`/`:596`). A merge would
  diverge from vanilla.
- Verification source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/dolphin/Dolphin.java`
  and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java`.
- Files touched: doc only (Result appended). rustfmt not applicable. No unit test: no code
  seam, all four rows match vanilla.
