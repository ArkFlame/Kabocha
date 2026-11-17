# blocks / Fire

Audit, 2026-11-16, branch kab/tblocks-fire. Reviewed revision: 78b6d96c0 (includes the recent fire
fixes 96093cfc7 #4451 and b388d9d7a #4478).

## Source map

| File | Class it ports |
|---|---|
| crates/pumpkin-core/src/block/blocks/fire/fire.rs | FireBlock |
| crates/pumpkin-core/src/block/blocks/fire/mod.rs | FireBlock/BaseFireBlock shared helpers (portal, soul base, collision) |
| crates/pumpkin-core/src/block/blocks/fire/soul_fire.rs | SoulFireBlock |
| crates/pumpkin-core/src/item/items/ignite/ignition.rs | FlintAndSteelItem/FireChargeItem "place fire" path |
| world/explosion.rs:390, entity/lightning.rs:111, entity/projectile/small_fireball.rs:118 | other fire placement call sites |

## Verified matching vanilla

- get_fire_tick_delay = 30 + rand(0..10) (fire.rs:33), rescheduled first thing in the tick (fire.rs:234).
- Placement state: solid/flammable face below -> plain fire, else per-direction north/east/.../up
  shape (fire.rs:58-85), mirrors FireBlock.getStateForPosition.
- getStateForNeighborUpdate keeps the existing age and drops to air when unsupported
  (fire.rs:196-219), mirrors FireBlock.updateShape.
- canSurvive = sturdy face below OR flammable face neighbour (fire.rs:221-227).
- Rain extinguish roll 0.2 + age*0.03, age increment min(15, age + rand(0..3)/2), age == 15
  self-extinguish at max age on non-flammable ground, age > 3 unsupported removal. All use the
  pre-increment age, as vanilla does (fire.rs:278-316).
- Burn-out odds rand(0..300|250) < burnOdds, fire re-age rand(0..age+10) < 5, growth
  + rand(0..5)/4 (fire.rs:127-163), mirrors FireBlock.checkBurnOut.
- Whole 3x3x6 spread loop: ignite odds = max neighbour spread chance, rate = 100 or 100+(y-1)*100,
  odds = (ignite + 40 + difficulty*7)/(age+30), biome burnout halving, -50 chance modifier under
  gameplay/increased_fire_burnout (fire.rs:318-443).
- Nether portal creation from fire on place, only Overworld/Nether, X axis (fire.rs:168-190).
- Entity damage: normal fire 1.0, soul fire 2.0, fire-immune skipped, set_on_fire_for(8s)
  (mod.rs:120-147).
- Soul fire: no age/spread behaviour, only soul-base survival and 2.0 damage (soul_fire.rs).

## Fixed in this audit

1. fire.rs:260. Infiniburn was keyed off hardcoded Dimension::OVERWORLD/THE_NETHER/THE_END ids with
   hand-picked tags. Vanilla FireBlock.tick uses
   level.getBlockState(pos.below()).is(level.dimensionType().infiniburn()), i.e. the tag declared by
   the dimension type, so any custom data-driven dimension got false. Now resolved through
   Block::is_tagged_with(world.dimension.infiniburn); the Dimension struct already carries the tag
   name. Size S, no behaviour change for the three built-in dimensions.

## Remaining gaps

1. fire.rs:147 and fire.rs:408. Neighbour burn-out and spread both gate on is_near_rain (5 columns:
   pos, west, east, north, south). Vanilla FireBlock.checkBurnOut uses level.isRainingAt(pos) for the
   single position; the 5-column form is only used for the fire's own block. Effect: fire under a
   1-block roof next to exposed rain stops spreading or burning out. Fix: use
   world.is_raining_at(neighbour_pos) in both places. Size S.
2. fire.rs:184-189. placed schedules the next fire tick without checking can_place_at. Vanilla
   FireBlock.onPlace removes the block when !state.canSurvive(level, pos) and only schedules
   otherwise. A fire set into an unsupported spot currently lives until its first tick. Fix:
   can_place_at check, remove instead of scheduling. Size S.
3. fire.rs:145-162. TNT is replaced with air first and primed afterwards (comment at fire.rs:159
   admits it). Vanilla primes the still-present TNT block in place. Effect is identical today because
   TNT carries no state, but it changes BlockBurnEvent/removal ordering; keep aligned with
   TNTBlock.prime if TNT ever gains state. Size S.
4. fire.rs:88-106 vs fire.rs:37-45. Spread odds (get_burn_chance) read block-level spread_chance and
   only skip fluid neighbours, while survival (is_flammable) also rejects waterlogged states. Vanilla
   getIgniteOdds is block-registry based too, so this probably matches; listed so the next reviewer
   checks waterlogged wood next to air. Size S.

## Unverified suspicions (need a decompiled reference)

- Vanilla FireBlock.tick schedules an extra long fire tick on the block below while the fire sits on
  an infiniburn block (fire-on-fire / netherrack stacking). fire.rs:297-316 has no equivalent.
- Sentinel semantics of fire_spread_radius_around_player (fire.rs:364-392 treats -1 as unlimited and
  0 as disabled), and whether vanilla also requires level.isRaining() for the neighbour rain checks.
- mod.rs:128-133 extends fire ticks only for PLAYER with rand(1..3); confirm the entity subtype check
  against BaseFireBlock.entityInside in the targeted version.
