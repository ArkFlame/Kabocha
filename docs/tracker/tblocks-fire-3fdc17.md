# blocks / Fire

Re-review, 2026-11-17, branch kab/tblocks-fire-3fdc17. Supersedes `docs/tracker/tblocks-fire.md`
(2026-11-16 audit at 78b6d96c0).

Tracker note: the reviewed source changed after that audit. Two commits touched
`crates/pumpkin-core/src/block/blocks/fire/fire.rs`:

- infiniburn now resolved through `world.dimension.infiniburn` + `Block::is_tagged_with`
  (fire.rs:262-267) instead of hardcoded dimension ids. Correct: vanilla
  `FireBlock.tick` does `level.getBlockState(pos.below()).is(level.dimensionType().infiniburn())`.
- a new `if age >= 15 { return; }` gate before neighbour burning/spreading (fire.rs:335, from
  upstream #3772). Not covered by the previous audit; see Remaining gaps 1.

## Source map

| File | Class it ports |
|---|---|
| crates/pumpkin-core/src/block/blocks/fire/fire.rs | FireBlock |
| crates/pumpkin-core/src/block/blocks/fire/mod.rs | BaseFireBlock shared helpers (portal, soul base, collision) |
| crates/pumpkin-core/src/block/blocks/fire/soul_fire.rs | SoulFireBlock |
| crates/pumpkin-core/src/item/items/ignite/ignition.rs:49 | FlintAndSteel/FireCharge place-fire path |
| entity/lightning.rs:111, world/explosion.rs:390, entity/projectile/small_fireball.rs | other fire placement sites |

## Verified matching vanilla

- `get_fire_tick_delay` = 30 + rand(0..10) (fire.rs:33); tick reschedules first (fire.rs:256).
- onPlace: same-block guard, portal attempt in Overworld/Nether on X axis, unsupported position
  removed instead of scheduled (fire.rs:168-209, fixed in this review).
- `getStateForPosition`: flat fire on solid/flammable ground, else per-direction shape
  (fire.rs:58-85); `updateShape` keeps age, drops to air when unsupported (fire.rs:219-242).
- `canSurvive`: sturdy face below OR flammable face neighbour (fire.rs:244-250).
- Own-block rain: `0.2 + age*0.03` roll with 5-column `isNearRain` (fire.rs:108-114, 291-299), age
  `min(15, age + rand(0..3)/2)` (fire.rs:301-307), pre-increment age reused afterwards.
- `checkBurnOut`: `rand(0..chance) < burn_odds`, re-light `rand(0..age+10) < 5` (single-column rain,
  see fix below), growth `+ rand(0..5)/4`, TNT primed after replacement (fire.rs:127-164).
- 3x3x6 spread loop: ignite odds = max neighbour spread chance, rate `100` / `100+(y-1)*100`,
  `odds = (ignite + 40 + difficulty*7)/(age+30)`, increased-burnout halving and `-50` burn
  modifier, gamerule radius gate (fire.rs:340-462).
- Entity damage: fire 1.0, soul fire 2.0, immune skipped, `set_on_fire_for(8.0)` (mod.rs:120-147).
- Soul fire: no age/spread, soul-base survival only, 2.0 damage (soul_fire.rs).

## Fixed in this re-review

1. fire.rs:146-149 and 405-408. `checkBurnOut` re-light and the spread loop used the 5-column
   `is_near_rain` helper. Vanilla `FireBlock.checkBurnOut` uses `!level.isRainingAt(pos)` for the
   single target position, and the spread loop tests `!level.isRainingAt(neighbourPos)`; only the
   fire's own extinguish check uses `isNearRain`. Effect: fire under a 1-block roof beside an
   exposed column stopped re-lighting and stopped spreading. Now `world.is_raining_at(target)`.
   Size S.
2. fire.rs:186-207. `placed` scheduled the next tick without checking survival. Vanilla
   `FireBlock.onPlace` removes the block when `!state.canSurvive(level, pos)` and only schedules
   otherwise. A fire set into an unsupported position lived until its first tick. Now checks
   `can_place_at` and removes with NOTIFY_ALL. Size S.

## Remaining gaps

1. fire.rs:335. New `if age >= 15 { return; }` skips neighbour burning and the spread loop for
   max-age fire. Vanilla's age only feeds the odds denominator `(age + 30)` and the growth rolls;
   whether current vanilla gates the whole burn/spread block on `age < 15` (upstream #3772 claims
   it does) must be checked against the decompiled `FireBlock.tick` of the targeted version.
   If it does not exist, remove the gate. Size S. Needs reference.
2. fire.rs:88-106 vs 37-45. Spread odds (`get_burn_chance`) read block-level `spread_chance` and
   skip fluid neighbours, while survival (`is_flammable`) also rejects waterlogged states.
   Vanilla `getIgniteOdds` asks `getFlammability` per face; waterlogged flammable blocks deserve a
   check. Size S, unverified.
3. fire.rs:64-68. `getStateForPosition` treats a flammable block below as flat ground and uses
   `is_side_solid(Up)` as the face test. Vanilla uses `isFaceFull(collisionShape, UP)` in current
   versions (older versions also OR'd a flammable-below check). Partial-shape blocks below (top
   slab, fence, soul-sand-like) may yield a flat state where vanilla yields a shaped one. Size S,
   unverified.
4. fire.rs:163. TNT is replaced with air before `TNTBlock::spawn_primed` runs (comment admits it).
   Net effect matches vanilla because the prime path reads nothing from the world, but ordering
   differs from `checkBurnOut` if TNT ever gains state. Size S, low value.
5. mod.rs:126-133. Fire tick timer: negative ticks incremented by 1, players get `rand(1..3)`
   extra. Confirm against `BaseFireBlock.entityInside` / `Entity.baseTick` of the targeted version;
   the player-only randomness needs a reference. Size S.
6. mod.rs:51-81. `can_place_at` always offers `Block::FIRE.default_state` as the state and ORs
   `should_light_portal_at(Up)`, while vanilla `FireBlock.canSurvive` is shape-independent and
   portal creation only runs from `onPlace`. Cosmetic today; check if any caller relies on the
   `Up` portal variant. Size S, unverified.

## Unverified suspicions

- Whether vanilla still burns neighbours at age 15 and whether `level.isRaining()` is separately
  required in `checkBurnOut` (currently covered by `is_raining_at`).
- `fire_spread_radius_around_player` sentinel handling (fire.rs:379-407: -1 unlimited, 0 disabled).

## Status (code-verified sweep)
FIXED: fire tick delay is 30 + rand(0..10) (block/blocks/fire/fire.rs:33-34). STILL OPEN: getStateForPosition shape and canSurvive neighbour rule not re-read.


## Result

Re-review against `.vendor/vanilla-decompiled/.../FireBlock.java` + `BaseFireBlock.java`.

- STILL OPEN "canSurvive neighbour rule": FIXED. `FireBlock::is_flammable` (fire.rs:37)
  used `burn_chance` (vanilla burnOdds, only used by checkBurnOut). Vanilla `canBurn`
  uses ignite odds (`spread_chance`, the field the spread loop already reads). Switched
  to `spread_chance`. No current data has one field zero and the other nonzero, so no
  behaviour change today, but it now matches vanilla semantics exactly.
- Gap 1 (age >= 15 gate): already resolved. Current fire.rs has no such gate; the
  age==15 extinguish chance at fire.rs:321-328 matches vanilla `FireBlock.tick:181`.
- Gap 2 (spread odds / rain): FIXED. Decompiled `FireBlock.tick:216` gates the spread
  loop on `isNearRain(testPos)` (5 columns), but the prior re-review had replaced it
  with single-column `is_raining_at`. Restored `Self::is_near_rain`. The
  `!isRaining() ||` term in vanilla is redundant because is_raining_at already
  short-circuits when it is not raining globally. Waterlogged flammable neighbours
  are still excluded by the existing fluid skip.
- Gap 3 (getStateForPosition shape): verified matching, no change. `is_side_solid(Up)`
  is documented in pumpkin-data as Java `isFaceSturdy`; vanilla `getStateForPlacement`
  uses `isFaceSturdy(UP)`, not `isFaceFull`. The doc's isFaceFull concern does not
  apply to this version.
- STILL OPEN "getStateForPosition shape": verified matching (see Gap 3), no change.
- Gap 4 (TNT replace ordering): left open. Net effect matches vanilla and it is low
  value; changing it needs TNTBlock, a file outside this unit.
- Gap 5 (fire tick timer, mod.rs): left open. mod.rs is outside this unit's files.
- Gap 6 (can_place_at portal variant, mod.rs): left open. mod.rs is outside this unit's files.
- Suspicion (fire_spread_radius_around_player sentinel): left open, no code seam to
  verify without a runtime test; behaviour is a Pumpkin-only gamerule.

Files touched: crates/pumpkin-core/src/block/blocks/fire/fire.rs (rustfmt --edition 2024 OK).


## Result (re-verification, this pass)

Re-checked the two STILL OPEN rows from the code-verified sweep against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java`
(`getStateForPlacement`:126-143, `canSurvive`:146-149, `canBurn`:326-329,
`getIgniteOdds`:251-255 and 311-324). Both already match vanilla in the current
fire.rs, so no code change was needed this pass.

- STILL OPEN "canSurvive neighbour rule": verified matching. `is_flammable`
  (fire.rs:50-52) delegates to `get_ignite_odds` (fire.rs:39-48), which reads
  flammable.spread_chance and returns 0 for waterlogged states. This matches
  vanilla `canBurn` -> `getIgniteOdds`, which also uses ignite odds (spread
  chance), not burn odds. No change.
- STILL OPEN "getStateForPosition shape": verified matching. fire.rs:73-75
  returns the flat default state when the block below is flammable or is
  face-solid on `Up`, otherwise the per-direction shaped state. `is_side_solid`
  is documented as Java `isFaceSturdy` in pumpkin-data block_state.rs:251-253,
  so this matches vanilla `getStateForPlacement`. No change.
- Remaining gap 4 (TNT replace ordering, fire.rs:165-168): left open. Net effect
  matches vanilla `checkBurnOut`:258-292 (air plus primed TNT in both branches);
  an exact ordering fix would touch TNTBlock, a file outside this unit.
- Remaining gap 5 (fire tick timer, mod.rs:126-133): left open. mod.rs is outside
  this unit's files.
- Remaining gap 6 (can_place_at portal variant, mod.rs:79): left open. mod.rs is
  outside this unit's files.
- Suspicion (fire_spread_radius_around_player sentinel, fire.rs:383-411): left
  open. This is a Pumpkin-only gamerule with no decompiled-vanilla seam; it
  needs a runtime test to verify.

No .rs files were touched this pass (the fixes from the earlier pass are
already present in fire.rs), so rustfmt was not applicable. The existing unit
test `waterlogged_flammable_has_zero_ignite_odds` (fire.rs:479-489) already
covers the ignite-odds seam.


## Result (this pass)

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java`
(`tick`:152-235, `checkBurnOut`:257-294, `getStateForPlacement`:126-143,
`canSurvive`:146-149) and `world/mod.rs::is_raining_at`:3801-3813.

- Gap 1 (age >= 15 gate): verified matching, no change. Current tick has no
  blanket gate; the `age == 15` extinguish roll at fire.rs:331-337 matches
  vanilla `FireBlock.tick:181`, and the burn/spread loop still runs at max age.
- Gap 2 (spread odds / rain): verified matching, no change. fire.rs:432 uses
  `Self::is_near_rain` in the spread loop, matching vanilla `tick:216`. The
  missing `!level.isRaining()` term is redundant: `is_raining_at` returns false
  immediately when it is not raining globally (world/mod.rs:3802).
- Gap 3 (getStateForPosition shape): verified matching, no change. fire.rs:73
  returns the flat state when the block below is flammable or face-solid Up;
  `is_side_solid` is Java `isFaceSturdy`, matching vanilla `getStateForPlacement:129`.
- Gap 4 (TNT replace ordering): FIXED. Vanilla `checkBurnOut:275-292` always
  removes TNT before `TntBlock.prime`, even when the re-light roll succeeds;
  the old code left a fire block behind on that path. `try_spreading_fire` now
  routes TNT to the air branch (`relight && old_block != &Block::TNT`) before
  `spawn_primed`, so TNT ends as air + primed entity in every branch. A
  redundant nested TNT check from a concurrent edit was consolidated into this
  single guard.
- Gap 5 (fire tick timer, mod.rs): left open, mod.rs is outside this unit.
- Gap 6 (can_place_at portal variant, mod.rs): left open, mod.rs is outside
  this unit.
- Suspicion (isRaining in checkBurnOut): verified, no change. `is_raining_at`
  short-circuits on `!is_raining()`, so the single-column test covers it.
- Suspicion (fire_spread_radius_around_player sentinel): left open, Pumpkin-only
  gamerule with no decompiled-vanilla seam; needs a runtime test.

Files touched: crates/pumpkin-core/src/block/blocks/fire/fire.rs. No pure unit
test seam exists for the TNT branch (it needs `World`, `Arc` and plugin events);
the existing `waterlogged_flammable_has_zero_ignite_odds` test still covers the
ignite-odds seam. Cannot compile as a worker; orchestrator verifies.


## Result (third pass)

All remaining STILL OPEN rows resolved against decompiled vanilla.

- Gap 5 (fire tick timer, mod.rs:120-147): VERIFIED matches vanilla, no change.
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/BaseFireBlock.java:142-166`
  (`fireIgnite`): negative ticks `+1`, otherwise `ServerPlayer` adds
  `random.nextInt(1, 3)` (= 1 or 2, Rust `random_range(1..3)`), then `>= 0`
  ignites for 8 seconds. `apply_fire_collision` implements this exactly.

- Suspicion (`fire_spread_radius_around_player` sentinel, fire.rs:154-182):
  VERIFIED matches vanilla, no change.
  `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerLevel.java:4741-4744`
  is `spreadRadius == -1 || anyPlayerCloseEnoughTo(pos, spreadRadius)`.
  `ChunkMap.java:777-787,869-876` excludes spectators and uses
  `player.position().distanceTo(pos) < maxDistance` from the block corner
  (`new Vec3(pos)`); Pumpkin's `!is_spectator()` and squared-distance `< r^2`
  from the integer position are equivalent. `radius == -1` unlimited and
  `radius <= 0` disabled match.

- Gap 6 (can_place_at portal variant, mod.rs:51-81): FIXED. The doc's concern
  was real: this vanilla version passes the player's horizontal facing / the
  dispenser facing into `canBePlacedAt`, not `Up`:
  `.vendor/.../world/item/FlintAndSteelItem.java:34` and
  `FireChargeItem.java:38` pass `context.getHorizontalDirection()`;
  `.vendor/.../core/dispenser/FlintAndSteelDispenseItemBehavior.java:36`
  passes the dispenser `facing`. `BaseFireBlock.java:202-225` (`canBePlacedAt`
  -> `isPortal`) uses that direction to pick the preferred portal axis
  (horizontal -> counter-clockwise axis, else random horizontal). Hardcoded
  `Up` made Pumpkin pick a random axis for item-use, so side-facing portal
  lighting could fail ~50% of the time. Threaded the real direction through:
  `FireBlockBase::can_place_at` now takes `direction`, `Ignition::ignite_block`
  takes `direction` and forwards it, flint & steel and fire charge pass
  `player.get_entity().get_horizontal_facing().to_block_direction()`, the
  dispenser passes `ctx.facing.to_block_direction()`, and lightning (which has
  no facing and never tries portals in vanilla) passes `Up` to keep prior
  behaviour.

No pure unit-test seam exists for these rows: all three paths need `World`,
`Arc`, player lists or entity collision state, so none was added. Cannot
compile as a worker; orchestrator verifies with `scripts/integrate.sh`.


WRITE_TEST_OK
