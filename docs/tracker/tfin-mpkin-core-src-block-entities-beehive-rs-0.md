# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-entities-beehive-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain. No generated-data or whole-subsystem blocker.

## Result (FINAL PASS, 2026-11-18)

Verified the single referenced item against decompiled vanilla MC 26.3:
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java`
and `.vendor/vanilla-decompiled/net/minecraft/core/BlockPos.java`. Re-read the current
`crates/pumpkin-core/src/block/entities/beehive.rs`. No STILL OPEN rows existed on entry;
no source edit was needed this pass. No unwrap/expect/panic added. rustfmt is run by the
orchestrator.

- **own/tentities-bee-8270b4: FIXED / VERIFIED.** `BeehiveBlockEntity.java:83` `MAX_OCCUPANTS=3`
  matches `MAX_OCCUPANTS` (beehive.rs:166); `:85-86` nectar/nectarless 2400/600-tick stays match
  `NECTAR_TICKS_IN_HIVE`/`MIN_TICKS_IN_HIVE` (beehive.rs:167-168, 206-213). `addOccupant` at
  `:174-202` and `Occupant.of` at `:438-446` are implemented by `add_occupant` (beehive.rs:190-226,
  ticks 0, nectar-based min ticks). The flower-adoption rule at `:190`
  (`hasSavedFlowerPos() && (!hasSavedFlowerPos() || random.nextBoolean())`) matches
  `adopts_flower`/its call (beehive.rs:144, 221). `releaseOccupant` at `:214-296` matches
  `release_occupant` (beehive.rs:230-274): noGravity spawn, front-face offset from the FACING
  property (`front_offset`, beehive.rs:125-132; vanilla `:229-245`), `dropOffNectar` timer clears
  (`clear_carried_nectar`, beehive.rs:149-155; vanilla `Bee.dropOffNectar`) and the 90% flower
  inheritance rule at `:251` (`inherits_flower`, beehive.rs:159, 243). `tickOccupants` at `:302-329`
  matches `tick_occupants` (beehive.rs:312-354). `saveAdditional`/`loadAdditional` at `:342-355`
  store `flower_pos` via `BlockPos.CODEC`, which is `Codec.INT_STREAM`
  (BlockPos.java:32-36, a 3-int array), matching `write_nbt`/`chunk_data_nbt` (beehive.rs:67-71,
  88-92) and the tolerant `from_nbt` (beehive.rs:37-50). `collect`/`applyImplicitComponents`
  at `:358-376` match `collect_item_components`/`apply_item_components` (beehive.rs:96-117).
  Pure helper tests cover the flower rules, front-face offsets, nectar clearing and position
  parsing (beehive.rs:357-426). The six sub-gaps of the referenced audit are closed in
  `docs/tracker/tentities-bee-8270b4.md`.

No row remains STILL OPEN. No BLOCKED-DATA: no missing generated `pumpkin-data` field and no
missing subsystem. Deliberate approximations already recorded in the referenced audit (fire-nearby
scan, `BEES_STAY_IN_HIVE`/front-blocked release gates, sedated-campfire handling, honey-level
increment on HONEY_DELIVERED, bee release age adjustment) are not flagged rows.
