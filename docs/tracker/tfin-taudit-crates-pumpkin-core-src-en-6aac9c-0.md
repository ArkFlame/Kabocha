# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Was open: gap 1 (M) needs the vanilla NBT key (`DisplayTile` vs newer `DisplayState`) and block-state tracked-data serializer confirmed against decompiled `AbstractMinecart`; `.vendor/` is empty and no cargo run was allowed.
Was open: gap 2 (M) needs non-player dimension change support, which does not exist yet; L-scale infrastructure risk.
Was open: gap 3 (S) vanilla `MinecartFurnace`/`MinecartCommandBlock` tick ordering is marked verify and not confirmed against decompiled source.
Was open: gap 4 (S) vanilla entry point for flint-and-steel on `MinecartTNT` is marked verify.

## Result

Verified against decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/`.

- Gap 1 (M) FIXED: `MinecartDisplay` stores custom block state and offset, reads/writes vanilla NBT
  `DisplayState` (via `BlockStateCodec`) and `DisplayOffset` (default 6), and `init_data_tracker` sets
  `abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK` (varint 0 = empty, state id + 1 = present) and
  `DATA_ID_DISPLAY_OFFSET`. Pure unit test `display_block_nbt_roundtrip` covers the round trip.
  Vanilla: `AbstractMinecart.addAdditionalSaveData` / `readAdditionalSaveData`, `getDisplayBlockState`,
  `getDisplayOffset`.
- Gap 2 (M) FIXED: `tick` calls `vehicle.entity.tick_portal(caller)`, matching
  `AbstractMinecart.tick` -> `handlePortal` in
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/AbstractMinecart.java`.
- Gap 3 (S) VERIFIED, no change: decompiled `MinecartFurnace.tick` calls `super.tick()` (movement)
  before decrementing fuel and emitting smoke, and Pumpkin already runs the furnace tick after
  movement at the end of `MinecartEntity::tick`. `MinecartCommandBlock` has no `tick` override in
  vanilla (activation via `activateMinecart` on activator rails), and Pumpkin's `step_tick` before
  rail activation matches the `tickCount - lastActivated >= 4` cooldown.
  Vanilla: `MinecartFurnace.java:50-67`, `MinecartCommandBlock.java` (no tick override).
- Gap 4 (S) VERIFIED, no change: decompiled `MinecartTNT` has no `interact` override and neither
  `FlintAndSteelItem` nor `FireChargeItem` references TNT minecarts, so flint-and-steel does not
  prime a TNT minecart; Pumpkin returning false for `MinecartKind::Tnt` in `interact` is correct.
  Vanilla: `MinecartTNT.java` (no `interact` override).

No gap remains open. No BLOCKED-DATA. rustfmt is run by the orchestrator; no cargo or git run.
