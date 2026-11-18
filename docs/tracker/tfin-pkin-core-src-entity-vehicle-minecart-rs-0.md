# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-vehicle-minecart-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/,
change the Rust to match (edit any file needed, no cargo, rustfmt touched files). No row remains STILL
OPEN; no BLOCKED-DATA: the generated tracked-data fields and the block-state NBT serializer all exist.

Rows (from the referenced tracker doc):
- entities/Command Block Minecart
- entities/Spawner Minecart
- own/taudit-crates-pumpkin-core-src-en-6aac9c (deep minecart.rs audit)
- own/tentities-command-block-minecart-5e81b3
- own/tentities-command-block-minecart

## Result (FINAL PASS, terminal)

All five rows re-verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`,
`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseCommandBlock.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java`. Every row ends FIXED or
VERIFIED; none is STILL OPEN or BLOCKED-DATA.

- **entities/Command Block Minecart**: VERIFIED (matches vanilla). `minecart/command_block.rs`
  matches `MinecartCommandBlock.java` (4-tick `ACTIVATION_DELAY`, `getDropItem` = plain minecart,
  `interact` creative/permission gate, synced `DATA_ID_COMMAND_NAME`/`DATA_ID_LAST_OUTPUT`) and
  `BaseCommandBlock.java#performCommand` (`getGameTime == lastExecution` early return, case-insensitive
  `Searge` branch with early return before the `commandBlocksWork` guard, `successCount` reset before
  the guard, `lastOutput` clear, `UpdateLastExecution`/`LastExecution` bookkeeping). NBT keys
  `Command`, `SuccessCount`, `CustomName`, `TrackOutput`, `LastOutput`, `LastExecution` match. No change.
- **entities/Spawner Minecart**: VERIFIED (matches vanilla). `minecart/spawner.rs` matches
  `MinecartSpawner.java` (`spawner.serverTick(serverLevel, this.blockPosition())` run after
  `super.tick()`, `broadcastEntityEvent` id from `EVENT_SPAWN = 1` in `BaseSpawner.java`, NBT load/save
  via `BaseSpawner`, plain-minecart `getDropItem`) and the defaults 20/200/800/4/4/6/16. No change.
- **own/taudit-crates-pumpkin-core-src-en-6aac9c** (deep audit): FIXED + VERIFIED. Gaps 3-10 were
  closed in prior passes (furnace post-move tick, `position_riders` seat offsets, loot unpack only in
  `get_container_inventory`, TNT fuse guard, direction push, table-driven rail snap test).
  Gap 2 (portal) FIXED: `minecart.rs` calls `vehicle.entity.tick_portal`, behaviourally equal to
  `AbstractMinecart.tick`/handlePortal (vanilla calls it directly; this Paper build moves it to
  `postTick` with the call commented out at `AbstractMinecart.java:311`). Gap 1 (display block) FIXED:
  `MinecartDisplay` reads/writes vanilla NBT `DisplayState` (`BlockState.CODEC`) and `DisplayOffset`
  (default 6, written only when non-default) and sets the generated
  `abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK`/`DATA_ID_DISPLAY_OFFSET` (varint 0 empty, state+1
  present); pure unit test `display_block_nbt_roundtrip` added. No change this pass.
- **own/tentities-command-block-minecart-5e81b3**: VERIFIED. Version-free `to_nbt_tag`/`from_nbt` in
  `command_block.rs` matches `MinecartCommandBlock.java`/`BaseCommandBlock.java`; all rows terminal.
  No change.
- **own/tentities-command-block-minecart**: VERIFIED. Gaps 4/6 VERIFIED and gap 5 closed by the
  version-free NBT now in `command_block.rs`, matching `MinecartCommandBlock.java`. No change.

No BLOCKED-DATA: generated fields `abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK` /
`DATA_ID_DISPLAY_OFFSET` exist in `crates/pumpkin-data/src/generated/tracked_data.rs` and the
`pumpkin_world::block::BlockStateCodec` serializer exists in `crates/pumpkin-world/src/block/mod.rs`.
No source file changed this pass; every fix was already present and re-verified. rustfmt is run by the
orchestrator; no cargo or git run.
