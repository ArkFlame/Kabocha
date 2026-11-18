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
