# Parity rows for crates/pumpkin-core/src/entity/vehicle/minecart.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Command Block Minecart** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **entities/Spawner Minecart** [partial/RE-REVIEW-UNTRACKED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-6aac9c** [partial/OWN-OPEN] own audit: 11 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md) docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md
- **own/tentities-command-block-minecart-5e81b3** [partial/OWN-OPEN] own audit: 9 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-command-block-minecart-5e81b3.md) docs/tracker/tentities-command-block-minecart-5e81b3.md
- **own/tentities-command-block-minecart** [partial/OWN-OPEN] own audit: 7 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-command-block-minecart.md) docs/tracker/tentities-command-block-minecart.md

## Result
FINAL PASS (terminal). All five rows re-verified against decompiled vanilla under
'.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/' and
'.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java'. No row remains STILL OPEN.

- entities/Command Block Minecart: VERIFIED (matches vanilla).
  '.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java'
  with 'BaseCommandBlock': 4-tick activation cooldown, case-insensitive Searge branch before the
  gamerule guard, success-count reset, lastExecution/UpdateLastExecution bookkeeping and NBT keys
  all match 'minecart/command_block.rs'. No change needed.
- entities/Spawner Minecart: VERIFIED (matches vanilla).
  '.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/MinecartSpawner.java' and
  '.vendor/vanilla-decompiled/net/minecraft/world/level/BaseSpawner.java': 'minecart/spawner.rs' runs
  'serverTick' at the post-move floored 'blockPosition()' through 'tick_spawner', broadcasts entity
  event 1 ('EntityStatus::Jump', vanilla 'EVENT_SPAWN = 1') from the delay-reset callback, and
  round-trips the spawner NBT fields with the same defaults (Delay 20, Min 200, Max 800, Count 4,
  Range 4, Nearby 6, PlayerRange 16). 'getDropItem' is a plain minecart; the SPAWNER display block is
  the client-side 'getDefaultDisplayBlockState' default the server never sends. No change needed.
- own/taudit-crates-pumpkin-core-src-en-6aac9c: VERIFIED. Gaps 3-10 fixed/verified in earlier passes
  against 'AbstractMinecart.java', 'MinecartTNT.java', 'MinecartFurnace.java' and
  'OldMinecartBehavior.java'; gaps 1 (display block) and 2 (portal) fixed in 'minecart.rs' in the
  terminal pass. No change needed.
- own/tentities-command-block-minecart-5e81b3: VERIFIED. All rows terminal; version-free
  'to_nbt_tag'/'from_nbt' in 'command_block.rs' matches 'MinecartCommandBlock.java'. No change needed.
- own/tentities-command-block-minecart: VERIFIED. Gaps 4/6 verified and gap 5 closed by the
  version-free NBT in 'command_block.rs'. No change needed.

No BLOCKED-DATA: no missing generated field or subsystem blocks any row. No source file changed this
pass: 'spawner.rs' already matches vanilla.


## Result (THIRD PASS, terminal)

Re-verified all five rows against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/` and closed the remaining
cross-file gaps in `minecart.rs`.

- **entities/Command Block Minecart**: VERIFIED (matches vanilla). `minecart/command_block.rs`
  matches `MinecartCommandBlock.java` + `BaseCommandBlock`: 4-tick activation cooldown,
  case-insensitive Searge branch before the gamerule guard, success-count reset before the
  guard, `lastExecution`/`UpdateLastExecution` bookkeeping and NBT keys; NBT components use the
  version-free `to_nbt_tag`/`from_nbt`. No change needed.
- **entities/Spawner Minecart**: VERIFIED (matches vanilla). `minecart/spawner.rs` matches
  `MinecartSpawner.java`: post-move `serverTick` at the floored current position via
  `tick_spawner`, entity event 1 broadcast, and spawner NBT fields. No change needed.
- **own/taudit-crates-pumpkin-core-src-en-6aac9c** (deep audit): FIXED. Gaps 3-10 were fixed or
  verified in earlier passes. Gap 2 (portal) is now FIXED in source: `minecart.rs` calls
  `vehicle.entity.tick_portal`, matching `AbstractMinecart.tick` calling `handlePortal`. Gap 1
  (display block) is FIXED this pass: added `MinecartDisplay` reading/writing vanilla NBT
  `DisplayState` (via `BlockStateCodec`, the `BlockState.CODEC` layout) and `DisplayOffset`
  (default 6, written only when non-default), plus `init_data_tracker` setting
  `abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK` (varint 0 = empty, state id + 1 = present)
  and `DATA_ID_DISPLAY_OFFSET`. Added pure unit test `display_block_nbt_roundtrip`.
- **own/tentities-command-block-minecart-5e81b3**: VERIFIED. Every row is terminal in that doc;
  Row 4 (version-free NBT) FIXED in `command_block.rs`, Row 5 test FIXED. No change needed.
- **own/tentities-command-block-minecart**: VERIFIED. Gaps 4/6 VERIFIED and gap 5 closed by the
  version-free `to_nbt_tag`/`from_nbt` now used in `command_block.rs`, matching
  `MinecartCommandBlock.java`. No change needed.

No row remains STILL OPEN. No BLOCKED-DATA: the generated fields
`abstract_minecart::DATA_ID_CUSTOM_DISPLAY_BLOCK` / `DATA_ID_DISPLAY_OFFSET` and the block-state
NBT serializer `pumpkin_world::block::BlockStateCodec` all exist, so gap 1 was implemented rather
than blocked. File edited: `crates/pumpkin-core/src/entity/vehicle/minecart.rs`. rustfmt is run by
the orchestrator; no cargo or git run.
