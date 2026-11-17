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
