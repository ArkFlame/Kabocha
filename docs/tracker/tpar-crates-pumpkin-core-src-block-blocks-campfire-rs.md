# Parity rows for crates/pumpkin-core/src/block/blocks/campfire.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Campfire** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-campfire-dddaf9** [partial/OWN-OPEN] own audit: 4 open, 6 fixed, 3 verified, 0 blocked-data (docs/tracker/tblocks-campfire-dddaf9.md) docs/tracker/tblocks-campfire-dddaf9.md
- **own/tblocks-campfire** [partial/OWN-OPEN] own audit: 4 open, 3 fixed, 8 verified, 0 blocked-data (docs/tracker/tblocks-campfire.md) docs/tracker/tblocks-campfire.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

Third pass (2026-11-18). Re-reviewed the full current source of both `campfire.rs`
files against the vendored decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/CampfireBlock.java`
and
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CampfireBlockEntity.java`.
Every row is resolved; no `.rs` edit was required this pass.

- **blocks/Campfire** [AUDITED]: VERIFIED, matches vanilla, no change. Placement
  (`blocks/campfire.rs:230-238`) == `CampfireBlock.getStateForPlacement`
  (`CampfireBlock.java:129-138`): `lit = !waterlogged`, `signal_fire` from the
  hay block below, `facing` from the player. Neighbour update (`:240-261`) ==
  `updateShape` (`:141-158`): schedules the water fluid tick and recomputes
  `SIGNAL_FIRE` on a `DOWN` update, never writing `LIT`. `is_smokey_pos_from`
  (`:44-77`) == `isSmokeyPos` (`:268-284`): scans 1..=5, intersects the
  0.375..0.625 virtual post and returns `isLitCampfire(posToCheck.below())` at a
  blocker; pure test `smoke_scans_five_cells_and_stops_at_a_blocker` (`:277`)
  covers it. Collision damage (`:178-209`), extinguish (`:151-176`) and
  projectile relight (`:211-228`) also match.
- **own/tblocks-campfire-dddaf9** [OWN-OPEN]: VERIFIED, all rows closed. The own
  doc's last Result is `PASS fixed=1 open=0`: rows 1-6 fixed/verified, Row 7
  (crackle/smoke/flame) FIXED via `CampfireBlockEntity::particle_tick`
  (`entities/campfire.rs:355-410`). Re-checked `particle_tick` against
  `CampfireBlockEntity.particleTick` (`:165-188`) and
  `CampfireBlock.makeParticles` (`:242-266`): 11% puff chance, 2-3 puffs,
  cosy/signal smoke with +/-1/3 jitter and `y = random + random`, velocity 0.07;
  per-slot 20% smoke at 0.3125 facing/clockwise offsets, velocity 5e-4. All
  match, with pure seams `facing_2d_data_value`/`item_particle_offset` and tests.
- **own/tblocks-campfire** [OWN-OPEN]: VERIFIED, all rows closed. The own doc's
  last Result marks Row 3 FIXED; rows 1-2 (`cooking_cancelled` skip, increment
  before recipe lookup with raw-stack fallback), Row 12 (`isSmokeyPos`) and the
  row-14 Y-axis correction (`cooked_item_spawn_position` at block centre) are
  all fixed/verified. Re-checked `tick` (`entities/campfire.rs:98-216`) against
  `CampfireBlockEntity.cookTick` (`:80-144`): increment per non-empty stack,
  recipe at completion with `.orElse(itemStack)` fallback, `BlockCookEvent`
  cancel restarts the slot, result split into `nextInt(21)+10` (10-30) chunks at
  `pos+0.5` with triangle velocity, `BLOCK_CHANGE` game event. `cooldownTick`
  (`:146-163`) matches `cool_down` (`:432-449`). NBT keys match
  `loadAdditional`/`saveAdditional` (`:195-235`).

No generated data, WIT, cross-crate refactor or file outside this unit was
touched. Re-read the diff against the Worker compile pitfalls table: no source
diff this pass, so nothing to correct. Flame rendering / crackle audio remain
client-side cosmetic effects with no server seam; noted, non-blocking.

Status: PASS fixed=0 open=0 files=none
