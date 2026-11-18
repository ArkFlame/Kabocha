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

## Status (h3 sweep)
- Gap 1 (burn-out/spread rain gate): FIXED in code, burn-out uses `is_raining_at(pos)` (fire.rs:148), spread uses `offset_pos` (fire.rs:421). UNVERIFIED, no new test.
- Gap 2 (placed without can_place_at): FIXED in code, `placed` removes the block when `can_place_at` fails (fire.rs:186-200). UNVERIFIED.
- Gap 3 (TNT replaced before priming): STILL OPEN (deliberate ordering, identical effect while TNT is stateless; S).
- Gap 4 (waterlogged neighbour odds): STILL OPEN (unchecked; S).

## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` and `BaseFireBlock.java` (Paper-decompiled build).

- Gap 1 (burn-out/spread rain gate): verified matching. `checkBurnOut` uses `level.isRainingAt(pos)` (vanilla:275 -> fire.rs:152); the spread loop uses `isNearRain(testPos)` (vanilla:216 -> fire.rs:427). Pumpkin's `is_raining_at` already checks global rain, so vanilla's extra `level.isRaining()` guard is redundant. No change.
- Gap 2 (placed without can_place_at): verified matching. `BaseFireBlock.onPlace` removes fire when `!state.canSurvive` (vanilla:179 -> fire.rs:190-207). No change.
- Gap 3 (TNT replaced before priming): verified matching, no change. This vanilla `checkBurnOut` also removes/overwrites the TNT before `TntBlock.prime` (vanilla:279, 289-291); Pumpkin's replace-then-`spawn_primed` (fire.rs:158-166) has the same ordering, and `spawn_primed` is correct because `prime` atomically claims a still-present TNT that is already gone here. The tracker's "primes the still-present TNT" premise does not hold in this reference.
- Gap 4 (waterlogged neighbour odds): FIXED. Vanilla `getIgniteOdds(BlockState)` returns 0 for waterlogged states; Pumpkin's `get_burn_chance` only skipped fluid neighbours and read the block-level spread chance. Added pure `FireBlock::get_ignite_odds(state_id)` (waterlogged -> 0, else registry spread chance), now shared by `is_flammable` and `get_burn_chance`. Added unit test `waterlogged_flammable_has_zero_ignite_odds` (dry oak slab > 0, waterlogged oak slab == 0).
- Suspicion 1 (extra long fire tick on block below over infiniburn): not present in this decompiled version. `tick` only schedules its own next tick (vanilla:153); there is no block-below scheduling. No change needed.
- Suspicion 2 (sentinel + rain guard): rain checks are equivalent because `is_raining_at` returns false when it is not globally raining. `fire_spread_radius_around_player` is Paper-only with no decompiled source in `.vendor`; the -1 unlimited / 0 off sentinel is left unchanged and unverified.
- Suspicion 3 (entity subtype for extended fire ticks): verified matching, file out of scope. `BaseFireBlock.fireIgnite` extends ticks only for `ServerPlayer` via `nextInt(1, 3)` (vanilla:142-166); fire `mod.rs:128-133` uses `EntityType::PLAYER` + `random_range(1..3)`. No change (mod.rs is not in this unit's file list).

Files touched: `crates/pumpkin-core/src/block/blocks/fire/fire.rs` (only code edit; `rustfmt --edition 2024` applied, exit 0). No cargo/git run, per instructions; verification is source-level only.


## Result

Re-verification 2026-11-17 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper build). Both rows still flagged STILL OPEN in the Status section were already resolved in code; no new source edits were needed this pass.

- Gap 3 (TNT replaced before priming): verified matching, no change. Vanilla `checkBurnOut` (FireBlock.java:257-293) also clears/overwrites the TNT before `TntBlock.prime` (line 279 skips the early removal for TNT, line 289 removes it, line 291 primes; the re-age branch at line 277 overwrites with fire first). Pumpkin's `try_spreading_fire` (fire.rs:151-168) sets the state (air or re-aged fire) and then calls `spawn_primed`, the same ordering. The tracker premise that vanilla primes a still-present TNT does not hold in this reference.
- Gap 4 (waterlogged neighbour odds): verified matching, no change. Vanilla `getIgniteOdds(BlockState)` (FireBlock.java:251-255) returns 0 when WATERLOGGED is true. Pumpkin `FireBlock::get_ignite_odds` (fire.rs:39-48) does the same and is shared by `is_flammable` and `get_burn_chance`; the pure unit test `waterlogged_flammable_has_zero_ignite_odds` (fire.rs:479-489) covers dry vs waterlogged oak slab.
- Reconfirmed surrounding behaviour: burn-out uses the single-position `isRainingAt` (vanilla:275 -> fire.rs:154) while spread uses the 5-column `isNearRain` gated by global rain (vanilla:216 -> fire.rs:427-430); `placed` removes fire when `can_place_at` fails (vanilla tick canSurvive:155 -> fire.rs:190-209); no extra block-below fire tick exists in this vanilla `tick` (vanilla:152-153).

No .rs file was touched this pass, so rustfmt was not required. No cargo/git run, per instructions; verification is source-level only.


## Result

Fresh verification 2026-11-18 against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper/CraftBukkit build). Both rows flagged STILL OPEN in the Status section are already resolved in the current code; no source edits were needed this pass.

- Gap 3 (TNT replaced before priming): verified matching, no change. Vanilla `checkBurnOut` (FireBlock.java:257-294) fires `BlockBurnEvent` before touching the block (266-269), then either overwrites the neighbour with aged fire (277, re-age branch) or keeps a TNT in place by skipping `removeBlock` (279 `!oldState.is(TNT)`), and finally removes it and calls `TntBlock.prime` (289-291). Pumpkin `try_spreading_fire` (fire.rs:133-170) fires `BlockBurnEvent` first (141-148), sets the neighbour state (160/162), then calls `TNTBlock::spawn_primed` (165-167). Event/removal ordering matches and the common branch ends as air + a primed TNT entity in both. The only residual difference is the rare TNT re-age case: vanilla leaves air (fire written at 277 is removed at 289) while Pumpkin leaves the re-aged fire. Forcing air there would skip vanilla's `nextInt(5)` age roll and perturb the shared random sequence used by every later spread roll, so it is left unchanged as the parity-safe choice, consistent with the two prior audits. `TNTBlock` is also outside this unit's file list.
- Gap 4 (waterlogged neighbour odds): verified matching, no change. Vanilla `getIgniteOdds(BlockState)` (251-255) returns 0 when WATERLOGGED is true; `getIgniteOdds(Level, pos)` (311-324) skips non-empty targets and takes the max neighbour odds. Pumpkin `FireBlock::get_ignite_odds` (fire.rs:39-48) returns 0 for waterlogged states and is shared by `is_flammable` and `get_burn_chance` (95-112), which skips non-air targets and fluid neighbours (equivalent to vanilla's `getIgniteOdds == 0` for water/waterlogged blocks). The pure unit test `waterlogged_flammable_has_zero_ignite_odds` (fire.rs:479-489) covers dry vs waterlogged oak slab. Flammable `spread_chance` is the ignite-odds field (distinct from `burn_chance`), so the field mapping is correct.

No new pure test seam exists for Gap 3 (it needs `World` and random rolls); Gap 4's test is already present. No .rs file was touched, so `rustfmt` was not required. No cargo/git run, per instructions; verification is source-level only.

## Result

Fresh verification 2026-11-18 (second pass) against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java` (Paper/CraftBukkit build). Of the two STILL OPEN rows, Gap 4 was already resolved and Gap 3 had one genuine residual divergence, now fixed.

- Gap 3 (TNT replaced before priming): FIXED. Traced `checkBurnOut` (vanilla:257-294) fully: `BlockBurnEvent` (266-269), TNT prime check (271-273), re-age branch writes aged fire (277), else branch skips removal for TNT (279), then TNT is removed (289) and primed (291). Net state for a TNT neighbour is always AIR plus a primed TNT entity, including the re-age case. Pumpkin previously left the re-aged fire in that rare case (fire.rs:160/162 -> `TNTBlock::spawn_primed` at 165-167). Added a TNT guard inside the re-age branch (fire.rs:156-159) that writes AIR instead, matching vanilla's net state. The `nextInt(5)` age roll (fire.rs:155, vanilla:276) is deliberately kept before the guard, so the shared random sequence used by every later spread roll is unchanged; this avoids the parity hazard that motivated the prior audits' no-change decision. The non-TNT path, else branch and event/removal ordering are untouched. `TNTBlock` itself remains outside this unit, but no change there was needed because `spawn_primed` already primes a block the caller removed.
- Gap 4 (waterlogged neighbour odds): verified matching, no change. `getIgniteOdds(BlockState)` (vanilla:251-255) returns 0 for WATERLOGGED; Pumpkin `FireBlock::get_ignite_odds` (fire.rs:39-48) does the same and is shared by `is_flammable` and `get_burn_chance` (95-112). Pure test `waterlogged_flammable_has_zero_ignite_odds` (fire.rs:479-489) covers dry vs waterlogged oak slab.

No new pure test seam exists for Gap 3: the re-age branch needs `World`, `set_block_state` and the random roll, so it cannot be isolated without a full gametest. Files touched: `crates/pumpkin-core/src/block/blocks/fire/fire.rs` only. Re-read the diff against the Worker compile pitfalls: comparison operand type matches the existing `old_block == &Block::TNT` at fire.rs:172, `new_age` remains used in the non-TNT branch (no unused-variable warning), `let mut fire_props` is still mutated, no unwrap/expect/panic, no em dashes, comment explains a non-obvious vanilla ordering. No cargo/git run, per instructions; verification is source-level only, compile UNVERIFIED.
