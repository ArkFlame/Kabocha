# Parity rows for crates/pumpkin-core/src/block/blocks/fire/fire.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Fire** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-fire-3fdc17** [partial/OWN-OPEN] own audit: 7 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-fire-3fdc17.md) docs/tracker/tblocks-fire-3fdc17.md
- **own/tblocks-fire** [partial/OWN-OPEN] own audit: 5 open, 5 fixed, 4 verified, 0 blocked-data (docs/tracker/tblocks-fire.md) docs/tracker/tblocks-fire.md

## Result
Third pass (2026-11-18). All rows closed, none left STILL OPEN.

- **blocks/Fire** [partial/AUDITED]: FIXED. The last remaining open parity seam
  was explosion-created fire: `world/explosion.rs::place_fire` called
  `FireBlockBase::get_fire_type` plus `get_state_for_position`, which built
  `FireLikeProperties` from a soul fire state id, so an explosion over a soul
  base produced an invalid soul fire shape state. Routed through
  `FireBlock::get_placement_state(world, pos, 0)`. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/level/ServerExplosion.java:672-681`
  (`createFire`) uses `BaseFireBlock.getState`, which is exactly that helper
  (soul fire default on a soul base, plain fire with age 0 otherwise). The
  other fire placement sites (lightning, small fireball, ignition) were already
  routed through it.

- **own/tblocks-fire-3fdc17** [partial/OWN-OPEN]: VERIFIED against decompiled
  vanilla. Its final Result (doc lines 198-238) resolved every remaining
  STILL OPEN row: fire tick timer verified against
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/BaseFireBlock.java:142-166`,
  fire-spread-radius sentinel verified against
  `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerLevel.java:4741-4744`
  and `ChunkMap.java:777-787,869-876`, portal facing fixed against
  `.vendor/vanilla-decompiled/net/minecraft/world/item/FlintAndSteelItem.java:34`,
  `FireChargeItem.java:38` and
  `core/dispenser/FlintAndSteelDispenseItemBehavior.java:36`.

- **own/tblocks-fire** [partial/OWN-OPEN]: FIXED. Its third-pass Result
  (doc lines 125-135) closed every row except the explosion fire-placement
  item it explicitly left for a later pass (line 135); that item is fixed here.
  The other rows (tick radius gate, soul fire placement state, TNT before
  priming, waterlogged odds, sentinels) were already resolved in that pass.

Files touched this pass: `crates/pumpkin-core/src/world/explosion.rs`
(import simplification plus one call-site change). No pure unit test seam
exists for `place_fire` (needs `World`, `Arc` and event firing); the existing
`waterlogged_flammable_has_zero_ignite_odds` test still covers the ignite-odds
seam. rustfmt is run by the orchestrator; compile UNVERIFIED by worker.


### Final pass verification (2026-11-18, no source edits needed)

Re-read the current sources; every fix from earlier passes is present and matches decompiled vanilla. No row remains open; no row needs a generated pumpkin-data field or a missing subsystem (BLOCKED-DATA: none).

| Row | Final disposition | Vanilla evidence (all under .vendor/vanilla-decompiled/net/minecraft/) |
|---|---|---|
| blocks/Fire tick delay 30 + rand(0..10) | VERIFIED | world/level/block/FireBlock.java:339-341 (fireTickDelay default 30 + nextInt(10)); fire.rs:34-35 |
| canSurvive neighbour rule | VERIFIED | FireBlock.java:146-149 + 301-309; fire.rs:302-308 (down isFaceSturdy(UP) || any neighbour canBurn) |
| getStateForPosition shape | VERIFIED | FireBlock.java:126-143; fire.rs:66-93 (flammable/solid below -> default, else per-direction shape) |
| age==15 extinguish gate | VERIFIED | FireBlock.java:181-184; fire.rs:388-394 |
| spread rain gate (5 columns) | VERIFIED | FireBlock.java:216 + 237-243; fire.rs:135-141,463-468 |
| TNT replaced before priming | VERIFIED | FireBlock.java:257-294; fire.rs:206-226 (aged-fire branch writes air for TNT, spawn_primed after removal) |
| waterlogged ignite odds = 0 | VERIFIED | FireBlock.java:251-255 + 311-328; fire.rs:40-49 + test waterlogged_flammable_has_zero_ignite_odds:517-526 |
| fire tick timer in apply_fire_collision | VERIFIED | world/level/block/BaseFireBlock.java:142-166 |
| portal facing threading (item/dispenser) | VERIFIED | world/item/FlintAndSteelItem.java:34, FireChargeItem.java:38, core/dispenser/FlintAndSteelDispenseItemBehavior.java:36 |
| fire_spread_radius_around_player sentinel | VERIFIED | server/level/ServerLevel.java:4741-4744, ChunkMap.java:777-787/869-876; fire.rs:158-182 and on_scheduled_tick:324-326 |
| explosion fire placement via get_placement_state | VERIFIED | world/level/ServerExplosion.java:672-681 (BaseFireBlock.getState); world/explosion.rs:454 |

Sources re-read this pass: fire.rs (whole file), world/explosion.rs:438-454, plus grep confirming lightning.rs:112/120, small_fireball.rs:118 and ignition.rs:47 route through the shared helpers. No unwrap/expect/panic, no println, no em dashes introduced; no source file touched so rustfmt is not required. Compile UNVERIFIED by worker (orchestrator runs scripts/integrate.sh).

## Result (x_01)
- Doc state re-read: all rows closed in earlier passes. Spot-checked `block/blocks/fire/fire.rs:203,247-248` (vanilla `FireBlock.checkBurnOut`/`onPlace`/`tick` comments) present. Full per-row re-verification NOT done this pass.
Files edited: none. Checks: none run.
